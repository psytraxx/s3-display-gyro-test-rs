use anyhow::{Result, anyhow};
use core::cell::RefCell;
use critical_section::Mutex;
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;
use embedded_hal_bus::i2c::CriticalSectionDevice;

const SEESAW_ADDR: u8 = 0x36;
const MODULE_TOUCH: u8 = 0x0F;
const FUNC_TOUCH_CHANNEL_OFFSET: u8 = 0x10;
const MODULE_STATUS: u8 = 0x00;
const FUNC_STATUS_TEMP: u8 = 0x04;

pub struct MoistureSensor<I2C>
where
    I2C: I2c + 'static,
{
    dev: CriticalSectionDevice<'static, I2C>,
}

#[derive(Debug, Clone, Copy)]
pub struct MoistureData {
    pub capacitance: u16,
    pub temperature_c: f32,
}

impl<I2C> MoistureSensor<I2C>
where
    I2C: I2c + 'static,
    I2C::Error: core::fmt::Debug,
{
    pub fn new(bus: &'static Mutex<RefCell<I2C>>) -> Self {
        Self {
            dev: CriticalSectionDevice::new(bus),
        }
    }

    fn read_register(
        &mut self,
        module: u8,
        func: u8,
        buf: &mut [u8],
        delay: &mut impl DelayNs,
    ) -> Result<()> {
        self.dev
            .write(SEESAW_ADDR, &[module, func])
            .map_err(|e| anyhow!("seesaw: write register: {:?}", e))?;
        delay.delay_ms(5);
        self.dev
            .read(SEESAW_ADDR, buf)
            .map_err(|e| anyhow!("seesaw: read register: {:?}", e))?;
        Ok(())
    }

    pub fn read(&mut self, delay: &mut impl DelayNs) -> Result<MoistureData> {
        let mut cap_buf = [0u8; 2];
        self.read_register(MODULE_TOUCH, FUNC_TOUCH_CHANNEL_OFFSET, &mut cap_buf, delay)?;
        let capacitance = u16::from_be_bytes(cap_buf);

        let mut temp_buf = [0u8; 4];
        self.read_register(MODULE_STATUS, FUNC_STATUS_TEMP, &mut temp_buf, delay)?;
        let raw_temp = i32::from_be_bytes(temp_buf);
        let temperature_c = raw_temp as f32 / 65536.0;

        Ok(MoistureData {
            capacitance,
            temperature_c,
        })
    }
}
