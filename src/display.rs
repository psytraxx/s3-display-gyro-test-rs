use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::mono_font::iso_8859_1::FONT_10X20 as FONT;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, Rectangle};
use embedded_graphics::Drawable;
use embedded_hal::delay::DelayNs;
use embedded_text::alignment::HorizontalAlignment;
use embedded_text::style::{HeightMode, TextBoxStyleBuilder};
use embedded_text::TextBox;
use esp_hal::gpio::{AnyPin, Level, Output, OutputConfig};
use mipidsi::interface::{Generic8BitBus, ParallelError, ParallelInterface};
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use mipidsi::{Builder, Display as MipiDisplay};

use crate::config::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::imu::ImuData;
use crate::visualization::*;

const TEXT_STYLE: MonoTextStyle<Rgb565> = MonoTextStyle::new(&FONT, Rgb565::WHITE);

type MipiDisplayWrapper<'a> = MipiDisplay<
    ParallelInterface<
        Generic8BitBus<
            Output<'a>,
            Output<'a>,
            Output<'a>,
            Output<'a>,
            Output<'a>,
            Output<'a>,
            Output<'a>,
            Output<'a>,
        >,
        Output<'a>,
        Output<'a>,
    >,
    ST7789,
    Output<'a>,
>;

pub struct DisplayState {
    last_bubble_pos: Point,
    last_bar_fills: [i32; 3],
    initialized: bool,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            last_bubble_pos: Point::new(TILT_CENTER_X, TILT_CENTER_Y),
            last_bar_fills: [0, 0, 0],
            initialized: false,
        }
    }
}

pub struct Display<'a, D: DelayNs> {
    display: MipiDisplayWrapper<'a>,
    backlight: Output<'a>,
    delay: D,
    state: DisplayState,
}

pub trait DisplayTrait {
    fn write_multiline(&mut self, text: &str) -> Result<(), Error>;
    fn enable_powersave(&mut self) -> Result<(), Error>;
    fn draw_sensor_visualization(&mut self, data: &ImuData) -> Result<(), Error>;
}

pub struct DisplayPeripherals {
    pub rst: AnyPin<'static>,
    pub cs: AnyPin<'static>,
    pub dc: AnyPin<'static>,
    pub wr: AnyPin<'static>,
    pub rd: AnyPin<'static>,
    pub power_en: AnyPin<'static>,
    pub backlight: AnyPin<'static>,
    pub d0: AnyPin<'static>,
    pub d1: AnyPin<'static>,
    pub d2: AnyPin<'static>,
    pub d3: AnyPin<'static>,
    pub d4: AnyPin<'static>,
    pub d5: AnyPin<'static>,
    pub d6: AnyPin<'static>,
    pub d7: AnyPin<'static>,
}

impl<D: DelayNs> Display<'_, D> {
    pub fn new(p: DisplayPeripherals, mut delay: D) -> Result<Self, Error> {
        // Enable power for T-Display S3 (needed when USB powered)
        let mut power_en = Output::new(p.power_en, Level::High, OutputConfig::default());
        power_en.set_high();

        let backlight = Output::new(p.backlight, Level::Low, OutputConfig::default());

        let dc = Output::new(p.dc, Level::Low, OutputConfig::default());
        let mut cs = Output::new(p.cs, Level::Low, OutputConfig::default());
        let rst = Output::new(p.rst, Level::Low, OutputConfig::default());
        let wr = Output::new(p.wr, Level::Low, OutputConfig::default());
        let mut rd = Output::new(p.rd, Level::Low, OutputConfig::default());

        cs.set_low();
        rd.set_high();

        let d0 = Output::new(p.d0, Level::Low, OutputConfig::default());
        let d1 = Output::new(p.d1, Level::Low, OutputConfig::default());
        let d2 = Output::new(p.d2, Level::Low, OutputConfig::default());
        let d3 = Output::new(p.d3, Level::Low, OutputConfig::default());
        let d4 = Output::new(p.d4, Level::Low, OutputConfig::default());
        let d5 = Output::new(p.d5, Level::Low, OutputConfig::default());
        let d6 = Output::new(p.d6, Level::Low, OutputConfig::default());
        let d7 = Output::new(p.d7, Level::Low, OutputConfig::default());

        let bus = Generic8BitBus::new((d0, d1, d2, d3, d4, d5, d6, d7));

        let di = ParallelInterface::new(bus, dc, wr);

        let display = Builder::new(mipidsi::models::ST7789, di)
            .display_size(DISPLAY_HEIGHT, DISPLAY_WIDTH)
            .display_offset((240 - DISPLAY_HEIGHT) / 2, 0)
            .orientation(Orientation::new().rotate(Rotation::Deg270))
            .invert_colors(ColorInversion::Inverted)
            .reset_pin(rst)
            .init(&mut delay)
            .map_err(|_| Error::InitError)?;

        Ok(Self {
            display,
            backlight,
            delay,
            state: DisplayState::default(),
        })
    }

    fn disable_powersave(&mut self) -> Result<(), Error> {
        self.backlight.set_high();
        self.display.wake(&mut self.delay)?;
        self.display.clear(RgbColor::BLACK)?;
        Ok(())
    }

    fn draw_static_elements(&mut self) -> Result<(), Error> {
        // Draw tilt indicator circles
        Circle::new(
            Point::new(
                TILT_CENTER_X - TILT_OUTER_RADIUS as i32,
                TILT_CENTER_Y - TILT_OUTER_RADIUS as i32,
            ),
            TILT_OUTER_RADIUS * 2,
        )
        .into_styled(PrimitiveStyle::with_stroke(COLOR_OUTER_CIRCLE, 3))
        .draw(&mut self.display)?;

        Circle::new(
            Point::new(
                TILT_CENTER_X - TILT_INNER_RADIUS as i32,
                TILT_CENTER_Y - TILT_INNER_RADIUS as i32,
            ),
            TILT_INNER_RADIUS * 2,
        )
        .into_styled(PrimitiveStyle::with_stroke(COLOR_INNER_CIRCLE, 2))
        .draw(&mut self.display)?;

        // Draw crosshair
        Line::new(
            Point::new(TILT_CENTER_X - CROSSHAIR_LENGTH, TILT_CENTER_Y),
            Point::new(TILT_CENTER_X + CROSSHAIR_LENGTH, TILT_CENTER_Y),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
        .draw(&mut self.display)?;

        Line::new(
            Point::new(TILT_CENTER_X, TILT_CENTER_Y - CROSSHAIR_LENGTH),
            Point::new(TILT_CENTER_X, TILT_CENTER_Y + CROSSHAIR_LENGTH),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
        .draw(&mut self.display)?;

        // Draw gyro bar backgrounds and center lines
        for &bar_x in &[BAR_X_X, BAR_Y_X, BAR_Z_X] {
            Rectangle::new(
                Point::new(bar_x, BAR_Y_START),
                Size::new(BAR_WIDTH, BAR_HEIGHT),
            )
            .into_styled(PrimitiveStyle::with_fill(COLOR_BAR_BACKGROUND))
            .draw(&mut self.display)?;

            Line::new(
                Point::new(bar_x, BAR_CENTER_Y),
                Point::new(bar_x + BAR_WIDTH as i32, BAR_CENTER_Y),
            )
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
            .draw(&mut self.display)?;
        }

        Ok(())
    }

    fn update_tilt_bubble(&mut self, accel_x: i16, accel_y: i16) -> Result<(), Error> {
        // Clear old bubble
        if self.state.initialized {
            Circle::new(
                Point::new(
                    self.state.last_bubble_pos.x - BUBBLE_RADIUS as i32,
                    self.state.last_bubble_pos.y - BUBBLE_RADIUS as i32,
                ),
                BUBBLE_RADIUS * 2,
            )
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(&mut self.display)?;
        }

        // Redraw circles (they might have been erased by clearing the bubble)
        Circle::new(
            Point::new(
                TILT_CENTER_X - TILT_OUTER_RADIUS as i32,
                TILT_CENTER_Y - TILT_OUTER_RADIUS as i32,
            ),
            TILT_OUTER_RADIUS * 2,
        )
        .into_styled(PrimitiveStyle::with_stroke(COLOR_OUTER_CIRCLE, 3))
        .draw(&mut self.display)?;

        Circle::new(
            Point::new(
                TILT_CENTER_X - TILT_INNER_RADIUS as i32,
                TILT_CENTER_Y - TILT_INNER_RADIUS as i32,
            ),
            TILT_INNER_RADIUS * 2,
        )
        .into_styled(PrimitiveStyle::with_stroke(COLOR_INNER_CIRCLE, 2))
        .draw(&mut self.display)?;

        // Redraw crosshair
        Line::new(
            Point::new(TILT_CENTER_X - CROSSHAIR_LENGTH, TILT_CENTER_Y),
            Point::new(TILT_CENTER_X + CROSSHAIR_LENGTH, TILT_CENTER_Y),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
        .draw(&mut self.display)?;

        Line::new(
            Point::new(TILT_CENTER_X, TILT_CENTER_Y - CROSSHAIR_LENGTH),
            Point::new(TILT_CENTER_X, TILT_CENTER_Y + CROSSHAIR_LENGTH),
        )
        .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
        .draw(&mut self.display)?;

        // Calculate new position and color
        let bubble_pos = calculate_bubble_position(accel_x, accel_y);
        let bubble_color = calculate_bubble_color(accel_x, accel_y);

        // Draw new bubble (filled only, no outline)
        Circle::new(
            Point::new(
                bubble_pos.x - BUBBLE_RADIUS as i32,
                bubble_pos.y - BUBBLE_RADIUS as i32,
            ),
            BUBBLE_RADIUS * 2,
        )
        .into_styled(PrimitiveStyle::with_fill(bubble_color))
        .draw(&mut self.display)?;

        self.state.last_bubble_pos = bubble_pos;
        Ok(())
    }

    fn update_gyro_bars(&mut self, gyro_x: i16, gyro_y: i16, gyro_z: i16) -> Result<(), Error> {
        let gyro_values = [gyro_x, gyro_y, gyro_z];
        let bar_positions = [BAR_X_X, BAR_Y_X, BAR_Z_X];

        for (i, (&bar_x, &gyro_val)) in bar_positions.iter().zip(gyro_values.iter()).enumerate() {
            // Clear old bar fill
            if self.state.last_bar_fills[i] != 0 {
                let clear_start = if self.state.last_bar_fills[i] > 0 {
                    BAR_CENTER_Y - self.state.last_bar_fills[i]
                } else {
                    BAR_CENTER_Y
                };
                let clear_height = self.state.last_bar_fills[i].unsigned_abs();

                Rectangle::new(
                    Point::new(bar_x, clear_start),
                    Size::new(BAR_WIDTH, clear_height),
                )
                .into_styled(PrimitiveStyle::with_fill(COLOR_BAR_BACKGROUND))
                .draw(&mut self.display)?;
            }

            // Calculate and draw new bar fill
            let bar_fill = calculate_bar_fill(gyro_val);
            if bar_fill.height > 0 {
                Rectangle::new(
                    Point::new(bar_x, bar_fill.start_y),
                    Size::new(BAR_WIDTH, bar_fill.height),
                )
                .into_styled(PrimitiveStyle::with_fill(bar_fill.color))
                .draw(&mut self.display)?;
            }

            // Redraw center line
            Line::new(
                Point::new(bar_x, BAR_CENTER_Y),
                Point::new(bar_x + BAR_WIDTH as i32, BAR_CENTER_Y),
            )
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::WHITE, 1))
            .draw(&mut self.display)?;

            // Update state
            self.state.last_bar_fills[i] = if bar_fill.height > 0 {
                if bar_fill.color == Rgb565::BLUE {
                    bar_fill.height as i32
                } else {
                    -(bar_fill.height as i32)
                }
            } else {
                0
            };
        }

        Ok(())
    }

}

impl<D: DelayNs> DisplayTrait for Display<'_, D> {
    fn write_multiline(&mut self, text: &str) -> Result<(), Error> {
        self.disable_powersave()?;
        let textbox_style = TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .alignment(HorizontalAlignment::Justified)
            .build();

        let text_box = TextBox::with_textbox_style(
            text,
            self.display.bounding_box(),
            TEXT_STYLE,
            textbox_style,
        );
        text_box.draw(&mut self.display)?;
        Ok(())
    }

    fn enable_powersave(&mut self) -> Result<(), Error> {
        self.backlight.set_low();
        self.display.sleep(&mut self.delay)?;
        Ok(())
    }

    fn draw_sensor_visualization(&mut self, data: &ImuData) -> Result<(), Error> {
        if !self.state.initialized {
            self.disable_powersave()?;
            self.draw_static_elements()?;
            self.state.initialized = true;
        }
        self.update_tilt_bubble(data.accel_x, data.accel_y)?;
        self.update_gyro_bars(data.gyro_x, data.gyro_y, data.gyro_z)?;
        Ok(())
    }

}

#[derive(Debug)]
pub enum Error {
    DisplayInterface(&'static str),
    InitError,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::DisplayInterface(msg) => write!(f, "Display interface error: {msg}"),
            Error::InitError => write!(f, "Display initialization error"),
        }
    }
}

impl<BUS, DC, WR> From<ParallelError<BUS, DC, WR>> for Error {
    fn from(e: ParallelError<BUS, DC, WR>) -> Self {
        match e {
            ParallelError::Bus(_) => Self::DisplayInterface("Bus error"),
            ParallelError::Dc(_) => Self::DisplayInterface("Data/command pin error"),
            ParallelError::Wr(_) => Self::DisplayInterface("Write pin error"),
        }
    }
}
