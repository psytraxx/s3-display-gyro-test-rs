use embedded_graphics::geometry::Point;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};

// Tilt indicator layout constants
pub const TILT_CENTER_X: i32 = 85;
pub const TILT_CENTER_Y: i32 = 85;
pub const TILT_OUTER_RADIUS: u32 = 80;
pub const TILT_INNER_RADIUS: u32 = 60;
pub const BUBBLE_RADIUS: u32 = 12;
pub const MAX_BUBBLE_OFFSET: i32 = 65;
pub const CROSSHAIR_LENGTH: i32 = 10;

// Gyroscope bar layout constants
pub const BAR_WIDTH: u32 = 43;
pub const BAR_HEIGHT: u32 = 140;
pub const BAR_Y_START: i32 = 20;
pub const BAR_Y_END: i32 = 160;
pub const BAR_CENTER_Y: i32 = 90;
pub const BAR_MAX_HEIGHT: i32 = 70;

// Bar X positions for each axis
pub const BAR_X_X: i32 = 175;
pub const BAR_Y_X: i32 = 223;
pub const BAR_Z_X: i32 = 271;

// Scaling constants
pub const TILT_SCALE_FACTOR: i32 = 8192;
pub const GYRO_SCALE_FACTOR: i32 = 16384;
pub const TILT_DEAD_ZONE: i16 = 2000;

// Color constants
pub const COLOR_OUTER_CIRCLE: Rgb565 = Rgb565::new(12, 25, 12);  // Medium gray
pub const COLOR_INNER_CIRCLE: Rgb565 = Rgb565::new(7, 15, 7);    // Dark gray
pub const COLOR_BAR_BACKGROUND: Rgb565 = Rgb565::new(3, 7, 3);   // Very dark gray

pub struct BarFill {
    pub start_y: i32,
    pub height: u32,
    pub color: Rgb565,
}

/// Calculate bubble position based on accelerometer values
pub fn calculate_bubble_position(accel_x: i16, accel_y: i16) -> Point {
    let offset_x = ((accel_x as i32 * MAX_BUBBLE_OFFSET) / TILT_SCALE_FACTOR)
        .clamp(-MAX_BUBBLE_OFFSET, MAX_BUBBLE_OFFSET);
    // Invert Y axis so tilting forward moves bubble up (intuitive for bubble level)
    let offset_y = ((-accel_y as i32 * MAX_BUBBLE_OFFSET) / TILT_SCALE_FACTOR)
        .clamp(-MAX_BUBBLE_OFFSET, MAX_BUBBLE_OFFSET);

    Point::new(TILT_CENTER_X + offset_x, TILT_CENTER_Y + offset_y)
}

/// Determine bubble color based on tilt (green when level, yellow when tilted)
pub fn calculate_bubble_color(accel_x: i16, accel_y: i16) -> Rgb565 {
    if accel_x.abs() < TILT_DEAD_ZONE && accel_y.abs() < TILT_DEAD_ZONE {
        Rgb565::GREEN
    } else {
        Rgb565::YELLOW
    }
}

/// Calculate bar fill parameters for gyroscope visualization
pub fn calculate_bar_fill(gyro_value: i16) -> BarFill {
    let scaled = ((gyro_value as i32 * BAR_MAX_HEIGHT) / GYRO_SCALE_FACTOR)
        .clamp(-BAR_MAX_HEIGHT, BAR_MAX_HEIGHT);

    if scaled >= 0 {
        // Positive rotation: fill upward from center
        BarFill {
            start_y: BAR_CENTER_Y - scaled,
            height: scaled as u32,
            color: Rgb565::BLUE,
        }
    } else {
        // Negative rotation: fill downward from center
        BarFill {
            start_y: BAR_CENTER_Y,
            height: (-scaled) as u32,
            color: Rgb565::RED,
        }
    }
}
