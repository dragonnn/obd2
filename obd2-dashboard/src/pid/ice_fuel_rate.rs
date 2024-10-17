use defmt::{debug, info, unwrap, warn, Format};
use embedded_can::{Frame as _, StandardId};

use crate::{
    debug::internal_debug,
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

#[derive(Debug, Format, PartialEq, Clone)]
pub struct IceFuelRatePid {
    pub fuel_rate: f64,
}

impl Pid for IceFuelRatePid {
    fn request() -> CanFrame {
        //[0x04, 0x41, 0x5e, 0x00, 0x27, 0xaa, 0xaa, 0xaa]
        let can_id = unwrap!(StandardId::new(0x7df));
        unwrap!(CanFrame::new(can_id, &[0x02, 0x01, 0x5E, 0x00, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 7 {
            return Err(Obd2Error::FrameToShort);
        }
        //info!("fuel rate: {=[u8]:#04x}", data);
        Ok(Self { fuel_rate: (data[3] as i32 * 256 + data[4] as i32) as f64 / 20.0 })
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::IceFuelRatePid(self)
    }
}
