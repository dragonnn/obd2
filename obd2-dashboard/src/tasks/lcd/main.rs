use defmt::{info, unwrap, warn};
use embedded_graphics::geometry::{Point, Size};
use types::{BmsPid, IceTemperaturePid, Pid as Obd2Event};

use crate::{
    display::widgets::{
        Arrow, ArrowDirection, Battery, Battery12V, BatteryOrientation, Connection, GearboxGear, Humidity, IceFuelRate,
        Icon, MotorElectric, MotorIce, Position, Power, Temperature, Value,
    },
    tasks::ieee802154::{last_position, last_receive, last_send},
    types::{Display1, Display2},
};

#[derive(Default)]
pub struct LcdMainState {
    hv_battery: Battery,
    aux_battery: Battery12V,

    ice_temperature: Temperature,
    ice_fuel_rate: IceFuelRate,

    electric_power: Power,
    electric_power_arrow: Arrow,

    motor_electric: MotorElectric,
    motor_electric_rpm: Value,
    motor_ice: MotorIce,

    gearbox_gear: GearboxGear,
    vehicle_speed: Value,

    //humidity: Icon<embedded_iconoir::icons::size24px::design_tools::Droplet>,
    humidity: Humidity,

    connection: Connection,
    position: Position,

    ice_fuel_rate_value: f32,
    hv_battery_current: f32,
    vehicle_speed_value: f32,
}

impl defmt::Format for LcdMainState {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "LcdMainState {{  }}");
    }
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
            aux_battery: Battery12V::new(Point::new(84, 0)),
            ice_temperature: Temperature::new(Point::new(127, 0), Size::new(16, 64), 0.0, 130.0, 4),

            electric_power: Power::new(Point::new(128 + 36, 14)),
            electric_power_arrow: Arrow::new(
                Point { x: 9 + 128, y: 64 / 2 - 9 },
                Size { width: 54, height: 16 },
                14,
                ArrowDirection::Reverse,
            ),

            motor_electric: MotorElectric::new(Point::new(256 - 60, 0)),
            motor_electric_rpm: Value::new(Point::new(128 + 12, 55), &profont::PROFONT_10_POINT, "rpm", 0),

            motor_ice: MotorIce::new(Point::new(0, 0)),

            gearbox_gear: GearboxGear::new(Point::new(40, 14)),
            vehicle_speed: Value::new(Point::new(58, 48), &profont::PROFONT_14_POINT, "km/h", 0),
            ice_fuel_rate: IceFuelRate::new(Point::new(60, 60)),

            //humidity: Icon::new(Point::new(256 - 18 - 18, 18 + 18), true),
            humidity: Humidity::new(Point::new(228, 32)),

            connection: Connection::new(Point::new(256 - 18, 0)),
            position: Position::new(Point::new(256 - 18, 16)),

            ice_fuel_rate_value: 0.0,
            hv_battery_current: 0.0,
            vehicle_speed_value: 0.0,
        }
    }

    pub fn handle_obd2_event(&mut self, event: &Obd2Event) {
        match event {
            Obd2Event::BmsPid(bms_pid) => {
                self.update_bms_pid(bms_pid);
            }
            Obd2Event::IceTemperaturePid(ice_temperature_pid) => {
                self.ice_temperature.update_temp(ice_temperature_pid.temperature);
            }
            Obd2Event::IceFuelRatePid(ice_fuel_rate_pid) => {
                self.ice_fuel_rate_value = ice_fuel_rate_pid.fuel_rate;
                self.ice_fuel_rate.update_ice_fuel_rate(ice_fuel_rate_pid.fuel_rate);
            }
            Obd2Event::VehicleSpeedPid(vehicle_speed_pid) => {
                let speed = vehicle_speed_pid.vehicle_speed as f32;
                self.vehicle_speed.update_value(speed + speed * 0.1);
                self.ice_fuel_rate.update_vehicle_speed(speed);
                self.vehicle_speed_value = speed;
            }
            Obd2Event::TransaxlePid(transaxle_pid) => {
                self.gearbox_gear.update_gear(transaxle_pid.gear.into());
                self.gearbox_gear.update_clutch1_temp(transaxle_pid.clutch1_temp);
                self.gearbox_gear.update_clutch2_temp(transaxle_pid.clutch2_temp);
            }
            Obd2Event::AcPid(ac_pid) => {
                self.humidity.update_humidity(ac_pid.humidity);
                self.humidity.update_compressor(ac_pid.compressor_on);
                self.humidity.update_evaporator_temp(ac_pid.evaporator_temp);
            }
            _ => {}
        }
    }

    pub fn update_bms_pid(&mut self, bms_pid: &BmsPid) {
        self.hv_battery.update_percentage(bms_pid.hv_soc);
        self.hv_battery.update_voltage(bms_pid.hv_dc_voltage);
        self.hv_battery.update_max_temp(bms_pid.hv_max_temp);
        self.hv_battery.update_min_temp(bms_pid.hv_min_temp);
        self.hv_battery.update_cell_voltage((bms_pid.hv_max_cell_voltage + bms_pid.hv_min_cell_voltage) / 2.0);
        self.hv_battery
            .update_cell_voltage_deviation((bms_pid.hv_max_cell_voltage - bms_pid.hv_min_cell_voltage) * 100.0);
        self.aux_battery.update_voltage(bms_pid.aux_dc_voltage);
        self.electric_power_arrow.update_speed(50.0);
        self.electric_power.update_power(bms_pid.hv_battery_current * bms_pid.hv_dc_voltage);
        self.electric_power.update_current(bms_pid.hv_battery_current);
        if bms_pid.hv_battery_current > 0.0 {
            self.electric_power_arrow.update_direction(ArrowDirection::Forward);
        } else {
            self.electric_power_arrow.update_direction(ArrowDirection::Reverse);
        }
        self.hv_battery_current = bms_pid.hv_battery_current;
        self.motor_electric_rpm.update_value(bms_pid.motor_electric_rpm);
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        if let Some(last_send) = last_send() {
            self.connection.update_last_send(last_send.elapsed().as_millis() < 250);
        }
        if let Some(last_receive) = last_receive() {
            self.connection.update_last_receive(last_receive.elapsed().as_millis() < 250);
        }
        if let Some(last_position) = last_position() {
            self.position.update_last_position(last_position.elapsed().as_millis() < 250);
        }

        if self.motor_ice.update_on(self.ice_fuel_rate_value > 0.0) {
            self.gearbox_gear.force_redraw();
        }
        self.motor_electric.update_on(if self.ice_fuel_rate_value == 0.0 {
            true
        } else {
            self.hv_battery_current > 0.0
        });

        self.hv_battery.draw(display1).ok();
        self.aux_battery.draw(display2).ok();
        self.ice_temperature.draw(display2).ok();
        self.motor_electric.draw(display1).ok();
        self.motor_ice.draw(display2).ok();
        self.electric_power.draw(display1).ok();
        self.electric_power_arrow.draw(display1).ok();
        self.gearbox_gear.draw(display2).ok();
        self.ice_fuel_rate.draw(display2).ok();
        self.vehicle_speed.draw(display2).ok();
        self.motor_electric_rpm.draw(display1).ok();
        self.connection.draw(display2).ok();
        self.humidity.draw(display2).ok();
        self.position.draw(display2).ok();

        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
