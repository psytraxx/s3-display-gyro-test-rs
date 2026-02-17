#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::Pin;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::Blocking;
use log::info;
use s3_display_gyro_test_rs::display::{Display, DisplayPeripherals, DisplayTrait};
use s3_display_gyro_test_rs::sensor::{Sensor, SensorData};
use static_cell::StaticCell;

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

static SENSOR_CHANNEL: StaticCell<Channel<CriticalSectionRawMutex, SensorData, 1>> =
    StaticCell::new();

#[embassy_executor::task]
async fn sensor_task(
    mut sensor: Sensor<I2c<'static, Blocking>, Delay>,
    sender: Sender<'static, CriticalSectionRawMutex, SensorData, 1>,
) {
    info!("Sensor task started");
    loop {
        match sensor.read() {
            Ok(data) => {
                info!(
                    "Accel: [{}, {}, {}] Gyro: [{}, {}, {}]",
                    data.accel_x, data.accel_y, data.accel_z, data.gyro_x, data.gyro_y, data.gyro_z
                );
                sender.send(data).await;
            }
            Err(e) => {
                info!("Failed to read sensor: {}", e);
            }
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::task]
async fn display_task(
    mut display: Display<'static, Delay>,
    receiver: Receiver<'static, CriticalSectionRawMutex, SensorData, 1>,
) {
    info!("Display task started");
    loop {
        let data = receiver.receive().await;
        if let Err(e) = display.draw_sensor_visualization(&data) {
            info!("Failed to draw visualization: {}", e);
        }
    }
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    info!("Starting initialization...");

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 73744);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    info!("Timer group created, starting esp_rtos...");

    esp_rtos::start(timg0.timer0);

    let delay = Delay::new();

    let display_peripherals = DisplayPeripherals {
        rst: peripherals.GPIO5.degrade(),
        cs: peripherals.GPIO6.degrade(),
        dc: peripherals.GPIO7.degrade(),
        wr: peripherals.GPIO8.degrade(),
        rd: peripherals.GPIO9.degrade(),
        power_en: peripherals.GPIO15.degrade(),
        backlight: peripherals.GPIO38.degrade(),
        d0: peripherals.GPIO39.degrade(),
        d1: peripherals.GPIO40.degrade(),
        d2: peripherals.GPIO41.degrade(),
        d3: peripherals.GPIO42.degrade(),
        d4: peripherals.GPIO45.degrade(),
        d5: peripherals.GPIO46.degrade(),
        d6: peripherals.GPIO47.degrade(),
        d7: peripherals.GPIO48.degrade(),
    };

    let display = match Display::new(display_peripherals, delay) {
        Ok(d) => {
            info!("Display initialized successfully");
            d
        }
        Err(e) => {
            info!("Display initialization failed: {}", e);
            loop {
                Timer::after(Duration::from_secs(1)).await;
            }
        }
    };

    let _io = esp_hal::gpio::Io::new(peripherals.IO_MUX);

    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO17)
    .with_scl(peripherals.GPIO18);

    let sensor_delay = Delay::new();

    let sensor = match Sensor::new(i2c, sensor_delay) {
        Ok(s) => {
            info!("Sensor initialized successfully");
            s
        }
        Err(e) => {
            info!("Sensor initialization failed: {}", e);
            loop {
                Timer::after(Duration::from_secs(1)).await;
            }
        }
    };

    // Create channel
    let channel = SENSOR_CHANNEL.init(Channel::new());
    let sender = channel.sender();
    let receiver = channel.receiver();

    // Spawn tasks
    spawner.spawn(sensor_task(sensor, sender)).unwrap();
    spawner.spawn(display_task(display, receiver)).unwrap();

    info!("Tasks spawned successfully");

    // Main task just sleeps
    loop {
        Timer::after(Duration::from_secs(10)).await;
    }
}
