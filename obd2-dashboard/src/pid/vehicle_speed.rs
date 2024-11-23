use defmt::*;
use embedded_can::{Frame as _, StandardId};
pub use types::VehicleSpeedPid;

use crate::{
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

impl Pid for VehicleSpeedPid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x7df));
        unwrap!(CanFrame::new(can_id, &[0x02, 0x01, 0x0d, 0x00, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 7 {
            return Err(Obd2Error::FrameToShort);
        }

        Ok(Self { vehicle_speed: data[3] })
    }

    fn into_error() -> types::PidError {
        types::PidError::VehicleSpeedPid
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::VehicleSpeedPid(self)
    }
}
