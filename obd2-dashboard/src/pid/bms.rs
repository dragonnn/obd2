use defmt::*;
use embedded_can::{Frame as _, StandardId};
pub use types::BmsPid;

use crate::{
    event::Obd2Event,
    mcp2515::CanFrame,
    obd2::{Obd2Error, Pid},
};

impl Pid for BmsPid {
    fn request() -> CanFrame {
        let can_id = unwrap!(StandardId::new(0x7e4));
        unwrap!(CanFrame::new(can_id, &[0x02, 0x21, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00]))
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

        //info!("hv_cell_voltage_deviation: {}", hv_cell_voltage_deviation);
        //0_Niro_Auxillary Battery Voltage	Aux Batt Volts	2101	ad*0.1
        let aux_dc_voltage = data[31] as f64 * 0.1;
        //info!("aux_dc_voltage: {}", aux_dc_voltage);
        //0_Niro_Battery Current	Batt Current	2101	((Signed(K)*256)+L)/10
        let hv_battery_current = (data[12] as i8 as i32 * 256 + data[13] as i32) as f64 / 10.0;
        //warn!("hv_battery_current: {}", hv_battery_current);
        //0_Niro_Minimum Cell Voltage	Min Cell V	2101	z/50
        let hv_min_cell_voltage = (data[27] as f64) / 50.0;
        //0_Niro_Maximum Cell Voltage	Max Cell V	2101	x/50
        let hv_max_cell_voltage = (data[25] as f64) / 50.0;

        let motor_electric_rpm = (data[55] as i32 * 256) as f64 + data[56] as f64;

        Ok(Self {
            hv_max_temp,
            hv_min_temp,
            hv_dc_voltage,
            hv_soc,
            hv_cell_voltage_deviation,
            hv_min_cell_voltage,
            hv_max_cell_voltage,
            hv_battery_current,
            aux_dc_voltage,
            motor_electric_rpm,
        })
    }

    fn into_event(self) -> Obd2Event {
        Obd2Event::BmsPid(self)
    }
}
