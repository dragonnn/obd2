use defmt::*;
use embedded_can::{Frame as _, StandardId};
pub use types::IcuPid;

use crate::{
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

impl Pid for IcuPid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x770));
        //unwrap!(CanFrame::new(can_id, &[0x03, 0x22, 0xbc, 0x03, 0x00, 0x00, 0x00, 0x00]))
        unwrap!(CanFrame::new(can_id, &[0x03, 0x22, 0x00, 0x0b, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 18 {
            return Err(Obd2Error::FrameToShort);
        }

        let bat_discharge_warning_first_event_milage =
            data[7] as f64 * 6553.6 + data[8] as f64 * 25.55 + data[9] as f64 * 0.1;
        let bat_discharge_warning_first_event_soc = data[10];
        let bat_discharge_warning_final_event_milage =
            data[11] as f64 * 6553.6 + data[12] as f64 * 25.55 + data[13] as f64 * 0.1;
        let bat_discharge_warning_final_event_soc = data[14];
        let ret = Self {
            bat_discharge_warning_first_event_milage,
            bat_discharge_warning_first_event_soc,
            bat_discharge_warning_final_event_milage,
            bat_discharge_warning_final_event_soc,
        };
        info!("IcuPid: {:?}", ret);
        Ok(ret)
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::IcuPid(self)
    }

    fn period() -> Option<embassy_time::Duration> {
        Some(embassy_time::Duration::from_secs(1))
    }
}
