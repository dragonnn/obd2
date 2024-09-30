use defmt::{debug, info, unwrap, warn, Format};
use embedded_can::{Frame as _, StandardId};

use crate::{
    debug::internal_debug,
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

#[derive(Debug, Format, PartialEq, Clone, Copy, strum::IntoStaticStr)]
pub enum Gear {
    PN,
    R,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    U,
}

#[derive(Debug, Format, PartialEq, Clone)]
pub struct TransaxlePid {
    pub gear: Gear,
}

impl Pid for TransaxlePid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x7e1));
        unwrap!(CanFrame::new(can_id, &[0x03, 0x22, 0x01, 0xa4, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 7 {
            return Err(Obd2Error::FrameToShort);
        }

        let gear = match data[16] {
            0x04 => Gear::PN,
            0x69 => Gear::R,
            0x68 => Gear::D1,
            0x39 => Gear::D2,
            0x24 => Gear::D3,
            0x16 => Gear::D4,
            0x12 => Gear::D5,
            0x09 => Gear::D6,
            _ => Gear::U,
        };

        Ok(Self { gear })
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::TransaxlePid(self)
    }
}
