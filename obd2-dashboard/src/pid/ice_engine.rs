use defmt::*;
use embedded_can::{Frame as _, StandardId};
pub use types::IceEnginePid;

use crate::{
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

impl Pid for IceEnginePid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x7e0));
        unwrap!(CanFrame::new(can_id, &[0x03, 0x22, 0xe0, 0x01, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 7 {
            return Err(Obd2Error::FrameToShort);
        }
        Ok(Self { gear: data[2] as i32 })
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::IceEnginePid(self)
    }
}
