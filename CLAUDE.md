# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a `no_std` embedded Rust application for the **LilyGo T-Display S3** (ESP32-S3) that displays real-time graphical visualization of BMI160 IMU sensor data on an ST7789 LCD screen. Features a 2D tilt indicator (bubble level) and triple gyroscope bar gauges rendered using embedded-graphics.

**Target Hardware**: LilyGo T-Display S3
- ESP32-S3R8 with 16MB Flash, 8MB PSRAM
- 170×320 ST7789 LCD (1.9")
- 8-bit parallel display interface
- BMI160 6-axis IMU (I2C)

## Build & Development Commands

### Building
```bash
. ~/export-esp.sh
cargo build --release
```

Note: The project uses Rust edition 2024 and requires the ESP Rust toolchain (`channel = "esp"`).

### Toolchain
This project requires the ESP-specific Rust toolchain. The `rust-toolchain.toml` specifies `channel = "esp"`.

## Architecture

### Module Structure

- **`src/bin/main.rs`**: Entry point using embassy async runtime with esp-rtos scheduler, spawns IMU and display tasks
- **`src/display.rs`**: ST7789 display driver wrapper with 8-bit parallel interface and graphical rendering
- **`src/imu.rs`**: BMI160 sensor wrapper for I2C communication
- **`src/visualization.rs`**: Layout constants and coordinate mapping for sensor visualization
- **`src/config.rs`**: Display constants (170×320 resolution)

### Task Architecture

The application uses two Embassy tasks communicating via a channel:

```
┌──────────────┐         Channel          ┌──────────────┐
│  IMU Task    │ ──────────────────────> │ Display Task │
│              │    (ImuData)             │              │
│ - Reads BMI  │                          │ - Draws viz  │
│ - 100ms loop │                          │ - On demand  │
└──────────────┘                          └──────────────┘
```

- **IMU Task**: Polls BMI160 at 10Hz, logs to console, sends data via `embassy-sync` channel
- **Display Task**: Receives data and updates graphical visualization

### Hardware Pin Assignments (T-Display S3)

**Display (8-bit parallel):**
- GPIO5: Reset
- GPIO6: Chip Select
- GPIO7: Data/Command
- GPIO8: Write
- GPIO9: Read
- GPIO15: Power Enable (required for USB operation)
- GPIO38: Backlight
- GPIO39-42: Data lines D0-D3
- GPIO45-48: Data lines D4-D7

**I2C (BMI160 Sensor):**
- GPIO17: SDA
- GPIO18: SCL

**Available for future use:** GPIO1, GPIO2, GPIO4, GPIO16, GPIO21

### Key Dependencies

- **esp-hal 1.1.1**: Hardware abstraction layer for ESP32-S3
- **esp-rtos 0.3.0**: FreeRTOS integration with embassy
- **embassy-executor 0.10.0**: Async task executor
- **embassy-time 0.5.1**: Async timers and delays
- **embassy-sync 0.8.0**: Async channels and synchronization primitives
- **mipidsi 0.10.0**: MIPI display driver (ST7789)
- **embedded-graphics 0.8.2**: 2D graphics primitives (circles, rectangles, lines)
- **embedded-text 0.7.3**: Text rendering
- **embedded-hal 1.0.0**: Hardware abstraction traits
- **embedded-hal-bus 0.3.0**: Shared I2C bus via `CriticalSectionDevice`
- **bmi160 1.1.0**: IMU sensor driver
- **anyhow 1.0.102**: Error handling (no_std, `default-features = false`)

### Memory & Performance

- Heap allocation: 73744 bytes in `.dram2_uninit` section
- Optimization: Both dev and release profiles use `opt-level = "s"` (size optimization)
- Release profile uses thin LTO

### Important Constraints

1. **`#![no_std]`**: Standard library not available
2. **Async runtime**: Uses embassy-executor with esp-rtos scheduler
3. **Memory**: Limited to allocated heap in DRAM2
4. **Shared I2C**: Bus wrapped in `Mutex<RefCell<I2C>>` via `embedded-hal-bus` `CriticalSectionDevice`
5. **Peripheral ownership**: Each peripheral (I2C0, GPIO pins) can only be consumed once

### Common Patterns

**I2C setup:**
```rust
let i2c = I2c::new(peripherals.I2C0, I2cConfig::default().with_frequency(Rate::from_khz(400)))
    .unwrap()
    .with_sda(peripherals.GPIO17)
    .with_scl(peripherals.GPIO18);

static I2C_BUS: StaticCell<Mutex<RefCell<I2cBus>>> = StaticCell::new();
let i2c_bus = I2C_BUS.init(Mutex::new(RefCell::new(i2c)));
```

**Channel communication between tasks:**
```rust
static IMU_CHANNEL: StaticCell<Channel<CriticalSectionRawMutex, ImuData, 1>> = StaticCell::new();
let channel = IMU_CHANNEL.init(Channel::new());

// In IMU task
sender.send(data).await;

// In display task
let data = receiver.receive().await;
```

**Graphical visualization:**
```rust
let data = receiver.receive().await;
display.draw_sensor_visualization(&data)?;
```

## Visualization Details

### Tilt Indicator (Left Panel)
- Circular bubble level centered at (85, 85)
- Outer circle: 80px radius, inner circle: 60px radius
- Bubble: 12px radius, green when level (±12°), yellow when tilted
- Y-axis inverted: tilt forward → bubble moves up

### Gyroscope Bars (Right Panel)
- Three vertical bars for X, Y, Z rotation
- 140px height, centered at Y=90
- Blue fill: positive rotation (upward)
- Red fill: negative rotation (downward)

### Rendering Strategy
- **Partial updates**: Only redraws changed elements
- **Performance**: ~10-20ms per frame
- **Update rate**: 10Hz (driven by IMU task)
