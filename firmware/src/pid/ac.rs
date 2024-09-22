use defmt::{debug, info, unwrap, warn, Format};
use embedded_can::{Frame as _, StandardId};

use crate::{
    debug::internal_debug,
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

#[derive(Debug, Format, PartialEq, Clone)]
pub struct AcPid {
    pub gear: i32,
}

impl Pid for AcPid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x7b3));
        unwrap!(CanFrame::new(can_id, &[0x02, 0x22, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 7 {
            return Err(Obd2Error::FrameToShort);
        }
        //3 - A
        //4 - B
        //5 - C
        //6 - D
        info!("gearbox gear: {=[u8]:#04x}", data);
        Ok(Self { gear: data[2] as i32 })
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::AcPid(self)
    }
}
