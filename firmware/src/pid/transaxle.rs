use defmt::{debug, error, info, unwrap, warn, Format};
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

        let gear_byte = data[16];

        let gear = match gear_byte {
            04 => Gear::PN,
            69 => Gear::R,
            68 => Gear::D1,
            39 => Gear::D2,
            24 => Gear::D3,
            16 => Gear::D4,
            12 => Gear::D5,
            09 => Gear::D6,
            _ => {
                internal_debug!("unknown gear byte: {}", gear_byte);
                error!("unknown gear byte: {}", gear_byte);
                Gear::U
            }
        };

        Ok(Self { gear })
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::TransaxlePid(self)
    }
}