use defmt::{debug, info, warn, Format};
use embedded_can::{Frame as _, StandardId};

use crate::{
    debug::internal_debug,
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

#[derive(Debug, Format, PartialEq, Clone)]
pub struct GearboxGearPid {
    pub gear: i32,
}

impl Pid for GearboxGearPid {
    fn request() -> CanFrame {
        //let can_id = StandardId::new(0x7df).unwrap();
        //CanFrame::new(can_id, &[0x02, 0x01, 0xA4, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap()
        let can_id = StandardId::new(0x7e2).unwrap();
        CanFrame::new(can_id, &[0x02, 0x21, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap()
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 7 {
            return Err(Obd2Error::FrameToShort);
        }
        //3 - A
        //4 - B
        //5 - C
        //6 - D
        Ok(Self { gear: data[7] as i32 })
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::GearboxGearPid(self)
    }
}
