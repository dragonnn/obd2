use defmt::{debug, info, unwrap, warn, Format};
use embedded_can::{Frame as _, StandardId};

use crate::{
    debug::internal_debug,
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

#[derive(Debug, Format, PartialEq, Clone)]
pub struct IcuPid {
    pub gear: i32,
}

impl Pid for IcuPid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x770));
        unwrap!(CanFrame::new(can_id, &[0x03, 0x22, 0xbc, 0x03, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 7 {
            return Err(Obd2Error::FrameToShort);
        }
        Ok(Self { gear: data[2] as i32 })
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::Icu(self)
    }

    fn period() -> Option<embassy_time::Duration> {
        Some(embassy_time::Duration::from_secs(1))
    }
}
