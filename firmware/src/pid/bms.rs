use defmt::{info, Format};
use embedded_can::{Frame as _, StandardId};

use crate::{
    debug::internal_debug,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

#[derive(Debug, Format, PartialEq, Clone)]
pub struct BmsPid {
    pub hv_max_temp: f64,
    pub hv_min_temp: f64,
    pub hv_dc_voltage: f64,
    pub hv_soc: f64,
    pub hv_cell_voltage_deviation: f64,

    pub aux_dc_voltage: f64,
}

const fn toruge_pro_index_to_right_index(index: usize) -> usize {
    let packets = index / 6;
    index + packets
}

impl Pid for BmsPid {
    fn request() -> CanFrame {
        let can_id = StandardId::new(0x7e4).unwrap();
        CanFrame::new(can_id, &[0x02, 0x21, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap()
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        if data.len() < 32 {
            return Err(Obd2Error::FrameToShort);
        }
        let hv_max_temp = data[16] as i8 as f64; //14
        let hv_min_temp = data[17] as i8 as f64;

        let hv_dc_voltage = (((data[14] as u32) << 8) as f64 + data[15] as f64) / 10.0;

        let hv_soc = data[6] as f64 / 2.0;

        let hv_cell_voltage_deviation = data[22] as f64 / 50.0;

        info!("hv_cell_voltage_deviation: {}", hv_cell_voltage_deviation);
        let aux_dc_voltage = data[31] as f64 * 0.1;
        info!("aux_dc_voltage: {}", aux_dc_voltage);

        Ok(Self { hv_max_temp, hv_min_temp, hv_dc_voltage, hv_soc, hv_cell_voltage_deviation, aux_dc_voltage })
    }

    fn filter_frame(frame: &CanFrame) -> bool {
        if frame.data().len() < 3 {
            internal_debug!("bms filter frame out length");
            return false;
        }
        //not sure about checking for the 0x61
        let ret = frame.id() == StandardId::new(0x7ec).unwrap().into() && frame.data()[2] == 0x61;
        if !ret {
            internal_debug!("bms frame out {:x?} {:x}", frame.id(), frame.data()[2]);
        }
        ret;
        true
    }
}
