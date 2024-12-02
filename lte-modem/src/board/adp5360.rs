use bitbybit::bitenum;
use defmt::Format;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pull};
use embassy_time::{with_timeout, Duration};
use embedded_hal_async::i2c::I2c;

use super::destruct_twim::I2cBusReset;

const I2C_ADDRESS: u8 = 0x46;
const I2C_TIMEOUT: Duration = Duration::from_millis(100);

const REG_CHARGER_STATUS1: u8 = 0x08;
const REG_BAT_CAP: u8 = 0x20;
const REG_BAT_SOC: u8 = 0x21;
const REG_FUEL_GAUGE_MODE: u8 = 0x27;
const REG_INTERRUPT_ENABLE1: u8 = 0x32;
const REG_INTERRUPT_ENABLE2: u8 = 0x33;
const REG_INTERRUPT_FLAG1: u8 = 0x34;
const REG_INTERRUPT_FLAG2: u8 = 0x35;
const REG_VBAT_READ: u8 = 0x25;

const FUEL_GAUGE_MODE_ENABLE_BIT: u8 = 0;
const FUEL_GAUGE_MODE_SLEEP_MODE_BIT: u8 = 1;

const INTERRUPT_ENABLE1_CHARGER_MODE: u8 = 1;
const INTERRUPT_ENABLE1_LOW_VOLTAGE: u8 = 7;

#[derive(Format)]
#[bitenum(u8, exhaustive: false)]
pub enum InterputEvent {
    LowVoltage = 0b1000000,
    ChargeOverflow = 0b0100000,
    InputCurrentLimit = 0b0010000,
    BatteryFault = 0b0001000,
    TemperatureTreshold = 0b0000100,
    ChargeModeChange = 0b0000010,
    VoltageThreashold = 0b0000001,
}

#[derive(Format, PartialEq, Eq, Default)]
#[bitenum(u8, exhaustive: false)]
pub enum ChargerStatus {
    #[default]
    Off = 0b000,
    Trickle = 0b001,
    FastChargeCC = 0b010,
    FastChargeCV = 0b011,
    Complete = 0b100,
    LDOMode = 0b101,
    TrickleOrFastChargeTimeout = 0b110,
    BatteryDetection = 0b111,
}

impl ChargerStatus {
    pub fn is_charging(&self) -> bool {
        *self != ChargerStatus::Off
    }
}

pub struct Adp5360<I2C> {
    i2c: I2C,
    irq: Input<'static>,

    last_voltage: u16,
    last_soc: u8,
    last_charger_status: ChargerStatus,
}

impl<I2C> Adp5360<I2C>
where
    I2C: I2c + I2cBusReset,
{
    pub async fn new(i2c: I2C, irq: AnyPin) -> Self {
        let irq = Input::new(irq, Pull::Up);
        defmt::info!("adp5360 reset done");
        let mut adp5360 = Self { i2c, irq, last_voltage: 0, last_soc: 0, last_charger_status: ChargerStatus::Off };
        adp5360.set_u8_value(REG_BAT_CAP, 0xFF).await;
        adp5360.set_u8_bit(REG_FUEL_GAUGE_MODE, FUEL_GAUGE_MODE_ENABLE_BIT, true).await;
        adp5360.set_u8_bit(REG_FUEL_GAUGE_MODE, FUEL_GAUGE_MODE_SLEEP_MODE_BIT, true).await;
        adp5360.set_u8_bit(REG_INTERRUPT_ENABLE1, INTERRUPT_ENABLE1_CHARGER_MODE, true).await;
        adp5360.set_u8_bit(REG_INTERRUPT_ENABLE1, INTERRUPT_ENABLE1_LOW_VOLTAGE, true).await;

        let mut interput_reason = [0u8; 2];
        adp5360.get_u8_values(REG_INTERRUPT_FLAG1, &mut interput_reason).await;

        adp5360
    }

    async fn reset(&mut self) {
        self.i2c.reset().await;
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
                self.reset().await
            }
            Err(_) => {
                defmt::error!("i2c timeout");
                self.reset().await
            }
        }
    }

    async fn get_u8_value(&mut self, reg: u8) -> Result<u8, ()> {
        let mut buf = [0u8; 1];
        match with_timeout(I2C_TIMEOUT, self.i2c.write_read(I2C_ADDRESS, &[reg], &mut buf)).await {
            Ok(Ok(_)) => Ok(buf[0]),
            Ok(Err(err)) => {
                defmt::error!("i2c error: {:?}", defmt::Debug2Format(&err));
                self.reset().await;
                Err(())
            }
            Err(_) => {
                defmt::error!("i2c timeout");
                self.reset().await;
                Err(())
            }
        }
    }

    async fn get_u8_values(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), ()> {
        match with_timeout(I2C_TIMEOUT, self.i2c.write_read(I2C_ADDRESS, &[reg], buf)).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(err)) => {
                defmt::error!("i2c error: {:?}", defmt::Debug2Format(&err));
                self.reset().await;
                Err(())
            }
            Err(_) => {
                defmt::error!("i2c timeout");
                self.reset().await;
                Err(())
            }
        }
    }

    pub async fn irq(&mut self) -> Result<InterputEvent, u8> {
        self.irq.wait_for_any_edge().await;
        embassy_time::Timer::after(Duration::from_secs(1)).await;
        let mut interput_reason = [0u8; 2];
        self.get_u8_values(REG_INTERRUPT_FLAG1, &mut interput_reason).await.ok();

        InterputEvent::new_with_raw_value(interput_reason[0])
    }

    pub async fn voltage(&mut self) -> u16 {
        let mut buf = [0u8; 2];
        if self.get_u8_values(REG_VBAT_READ, &mut buf).await.is_err() {
            return self.last_voltage;
        }
        let new_voltage = (buf[0] as u16) << 5 | (buf[1] as u16) >> 3;
        self.last_voltage = new_voltage;
        new_voltage
    }

    pub async fn battery_soc(&mut self) -> u8 {
        match self.get_u8_value(REG_BAT_SOC).await {
            Ok(value) => {
                self.last_soc = value;
                value
            }
            Err(_) => self.last_soc,
        }
    }

    pub async fn charger_status(&mut self) -> Result<ChargerStatus, u8> {
        match self.get_u8_value(REG_CHARGER_STATUS1).await {
            Ok(value) => {
                self.last_charger_status = ChargerStatus::new_with_raw_value(value & 0b111)?;
                Ok(self.last_charger_status)
            }
            Err(_) => Ok(self.last_charger_status),
        }
    }
}
