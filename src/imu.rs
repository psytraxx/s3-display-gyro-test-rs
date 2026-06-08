use anyhow::{anyhow, Result};
use bmi160::{AccelerometerPowerMode, Bmi160, GyroscopePowerMode, SensorSelector, SlaveAddr};
use core::cell::RefCell;
use critical_section::Mutex;
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;
use embedded_hal_bus::i2c::CriticalSectionDevice;

pub struct Imu<I2C>
where
    I2C: I2c + 'static,
{
    bmi: Bmi160<bmi160::interface::I2cInterface<CriticalSectionDevice<'static, I2C>>>,
}

#[derive(Debug, Clone, Copy)]
pub struct ImuData {
    pub accel_x: i16,
    pub accel_y: i16,
    pub accel_z: i16,
    pub gyro_x: i16,
    pub gyro_y: i16,
    pub gyro_z: i16,
}

impl<I2C> Imu<I2C>
where
    I2C: I2c + 'static,
    I2C::Error: core::fmt::Debug,
{
    pub fn new(bus: &'static Mutex<RefCell<I2C>>, delay: &mut impl DelayNs) -> Result<Self> {
        let dev = CriticalSectionDevice::new(bus);
        let mut bmi = Bmi160::new_with_i2c(dev, SlaveAddr::default());
        delay.delay_ms(10);
        bmi.set_accel_power_mode(AccelerometerPowerMode::Normal)
            .map_err(|e| anyhow!("BMI160: set accel power mode: {:?}", e))?;
        delay.delay_ms(10);
        bmi.set_gyro_power_mode(GyroscopePowerMode::Normal)
            .map_err(|e| anyhow!("BMI160: set gyro power mode: {:?}", e))?;
        delay.delay_ms(10);
        Ok(Self { bmi })
    }

    pub fn read(&mut self) -> Result<ImuData> {
        let data = self
            .bmi
            .data(SensorSelector::new().accel().gyro())
            .map_err(|e| anyhow!("BMI160: read data: {:?}", e))?;
        let accel = data.accel.ok_or_else(|| anyhow!("BMI160: no accel data"))?;
        let gyro = data.gyro.ok_or_else(|| anyhow!("BMI160: no gyro data"))?;
        Ok(ImuData {
            accel_x: accel.x,
            accel_y: accel.y,
            accel_z: accel.z,
            gyro_x: gyro.x,
            gyro_y: gyro.y,
            gyro_z: gyro.z,
        })
    }
}
