use defmt::*;
use embedded_can::{Frame as _, StandardId};
pub use types::Icu2Pid;

use crate::{
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

impl Pid for Icu2Pid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x770));
        unwrap!(CanFrame::new(can_id, &[0x03, 0x22, 0xbc, 0x03, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 12 {
            return Err(Obd2Error::FrameToShort);
        }

        let back_door_driver_side_open = data[7] & 0b00000001 != 0;
        let actuator_back_dor_driver_side_unlock = data[7] & 0b00000010 != 0;
        let back_door_passenger_side_open = data[7] & 0b00000100 != 0;
        let actuator_back_door_passenger_side_unlock = data[7] & 0b00001000 != 0;
        let front_door_passenger_side_open = data[7] & 0b00010000 != 0;
        let front_door_driver_side_open = data[7] & 0b00100000 != 0;
        let trunk_open = data[7] & 0b10000000 != 0;

        let engine_hood_open = data[8] & 0b00000001 != 0;
        let driver_buckled = data[8] & 0b00000010 != 0;
        let passenger_buckled = data[8] & 0b00000100 != 0;
        let breaking_fluid = data[8] & 0b00010000 != 0;
        let ignition_1_on = data[8] & 0b00100000 != 0;
        let ignition_2_on = data[8] & 0b01000000 != 0;

        let signal_back_av = data[9] & 0b00000100 != 0;
        let ret = Self {
            back_door_driver_side_open,
            actuator_back_dor_driver_side_unlock,
            back_door_passenger_side_open,
            actuator_back_door_passenger_side_unlock,
            front_door_passenger_side_open,
            front_door_driver_side_open,
            trunk_open,

            engine_hood_open,
            driver_buckled,
            passenger_buckled,
            breaking_fluid,
            ignition_1_on,
            ignition_2_on,
            signal_back_av,
        };
        Ok(ret)
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::Icu2Pid(self)
    }

    fn period() -> Option<embassy_time::Duration> {
        Some(embassy_time::Duration::from_secs(1))
    }
}
