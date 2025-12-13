# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a `no_std` embedded Rust application for the **LilyGo T-Display S3** (ESP32-S3) that reads data from a BMI160 accelerometer/gyroscope sensor and displays it on the onboard ST7789 LCD screen.

**Target Hardware**: LilyGo T-Display S3
- ESP32-S3R8 with 16MB Flash, 8MB PSRAM
- 170×320 ST7789 LCD (1.9")
- 8-bit parallel display interface

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

- **`src/bin/main.rs`**: Entry point using embassy async runtime with esp-rtos scheduler
- **`src/display.rs`**: ST7789 display driver wrapper with 8-bit parallel interface
- **`src/sensor.rs`**: BMI160 sensor wrapper for I2C communication
- **`src/config.rs`**: Display constants (170×320 resolution)

### Hardware Pin Assignments (T-Display S3)

**Display (8-bit parallel):**
- GPIO5: Reset
- GPIO6: Chip Select
- GPIO7: Data/Command
- GPIO8: Write
- GPIO9: Read
- GPIO38: Backlight
- GPIO39-42: Data lines D0-D3
- GPIO45-48: Data lines D4-D7

**I2C (BMI160 Sensor):**
- GPIO17: SDA
- GPIO18: SCL

**Available for future use:** GPIO1, GPIO2, GPIO4, GPIO15, GPIO16, GPIO21

### Key Dependencies

- **esp-hal 1.0.0**: Hardware abstraction layer for ESP32-S3
- **esp-rtos 0.2.0**: FreeRTOS integration with embassy
- **embassy**: Async runtime (executor + time)
- **mipidsi 0.9.0**: MIPI display driver (ST7789)
- **embedded-graphics**: Drawing primitives
- **embedded-text**: Text rendering
- **bmi160 1.1.0**: IMU sensor driver

### Memory & Performance

- Heap allocation: 73744 bytes in `.dram2_uninit` section
- Optimization: Both dev and release profiles use `opt-level = "s"` (size optimization)
- Release profile uses fat LTO for maximum size reduction

### Important Constraints

1. **`#![no_std]`**: Standard library not available
2. **Async runtime**: Uses embassy-executor with esp-rtos scheduler
3. **Memory**: Limited to allocated heap in DRAM2
4. **Delay providers**: Uses SystemTimer targets for non-blocking delays (multiple targets needed if sharing peripherals)
5. **Peripheral ownership**: Each peripheral (I2C0, SYSTIMER, GPIO pins) can only be consumed once

### Common Patterns

**Initializing peripherals with delays:**
```rust
let systimer = SystemTimer::new(peripherals.SYSTIMER);
let delay = systimer.target(Target::Timer0);
```

**I2C setup for sensors:**
```rust
let io = esp_hal::gpio::Io::new(peripherals.GPIO_PINS, peripherals.IO_MUX);
let i2c = I2c::new(
    peripherals.I2C0,
    io.pins.gpio17,  // SDA
    io.pins.gpio18,  // SCL
    I2cConfig::default().with_frequency(400.kHz()),
);
```

**Display update loop:**
```rust
display.write_multiline(&text)?;  // Clears display and renders text
```
