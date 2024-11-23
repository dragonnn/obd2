use defmt::*;
use embedded_can::{Frame as _, StandardId};
pub use types::Icu3Pid;

use crate::{
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

impl Pid for Icu3Pid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x770));
        unwrap!(CanFrame::new(can_id, &[0x03, 0x22, 0xbc, 0x10, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 12 {
            return Err(Obd2Error::FrameToShort);
        }

        let on_board_charger_wakeup_output = data[8] & 0b00010000 != 0;
        let ret = Self { on_board_charger_wakeup_output };
        Ok(ret)
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::Icu3Pid(self)
    }

    fn period() -> Option<embassy_time::Duration> {
        Some(embassy_time::Duration::from_secs(1))
    }
}
