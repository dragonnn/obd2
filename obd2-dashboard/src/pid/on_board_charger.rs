use defmt::*;
use embassy_time::Duration;
use embedded_can::{Frame as _, StandardId};
pub use types::OnBoardChargerPid;

use crate::{
    event::{Obd2Error as PidError, Obd2Event},
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

impl Pid for OnBoardChargerPid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x7e5));
        unwrap!(CanFrame::new(can_id, &[0x02, 0x21, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]))
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 54 {
            return Err(Obd2Error::FrameToShort);
        }

        let ac_input_voltage_instant = data[6] as f64 * 2.56 + data[7] as f64 / 100.0;
        let ac_input_voltage_rms = data[8] as f64 * 2.56 + data[9] as f64 / 100.0;
        let pfc_output_voltage = data[10] as f64 * 25.6 + data[11] as f64 / 10.0;
        let obc_output_voltage = data[12] as f64 * 25.6 + data[13] as f64 / 10.0;
        let ac_input_current = data[14] as f64 * 2.56 + data[15] as f64 / 100.0;
        let obc_output_current = data[16] as f64 * 2.56 + data[17] as f64 / 100.0;
        let ac_input_frequency = data[18];
        let obc_temperature_a = data[19] as i8 - 100;
        let cp_voltage = data[21] as f64 * 2.56 + data[22] as f64 / 100.0;
        let cp_duty = data[23] as f64 * 25.6 + data[24] as f64 / 10.0;
        let cp_frequency = data[25] as f64 * 25.6 + data[26] as f64 / 10.0;
        let pd_voltage = data[27] as f64 * 2.56 + data[28] as f64 / 100.0;
        let interlock_voltage = data[29] as f64 * 2.56 + data[30] as f64 / 100.0;
        let aux_dc_voltage = data[31] as f64 * 2.56 + data[32] as f64 / 100.0;
        let ig3_voltage = data[33] as f64 * 2.56 + data[34] as f64 / 100.0;
        let pfc1_current_sensor_offset = data[51] as f64 - data[36] as f64 / 100.0;

        let ret = Self {
            ac_input_voltage_instant,
            ac_input_voltage_rms,
            pfc_output_voltage,
            obc_output_voltage,
            ac_input_current,
            obc_output_current,
            ac_input_frequency,
            obc_temperature_a,
            cp_voltage,
            cp_duty,
            cp_frequency,
            pd_voltage,
            interlock_voltage,
            aux_dc_voltage,
            ig3_voltage,
            pfc1_current_sensor_offset,
        };
        //info!("{:?}", ret);
        Ok(ret)
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::OnBoardChargerPid(self)
    }

    fn into_error() -> types::PidError {
        types::PidError::OnBoardChargerPid
    }

    fn period() -> Option<Duration> {
        Some(Duration::from_secs(1))
    }
}
