# S3 Display Gyro Test

Real-time graphical visualization of BMI160 IMU sensor data on LilyGo T-Display S3.

## Overview

This project displays accelerometer and gyroscope data from a BMI160 sensor on an ST7789 LCD display using the ESP32-S3 microcontroller. The visualization includes:

- **Left Panel**: 2D tilt indicator (bubble level) showing device orientation
- **Right Panel**: Triple bar gauges showing gyroscope rotation on X, Y, Z axes

## Hardware

- **Board**: LilyGo T-Display S3 (ESP32-S3)
- **Display**: ST7789 LCD (320×170 pixels, 8-bit parallel interface)
- **Sensor**: BMI160 6-axis IMU (accelerometer + gyroscope)
- **Interface**: I2C (GPIO17 SDA, GPIO18 SCL, 400 kHz)

### Pin Configuration

#### Display (8-bit Parallel)
- RST: GPIO5
- CS: GPIO6
- DC: GPIO7
- WR: GPIO8
- RD: GPIO9
- Power Enable: GPIO15
- Backlight: GPIO38
- Data pins: GPIO39-GPIO42, GPIO45-GPIO48

#### Sensor (I2C)
- SDA: GPIO17
- SCL: GPIO18

## Features

### Tilt Indicator (Accelerometer)
- Circular bubble level centered at (85, 85)
- Outer circle: 80px radius
- Inner circle: 60px radius (dead zone)
- Bubble: 12px radius
  - **Green**: Device is level (±12° tilt)
  - **Yellow**: Device is tilted
- Intuitive movement: Tilt forward → bubble moves up

### Gyroscope Bars
- Three vertical bars (X, Y, Z axes)
- 140px height, centered at Y=90
- **Blue fill**: Positive rotation (upward from center)
- **Red fill**: Negative rotation (downward from center)
- Real-time response to device rotation

### Architecture
- **Embassy-based async tasks**: Separate tasks for sensor reading and display rendering
- **Channel communication**: Sensor task sends data to display task via embassy-sync channel
- **Update rate**: 10 Hz (100ms sensor polling)
- **Rendering**: Partial updates for smooth animation (~10-20ms per frame)

## Dependencies

```toml
esp-hal = "1.0.0"
esp-rtos = "0.2.0"
embassy-executor = "0.9.1"
embassy-time = "0.5.0"
embassy-sync = "0.7.0"
mipidsi = "0.9.0"
embedded-graphics = "0.8.1"
bmi160 = "1.1.0"
```

## Building

### Prerequisites

1. Install Rust and espup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install espup
espup install
```

2. Source the ESP environment:
```bash
. ~/export-esp.sh
```

### Compile

```bash
cargo build --release
```

## Running

### Flash to device

```bash
cargo run --release
```

### Expected Output

Serial console will show:
```
Starting initialization...
Timer group created, starting esp_rtos...
Display initialized successfully
Sensor initialized successfully
Tasks spawned successfully
Sensor task started
Display task started
Accel: [123, -456, 16384] Gyro: [45, -67, 12]
...
```

Display will show:
- Left: Circular tilt indicator with moving bubble
- Right: Three bar gauges responding to device rotation

## Code Structure

```
src/
├── bin/
│   └── main.rs           # Entry point, task spawning, initialization
├── config.rs             # Display dimensions
├── display.rs            # Display driver, rendering logic
├── sensor.rs             # BMI160 sensor interface
├── visualization.rs      # Layout constants, coordinate calculations
└── lib.rs                # Module declarations
```

### Task Architecture

```
┌──────────────┐         Channel          ┌──────────────┐
│ Sensor Task  │ ──────────────────────> │ Display Task │
│              │    (SensorData)          │              │
│ - Reads BMI  │                          │ - Draws viz  │
│ - 100ms loop │                          │ - On demand  │
└──────────────┘                          └──────────────┘
```

## Visualization Details

### Scale Factors
- **Tilt sensitivity**: ±8192 raw units → ±65 pixels (±30° tilt)
- **Gyro sensitivity**: ±16384 raw units → ±70 pixels (±1000 dps)

### Color Scheme
- Background: Black
- Tilt circles: Gray (outer), Dark gray (inner)
- Crosshair/center lines: White
- Bubble: Green (level) / Yellow (tilted)
- Gyro bars: Blue (positive) / Red (negative)
- Bar background: Very dark gray

### Dead Zone
- Tilt dead zone: ±2000 raw units (~±12°) for green indicator

## License

MIT

## Troubleshooting

### Display not working
- Ensure GPIO15 power enable is set HIGH (required for USB power)
- Check 8-bit parallel connection and pin assignments

### Sensor not detected
- Verify I2C connections (GPIO17 SDA, GPIO18 SCL)
- Check I2C address (default: 0x68)
- Ensure sensor is powered

### Build errors
- Source ESP environment: `. ~/export-esp.sh`
- Check Rust toolchain: `rustup show`
- Clean build: `cargo clean && cargo build`

## Credits

Built with:
- [esp-hal](https://github.com/esp-rs/esp-hal) - ESP32 Hardware Abstraction Layer
- [embassy](https://embassy.dev/) - Async embedded framework
- [mipidsi](https://github.com/almindor/mipidsi) - MIPI Display Serial Interface driver
- [embedded-graphics](https://github.com/embedded-graphics/embedded-graphics) - 2D graphics library
- [bmi160](https://github.com/eldruin/bmi160-rs) - BMI160 sensor driver
