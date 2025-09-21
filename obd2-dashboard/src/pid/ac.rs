use defmt::*;
use embassy_time::Duration;
use embedded_can::{Frame as _, StandardId};
pub use types::AcPid;

use crate::{
    debug::internal_debug,
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

impl Pid for AcPid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x7b3));
        unwrap!(CanFrame::new(can_id, &[0x03, 0x22, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 36 {
            return Err(Obd2Error::FrameToShort);
        }

        let vehicle_front_temp = (data[8] as f32 / 255.0) * (87.5 + 40.0) - 40.0;
        let surround_temp = (data[9] as f32 / 255.0) * (87.5 + 40.0) - 40.0;
        let evaporator_temp = (data[10] as f32 / 255.0) * (87.5 + 40.0) - 40.0;

        let driver_mixing_air = (data[12] as f32 / 255.0) * 100.0;
        let passenger_air_direction = (data[13] as f32 / 255.0) * 100.0;

        let passenger_mixing_air = (data[15] as f32 / 255.0) * 100.0;
        let air_direction = (data[16] as f32 / 255.0) * 100.0;
        let input = (data[18] as f32 / 255.0) * 100.0;
        let humidity = data[28];
        let defrost_open = (data[29] as f32 / 255.0) * 100.0;

        let driver_vent_temp = (data[30] as f32 / 255.0) * (87.5 + 40.0) - 40.0;
        let driver_floor_temp = (data[31] as f32 / 255.0) * (87.5 + 40.0) - 40.0;
        let speed = data[32];
        let ice_cooling_temp = (data[33] as f32 / 255.0) * (143.25 + 48.0) - 48.0;

        let compressor_on = data[35] != 0;

        let ret = Self {
            vehicle_front_temp,
            surround_temp,
            evaporator_temp,
            driver_mixing_air,
            passenger_air_direction,
            passenger_mixing_air,
            air_direction,
            input,
            humidity,
            defrost_open,
            driver_vent_temp,
            driver_floor_temp,
            speed,
            ice_cooling_temp,
            compressor_on,
        };

        Ok(ret)
    }

    fn into_error() -> types::PidError {
        types::PidError::AcPid
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::AcPid(self)
    }

    fn period() -> Option<Duration> {
        Some(Duration::from_secs(5))
    }
}
