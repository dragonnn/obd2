use defmt::{info, Format};
use embedded_can::{Frame as _, StandardId};

use crate::{
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

#[derive(Format)]
pub struct BmsPid {
    max_temp: f64,
    min_temp: f64,
    dc_voltage: f64,
}

const fn toruge_pro_index_to_right_index(index: usize) -> usize {
    let packets = index / 6;
    index + packets
}

impl Pid for BmsPid {
    fn request() -> CanFrame {
        let can_id = StandardId::new(0x7df).unwrap();
        CanFrame::new(can_id, &[0x02, 0x21, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap()
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        info!("parsing bms pid: {:?}", data);
        //(x / 6 - 1) * 7 + 6

        let max_temp = data[16] as i8 as f64; //14
        let min_temp = data[17] as i8 as f64;

        info!("toruge_pro_index_to_right_index 12: {} should be 14", toruge_pro_index_to_right_index(12));
        info!("toruge_pro_index_to_right_index 13: {} should be 15", toruge_pro_index_to_right_index(13));
        info!("toruge_pro_index_to_right_index 14: {} should be 16", toruge_pro_index_to_right_index(14));
        info!("toruge_pro_index_to_right_index 15: {} should be 17", toruge_pro_index_to_right_index(15));

        //((m<8)+n)/10
        info!("data[12]: {:?}, data[13]: {:?}", data[14], data[15]);
        let dc_voltage = (((data[14] as u32) << 8) as f64 + data[15] as f64) / 10.0;

        Ok(Self { max_temp, min_temp, dc_voltage })
    }
}
