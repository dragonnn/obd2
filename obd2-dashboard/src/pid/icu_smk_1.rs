use defmt::*;
use embedded_can::{Frame as _, StandardId};
pub use types::Icu1Smk;

use crate::{
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

impl Pid for Icu1Smk {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x7a0));
        unwrap!(CanFrame::new(can_id, &[0x03, 0x22, 0xd0, 0x06, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 26 {
            return Err(Obd2Error::FrameToShort);
        }

        fn u8_to_voltage(v: u8) -> f32 {
            v as f32 * 0.08
        }

        let aux_battery_voltage_power_load = u8_to_voltage(data[7]);
        let aux_battery_voltage_signal_cpu = u8_to_voltage(data[8]);
        let ground_voltage_power = u8_to_voltage(data[9]);
        let ground_voltage_ecu = u8_to_voltage(data[10]);
        let ign1_voltage = u8_to_voltage(data[11]);
        let ign2_voltage = u8_to_voltage(data[12]);
        let acc_voltage = u8_to_voltage(data[13]);

        let engine_rpm = data[15] as u16 * 32;
        let vehicle_speed = data[16];

        let ret = Self {
            aux_battery_voltage_power_load,
            aux_battery_voltage_signal_cpu,
            ground_voltage_power,
            ground_voltage_ecu,
            ign1_voltage,
            ign2_voltage,
            acc_voltage,
            engine_rpm,
            vehicle_speed,
        };

        //info!("{:?}", ret);
        Ok(ret)
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::Icu1Smk(self)
    }

    fn period() -> Option<embassy_time::Duration> {
        Some(embassy_time::Duration::from_secs(1))
    }
}
