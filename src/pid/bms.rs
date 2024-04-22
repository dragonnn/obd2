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

impl Pid for BmsPid {
    fn request() -> CanFrame {
        let can_id = StandardId::new(0x7df).unwrap();
        CanFrame::new(can_id, &[0x02, 0x21, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]).unwrap()
    }

    fn parse(data: &[u8]) -> Result<Self, Obd2Error> {
        info!("parsing bms pid: {:?}", data);

        let max_temp = data[14] as i8 as f64;
        let min_temp = data[15] as i8 as f64;
        //((m<8)+n)/10
        info!("data[12]: {:?}, data[13]: {:?}", data[12], data[13]);
        let dc_voltage = (((data[12] as u32) << 8) as f64 + data[13] as f64) / 10.0;

        Ok(Self { max_temp, min_temp, dc_voltage })
    }
}
