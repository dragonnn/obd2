use defmt::unwrap;
use embedded_graphics::geometry::{Point, Size};

use crate::{
    display::widgets::{Battery, BatteryOrientation},
    pid::BmsPid,
    types::{Display1, Display2, Sh1122},
};

#[derive(Default)]
pub struct LcdMainState {
    hv_battery: Battery,
}

impl LcdMainState {
    pub fn new() -> Self {
        Self {
            hv_battery: Battery::new(
                Point::new(9, 1),
                Size::new(128, 62),
                BatteryOrientation::HorizontalRight,
                Some(Size::new(8, 32)),
                4,
                true,
            ),
        }
    }

    pub fn update_bms_pid(&mut self, bms_pid: &BmsPid) {
        self.hv_battery.update_percentage(bms_pid.hv_soc);
        self.hv_battery.update_voltage(bms_pid.hv_dc_voltage);
        self.hv_battery.update_max_temp(bms_pid.hv_max_temp);
        self.hv_battery.update_min_temp(bms_pid.hv_min_temp);
        //self.hv_battery.update_cell_voltage_deviation(bms_pid.hv_cell_voltage_deviation);
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        self.hv_battery.draw(display1);

        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
