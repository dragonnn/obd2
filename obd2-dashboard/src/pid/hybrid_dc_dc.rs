use defmt::*;
use embedded_can::{Frame as _, StandardId};
pub use types::HybridDcDcPid;

use crate::{
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

impl Pid for HybridDcDcPid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x7e2));
        unwrap!(CanFrame::new(can_id, &[0x02, 0x21, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 7 {
            return Err(Obd2Error::FrameToShort);
        }
        Ok(Self { gear: data[2] as i32 })
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::HybridDcDcPid(self)
    }
}