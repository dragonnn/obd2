use defmt::{unwrap, warn};
use embedded_graphics::geometry::{Point, Size};

use crate::{
    display::widgets::{
        Arrow, ArrowDirection, Battery, Battery12V, BatteryOrientation, GearboxGear, MotorElectric, MotorIce, Power,
        Temperature,
    },
    event::Obd2Event,
    pid::{BmsPid, GearboxGearPid, IceTemperaturePid},
    types::{Display1, Display2, Sh1122},
};

#[derive(Default)]
pub struct LcdMainState {
    hv_battery: Battery,
    aux_battery: Battery12V,

    ice_temperature: Temperature,

    electric_power: Power,
    electric_power_arrow: Arrow,

    motor_electric: MotorElectric,
    motor_ice: MotorIce,

    gearbox_gear: GearboxGear,
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

            electric_power: Power::new(Point::new(128 + 36, 14)),
            electric_power_arrow: Arrow::new(
                Point { x: 9 + 128, y: 64 / 2 - 9 },
                Size { width: 54, height: 16 },
                14,
                ArrowDirection::Reverse,
            ),

            motor_electric: MotorElectric::new(Point::new(256 - 60, 0)),
            motor_ice: MotorIce::new(Point::new(0, 0)),

            gearbox_gear: GearboxGear::new(Point::new(40, 14)),
        }
    }

    pub fn handle_obd2_event(&mut self, event: &Obd2Event) {
        match event {
            Obd2Event::BmsPid(bms_pid) => {
                self.update_bms_pid(bms_pid);
            }
            Obd2Event::IceTemperaturePid(ice_temperature_pid) => {
                self.update_ice_temperature(ice_temperature_pid);
            }
            Obd2Event::GearboxGearPid(gearbox_gear_pid) => {
                self.update_gearbox_gear(gearbox_gear_pid);
            }
            _ => {}
        }
    }

    pub fn update_bms_pid(&mut self, bms_pid: &BmsPid) {
        self.hv_battery.update_percentage(bms_pid.hv_soc);
        self.hv_battery.update_voltage(bms_pid.hv_dc_voltage);
        self.hv_battery.update_max_temp(bms_pid.hv_max_temp);
        self.hv_battery.update_min_temp(bms_pid.hv_min_temp);
        self.hv_battery.update_cell_voltage_deviation(bms_pid.hv_max_cell_voltage - bms_pid.hv_min_cell_voltage);
        self.aux_battery.update_voltage(bms_pid.aux_dc_voltage);
        self.electric_power_arrow.update_speed(50.0);
        self.electric_power.update_power(bms_pid.hv_battery_current * bms_pid.hv_dc_voltage);
        self.electric_power.update_current(bms_pid.hv_battery_current);
        if bms_pid.hv_battery_current > 0.0 {
            self.electric_power_arrow.update_direction(ArrowDirection::Forward);
        } else {
            self.electric_power_arrow.update_direction(ArrowDirection::Reverse);
        }
    }

    pub fn update_ice_temperature(&mut self, ice_temperature_pid: &IceTemperaturePid) {
        self.ice_temperature.update_temp(ice_temperature_pid.temperature);
    }

    pub fn update_gearbox_gear(&mut self, gearbox_gear_pid: &GearboxGearPid) {
        self.gearbox_gear.update_gear(gearbox_gear_pid.gear);
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        self.hv_battery.draw(display1).ok();
        self.aux_battery.draw(display2).ok();
        self.ice_temperature.draw(display2).ok();
        self.motor_electric.draw(display1).ok();
        self.motor_ice.draw(display2).ok();
        self.electric_power.draw(display1).ok();
        self.electric_power_arrow.draw(display1).ok();
        self.gearbox_gear.draw(display2).ok();

        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
