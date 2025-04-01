use bitbybit::bitenum;
use defmt::*;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pull};
use embassy_time::{with_timeout, Duration, Instant};
use embedded_hal_async::i2c::I2c;
use modular_bitfield::prelude::*;

use super::destruct_twim::I2cBusReset;

const I2C_ADDRESS: u8 = 0x46;
const I2C_TIMEOUT: Duration = Duration::from_millis(100);

const REG_CHARGER_STATUS1: u8 = 0x08;
const REG_BAT_CAP: u8 = 0x20;
const REG_BAT_SOC: u8 = 0x21;
const REG_FUEL_GAUGE_MODE: u8 = 0x27;
const REG_BUCK_CONFIGURE: u8 = 0x29;
const REG_BUCK_VOLTAGE_CONFIGURE: u8 = 0x2A;
const REG_BUCK_BOOST_CONFIGURE: u8 = 0x2B;
const REG_BUCK_BOOST_VOLTAGE_CONFIGURE: u8 = 0x2C;
const REG_INTERRUPT_ENABLE1: u8 = 0x32;
const REG_INTERRUPT_ENABLE2: u8 = 0x33;
const REG_INTERRUPT_FLAG1: u8 = 0x34;
const REG_INTERRUPT_FLAG2: u8 = 0x35;
const REG_VBAT_READ: u8 = 0x25;
const REG_CHARGER_FUNCTION_SETTING: u8 = 0x07;

const FUEL_GAUGE_MODE_ENABLE_BIT: u8 = 0;
const FUEL_GAUGE_MODE_SLEEP_MODE_BIT: u8 = 1;

const INTERRUPT_ENABLE1_CHARGER_MODE: u8 = 1;
const INTERRUPT_ENABLE1_LOW_VOLTAGE: u8 = 7;

#[bitfield]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default)]
pub struct ChargerFunctionSettings {
    pub enable_charging: bool,
    pub enable_adaptive_current: bool,
    pub enable_end_of_charge: bool,
    pub enable_ldo: bool,
    pub off_isolation_fet: bool,
    #[skip]
    __: B1,
    pub cool_mode_current: bool,
    pub enable_jeita: bool,
}

impl defmt::Format for ChargerFunctionSettings {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "ChargerFunctionSettings {{ enable_charging: {}, enable_adaptive_current: {}, enable_end_of_charge: {}, enable_ldo: {}, off_isolation_fet: {}, cool_mode_current: {}, enable_jeita: {} }}",
            self.enable_charging(),
            self.enable_adaptive_current(),
            self.enable_end_of_charge(),
            self.enable_ldo(),
            self.off_isolation_fet(),
            self.cool_mode_current(),
            self.enable_jeita()
        );
    }
}

#[bitfield]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default)]
pub struct BuckSettings {
    pub buck_output: bool,
    pub output_discharge_function: bool,
    pub stop_feature: bool,
    pub fpwm_mode: bool,
    pub current_limit: B2,
    pub soft_start_time: B2,
}

impl defmt::Format for BuckSettings {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "BuckSettings {{ buck_output: {}, output_discharge_function: {}, stop_feature: {}, fpwm_mode: {}, current_limit: {}, soft_start_time: {} }}",
            self.buck_output(),
            self.output_discharge_function(),
            self.stop_feature(),
            self.fpwm_mode(),
            self.current_limit(),
            self.soft_start_time()
        );
    }
}

#[bitfield]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default)]
pub struct BuckVoltageSettings {
    pub voltage: B6,
    pub delay: B2,
}

impl defmt::Format for BuckVoltageSettings {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "BuckVoltageSettings {{ delay: {}, voltage: {} }}", self.delay(), self.voltage());
    }
}

#[bitfield]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default)]
pub struct BuckBoostSettings {
    pub buck_boost_output: bool,
    pub output_discharge_function: bool,
    pub stop_feature: bool,
    pub current_limit: B3,
    pub soft_start_time: B2,
}

impl defmt::Format for BuckBoostSettings {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "BuckBoostSettings {{ buck_boost_output: {}, output_discharge_function: {}, stop_feature: {}, current_limit: {}, soft_start_time: {} }}",
            self.buck_boost_output(),
            self.output_discharge_function(),
            self.stop_feature(),
            self.current_limit(),
            self.soft_start_time()
        );
    }
}

#[bitfield]
#[repr(u8)]
#[derive(Copy, Clone, Debug, Default)]
pub struct BuckBoostVoltageSettings {
    pub voltage: B6,
    pub delay: B2,
}

impl defmt::Format for BuckBoostVoltageSettings {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "BuckBoostVoltageSettings {{ delay: {}, voltage: {} }}", self.delay(), self.voltage());
    }
}

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
    last_soc_update: Instant,
    last_charger_status: ChargerStatus,
}

impl<I2C> Adp5360<I2C>
where
    I2C: I2c + I2cBusReset,
{
    pub async fn new(i2c: I2C, irq: AnyPin) -> Self {
        let irq = Input::new(irq, Pull::Up);
        defmt::info!("adp5360 reset done");
        let mut adp5360 = Self {
            i2c,
            irq,
            last_voltage: 0,
            last_soc: 0,
            last_soc_update: Instant::now(),
            last_charger_status: ChargerStatus::Off,
        };

        adp5360.set_u8_value(REG_BAT_CAP, 0xFF).await;
        adp5360.set_u8_bit(REG_FUEL_GAUGE_MODE, FUEL_GAUGE_MODE_ENABLE_BIT, true).await;
        adp5360.set_u8_bit(REG_FUEL_GAUGE_MODE, FUEL_GAUGE_MODE_SLEEP_MODE_BIT, true).await;
        adp5360.set_u8_bit(REG_INTERRUPT_ENABLE1, INTERRUPT_ENABLE1_CHARGER_MODE, true).await;
        adp5360.set_u8_bit(REG_INTERRUPT_ENABLE1, INTERRUPT_ENABLE1_LOW_VOLTAGE, true).await;

        let mut interput_reason = [0u8; 2];
        adp5360.get_u8_values(REG_INTERRUPT_FLAG1, &mut interput_reason).await;

        let buck_settings = BuckSettings::default()
            .with_buck_output(true)
            .with_output_discharge_function(true)
            .with_current_limit(3)
            .with_soft_start_time(0);

        adp5360.set_u8_value(REG_BUCK_CONFIGURE, buck_settings.into()).await;

        let buck_voltage_settings = BuckVoltageSettings::default().with_delay(0).with_voltage(24);
        adp5360.set_u8_value(REG_BUCK_VOLTAGE_CONFIGURE, buck_voltage_settings.into()).await;

        let buck_boost_settings = BuckBoostSettings::default()
            .with_buck_boost_output(true)
            .with_output_discharge_function(false)
            .with_current_limit(3)
            .with_soft_start_time(0);

        adp5360.set_u8_value(REG_BUCK_BOOST_CONFIGURE, buck_boost_settings.into()).await;

        let buck_boost_voltage_settings = BuckBoostVoltageSettings::default().with_delay(0).with_voltage(19);
        adp5360.set_u8_value(REG_BUCK_BOOST_VOLTAGE_CONFIGURE, buck_boost_voltage_settings.into()).await;

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
        info!("battery irq");
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
                self.last_soc_update = Instant::now();
                value
            }
            Err(_) => {
                if self.last_soc_update.elapsed().as_secs() > 5 * 60 {
                    core::panic!("Battery SOC read timeout");
                }
                self.last_soc
            }
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

    async fn charger_settings(&mut self) -> Result<ChargerFunctionSettings, ()> {
        self.get_u8_value(REG_CHARGER_FUNCTION_SETTING).await.map(|value| ChargerFunctionSettings::from_bytes([value]))
    }

    async fn buck_settings(&mut self) -> Result<BuckSettings, ()> {
        let ret = self.get_u8_value(REG_BUCK_CONFIGURE).await.map(|value| BuckSettings::from_bytes([value]));
        info!("buck settings: {:?}", ret);
        ret
    }

    async fn buck_voltage_settings(&mut self) -> Result<BuckVoltageSettings, ()> {
        let ret =
            self.get_u8_value(REG_BUCK_VOLTAGE_CONFIGURE).await.map(|value| BuckVoltageSettings::from_bytes([value]));
        info!("buck voltage settings: {:?}", ret);
        ret
    }

    async fn buck_boost_settings(&mut self) -> Result<BuckBoostSettings, ()> {
        let ret = self.get_u8_value(REG_BUCK_BOOST_CONFIGURE).await.map(|value| BuckBoostSettings::from_bytes([value]));
        info!("buck boost settings: {:?}", ret);
        ret
    }

    async fn buck_boost_voltage_settings(&mut self) -> Result<BuckBoostVoltageSettings, ()> {
        let ret = self
            .get_u8_value(REG_BUCK_BOOST_VOLTAGE_CONFIGURE)
            .await
            .map(|value| BuckBoostVoltageSettings::from_bytes([value]));
        info!("buck boost voltage settings: {:?}", ret);
        ret
    }

    pub async fn disable_charging(&mut self) {
        if let Ok(mut settings) = self.charger_settings().await {
            info!("disable charging on: {:?}", settings);
            settings.set_enable_charging(false);
            self.set_u8_value(REG_CHARGER_FUNCTION_SETTING, settings.into_bytes()[0]).await;
        } else {
            error!("disable charging failed");
        }
    }

    pub async fn enable_charging(&mut self) {
        if let Ok(mut settings) = self.charger_settings().await {
            info!("enable charging on: {:?}", settings);
            settings.set_enable_charging(true);
            self.set_u8_value(REG_CHARGER_FUNCTION_SETTING, settings.into_bytes()[0]).await;
        } else {
            error!("enable charging failed");
        }
    }
}
