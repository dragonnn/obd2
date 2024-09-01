use defmt::{unwrap, warn};
use embedded_graphics::geometry::{Point, Size};

use crate::{
    display::widgets::{Battery, Battery12V, BatteryOrientation, MotorElectric, MotorIce, Temperature},
    pid::{BmsPid, IceTemperaturePid},
    types::{Display1, Display2, Sh1122},
};

#[derive(Default)]
pub struct LcdMainState {
    hv_battery: Battery,
    aux_battery: Battery12V,

    ice_temperature: Temperature,

    motor_electric: MotorElectric,
    motor_ice: MotorIce,
}

impl LcdMainState {
    pub fn new() -> Self {
        warn!("LcdMainState::new()");
        Self {
            hv_battery: Battery::new(
                Point::new(9, 1),
                Size::new(128, 62),
                BatteryOrientation::HorizontalRight,
                Some(Size::new(8, 32)),
                4,
                true,
            ),
            aux_battery: Battery12V::new(Point::new(256 - 41 - 22, 31)),
            ice_temperature: Temperature::new(Point::new(256 - 21, 0), Size::new(16, 64), 0.0, 130.0, 4),

            motor_electric: MotorElectric::new(Point::new(256 - 60, 0)),
            motor_ice: MotorIce::new(Point::new(0, 0)),
        }
    }

    pub fn update_bms_pid(&mut self, bms_pid: &BmsPid) {
        self.hv_battery.update_percentage(bms_pid.hv_soc);
        self.hv_battery.update_voltage(bms_pid.hv_dc_voltage);
        self.hv_battery.update_max_temp(bms_pid.hv_max_temp);
        self.hv_battery.update_min_temp(bms_pid.hv_min_temp);
        self.hv_battery.update_cell_voltage_deviation(bms_pid.hv_cell_voltage_deviation);
        self.aux_battery.update_voltage(bms_pid.aux_dc_voltage);
        //self.hv_battery.update_cell_voltage_deviation(bms_pid.hv_cell_voltage_deviation);
    }

    pub fn update_ice_temperature(&mut self, ice_temperature_pid: &IceTemperaturePid) {
        self.ice_temperature.update_temp(ice_temperature_pid.temperature);
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        self.hv_battery.draw(display1).ok();
        self.aux_battery.draw(display2).ok();
        self.ice_temperature.draw(display2).ok();
        self.motor_electric.draw(display1).ok();
        self.motor_ice.draw(display2).ok();

        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
