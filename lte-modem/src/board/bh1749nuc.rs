use bitfield_struct::bitfield;
use embassy_nrf::gpio::{AnyPin, Input, Pull};
use embassy_time::{with_timeout, Duration, Timer};
use embedded_hal_async::i2c::I2c;

use super::destruct_twim::I2cBusReset;

const I2C_ADDRESS: u8 = 0x38;
const I2C_TIMEOUT: Duration = Duration::from_millis(100);

const VALUE_PARTID: u8 = 0x0D;
const VALUE_MANUFACTURER_ID: u8 = 0xe0;

const REG_SYSTEM_CONTROL: u8 = 0x40;
const REG_MANUFACTURER_ID: u8 = 0x92;
const REG_MODE_CONTROL1: u8 = 0x41;
const REG_MODE_CONTROL2: u8 = 0x42;
const REG_INTERRUPT: u8 = 0x60;
const REG_PERSISTENCE: u8 = 0x61;

const REG_RED_DATA: u8 = 0x50;
const REG_GREEN_DATA: u8 = 0x52;
const REG_BLUE_DATA: u8 = 0x54;

#[bitfield(u8)]
pub struct SystemControl {
    #[bits(6)]
    part_id: u8,
    int_reset: bool,
    sw_reset: bool,
}

#[bitfield(u8)]
pub struct ModeControl1 {
    #[bits(3)]
    measurement_mode: u8,
    #[bits(2)]
    rgb_gain: u8,
    #[bits(2)]
    ir_gain: u8,
    _zero: bool,
}

#[bitfield(u8)]
pub struct Interrupt {
    int_enable: bool,
    _zero: bool,
    #[bits(2)]
    int_source: u8,
    #[bits(3)]
    _zero2: u8,
    int_status: bool,
}

#[repr(u8)]
pub enum MeasurementMode {
    Ms120 = 0b010,
    Ms240 = 0b011,
    Ms035 = 0b101,
}

#[repr(u8)]
pub enum Gain {
    X1 = 0b01,
    X32 = 0b11,
}

#[bitfield(u8)]
pub struct ModeControl2 {
    #[bits(4)]
    _zero: u8,
    rgb_en: bool,
    #[bits(2)]
    _zero2: u8,
    valid: bool,
}

pub struct Bh1749nuc<I2C> {
    i2c: I2C,
    irq: Input<'static>,
    enabled: bool,
}

impl<I2C> Bh1749nuc<I2C>
where
    I2C: I2c + I2cBusReset,
{
    pub async fn new(i2c: I2C, irq: AnyPin) -> Self {
        defmt::info!("init");
        let irq = Input::new(irq, Pull::Up);

        let mut bh1749nuc = Self { i2c, irq, enabled: false };

        for i in 0..120 {
            if bh1749nuc.inner_init().await.is_ok() {
                break;
            }
        }

        bh1749nuc
    }

    async fn inner_init(&mut self) -> Result<(), ()> {
        let mut system_control = SystemControl::from(self.get_u8_value(REG_SYSTEM_CONTROL).await?);
        let manufacture_id = self.get_u8_value(REG_MANUFACTURER_ID).await?;

        if system_control.part_id() != VALUE_PARTID {
            return Err(());
        }

        if manufacture_id != VALUE_MANUFACTURER_ID {
            return Err(());
        }
        system_control.set_sw_reset(true);

        self.set_u8_value(REG_SYSTEM_CONTROL, system_control.into()).await;

        Ok(())
    }

    async fn set_u8_bit(&mut self, reg: u8, bit: u8, value: bool) {
        if let Ok(mut reg_value) = self.get_u8_value(reg).await {
            if value {
                reg_value |= 1 << bit;
            } else {
                reg_value &= !(1 << bit);
            }
            self.set_u8_value(reg, reg_value).await;
        }
    }

    async fn set_u8_value(&mut self, reg: u8, value: u8) {
        match with_timeout(I2C_TIMEOUT, self.i2c.write(I2C_ADDRESS, &[reg, value])).await {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => {
                defmt::error!("i2c error: {:?}", defmt::Debug2Format(&err));
            }
            Err(_) => {
                defmt::error!("i2c timeout");
            }
        }
    }

    async fn get_u8_value(&mut self, reg: u8) -> Result<u8, ()> {
        let mut buf = [0u8; 1];
        match with_timeout(I2C_TIMEOUT, self.i2c.write_read(I2C_ADDRESS, &[reg], &mut buf)).await {
            Ok(Ok(_)) => Ok(buf[0]),
            Ok(Err(err)) => {
                defmt::error!("i2c error: {:?}", defmt::Debug2Format(&err));
                self.i2c.reset().await;
                Err(())
            }
            Err(_) => {
                defmt::error!("i2c timeout");
                self.i2c.reset().await;
                Err(())
            }
        }
    }

    async fn get_u8_values(&mut self, reg: u8, buf: &mut [u8]) {
        match with_timeout(I2C_TIMEOUT, self.i2c.write_read(I2C_ADDRESS, &[reg], buf)).await {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => {
                defmt::error!("i2c error: {:?}", defmt::Debug2Format(&err));
                self.i2c.reset().await;
            }
            Err(_) => {
                defmt::error!("i2c timeout");
                self.i2c.reset().await;
            }
        }
    }

    pub async fn enable(&mut self) {
        let mut mode_control_1 = ModeControl1::new();
        mode_control_1.set_measurement_mode(MeasurementMode::Ms240 as u8);
        mode_control_1.set_rgb_gain(Gain::X1 as u8);
        mode_control_1.set_ir_gain(Gain::X1 as u8);
        self.set_u8_value(REG_MODE_CONTROL1, mode_control_1.into()).await;

        let mut mode_control_2 = ModeControl2::from(self.get_u8_value(REG_MODE_CONTROL2).await.unwrap_or_default());
        mode_control_2.set_rgb_en(true);
        self.set_u8_value(REG_MODE_CONTROL2, mode_control_2.into()).await;
        Timer::after(Duration::from_millis(240)).await;

        self.enabled = true;
    }

    pub async fn shutdown(&mut self) {
        if self.enabled {
            let mut system_control =
                SystemControl::from(self.get_u8_value(REG_SYSTEM_CONTROL).await.unwrap_or_default());

            system_control.set_sw_reset(true);

            self.set_u8_value(REG_SYSTEM_CONTROL, system_control.into()).await;
            self.enabled = false;
        }
    }

    pub async fn irq(&mut self) {
        self.irq.wait_for_any_edge().await;
    }

    pub async fn r(&mut self) -> u16 {
        if !self.enabled {
            self.enable().await;
        }

        let mut buf = [0; 2];
        self.get_u8_values(REG_RED_DATA, &mut buf).await;
        u16::from_le_bytes(buf)
    }

    pub async fn g(&mut self) -> u16 {
        if !self.enabled {
            self.enable().await;
        }

        let mut buf = [0; 2];
        self.get_u8_values(REG_GREEN_DATA, &mut buf).await;
        u16::from_le_bytes(buf)
    }

    pub async fn b(&mut self) -> u16 {
        if !self.enabled {
            self.enable().await;
        }

        let mut buf = [0; 2];
        self.get_u8_values(REG_BLUE_DATA, &mut buf).await;
        u16::from_le_bytes(buf)
    }

    pub async fn w(&mut self) -> u16 {
        let r = self.r().await;
        let g = self.g().await;
        let b = self.b().await;
        ((r as u32 + g as u32 + b as u32) as f64 / 3.0) as u16
    }
}
