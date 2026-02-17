use bmi160::{AccelerometerPowerMode, Bmi160, GyroscopePowerMode, SensorSelector, SlaveAddr};
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

pub struct Sensor<I2C, D> {
    bmi: Bmi160<bmi160::interface::I2cInterface<I2C>>,
    #[allow(dead_code)]
    delay: D,
}

#[derive(Debug, Clone, Copy)]
pub struct SensorData {
    pub accel_x: i16,
    pub accel_y: i16,
    pub accel_z: i16,
    pub gyro_x: i16,
    pub gyro_y: i16,
    pub gyro_z: i16,
}

impl<I2C, D> Sensor<I2C, D>
where
    I2C: I2c,
    D: DelayNs,
{
    pub fn new(i2c: I2C, mut delay: D) -> Result<Self, Error<I2C::Error>> {
        let mut bmi = Bmi160::new_with_i2c(i2c, SlaveAddr::default());

        delay.delay_ms(10);

        bmi.set_accel_power_mode(AccelerometerPowerMode::Normal)
            .map_err(Error::Sensor)?;
        delay.delay_ms(10);

        bmi.set_gyro_power_mode(GyroscopePowerMode::Normal)
            .map_err(Error::Sensor)?;
        delay.delay_ms(10);

        Ok(Self { bmi, delay })
    }

    pub fn read(&mut self) -> Result<SensorData, Error<I2C::Error>> {
        let data = self
            .bmi
            .data(SensorSelector::new().accel().gyro())
            .map_err(Error::Sensor)?;

        let accel = data.accel.ok_or(Error::NoData)?;
        let gyro = data.gyro.ok_or(Error::NoData)?;

        Ok(SensorData {
            accel_x: accel.x,
            accel_y: accel.y,
            accel_z: accel.z,
            gyro_x: gyro.x,
            gyro_y: gyro.y,
            gyro_z: gyro.z,
        })
    }
}

#[derive(Debug)]
pub enum Error<E> {
    Sensor(bmi160::Error<E>),
    NoData,
}

impl<E: core::fmt::Debug> core::fmt::Display for Error<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Sensor(e) => write!(f, "BMI160 sensor error: {:?}", e),
            Error::NoData => write!(f, "No sensor data available"),
        }
    }
}
