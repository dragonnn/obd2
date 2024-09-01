use defmt::{debug, info, warn, Format};
use embedded_can::{Frame as _, StandardId};

use crate::{
    debug::internal_debug,
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

#[derive(Debug, Format, PartialEq, Clone)]
pub struct IceTemperaturePid {
    pub temperature: f64,
}

impl Pid for IceTemperaturePid {
    fn request() -> CanFrame {
        let can_id = StandardId::new(0x7e4).unwrap();
        CanFrame::new(can_id, &[0x02, 0x01, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap()
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 7 {
            return Err(Obd2Error::FrameToShort);
        }

        Ok(Self { temperature: (data[3] as i8 - 40) as f64 })
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::IceTemperaturePid(self)
    }
}
