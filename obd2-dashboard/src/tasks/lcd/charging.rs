use defmt::{info, unwrap, warn};
use embedded_graphics::geometry::{Point, Size};
use types::{BmsPid, IceTemperaturePid, Pid as Obd2Event};

use crate::{
    display::widgets::{
        Arrow, ArrowDirection, Battery, Battery12V, BatteryOrientation, Connection, GearboxGear, IceFuelRate, Icon,
        MotorElectric, MotorIce, Position, Power, Temperature, Value,
    },
    tasks::ieee802154::{last_position, last_receive, last_send},
    types::{Display1, Display2},
};

#[derive(Default)]
pub struct LcdChargingState {
    hv_battery: Battery,

    electric_power: Power,
    electric_power_arrow: Arrow,

    connection: Connection,

    hv_battery_current: f32,
}

impl LcdChargingState {
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

            electric_power: Power::new(Point::new(128 + 36, 14)),
            electric_power_arrow: Arrow::new(
                Point { x: 9 + 128, y: 64 / 2 - 9 },
                Size { width: 54, height: 16 },
                14,
                ArrowDirection::Reverse,
            ),

            connection: Connection::new(Point::new(256 - 18, 0)),

            hv_battery_current: 0.0,
        }
    }

    pub fn handle_obd2_event(&mut self, event: &Obd2Event) {
        match event {
            Obd2Event::BmsPid(bms_pid) => {
                self.update_bms_pid(bms_pid);
            }
            Obd2Event::OnBoardChargerPid(obc) => {}
            _ => {}
        }
    }

    fn update_bms_pid(&mut self, bms_pid: &BmsPid) {
        self.hv_battery.update_percentage(bms_pid.hv_soc);
        self.hv_battery.update_voltage(bms_pid.hv_dc_voltage);
        self.hv_battery.update_max_temp(bms_pid.hv_max_temp);
        self.hv_battery.update_min_temp(bms_pid.hv_min_temp);
        self.hv_battery.update_cell_voltage((bms_pid.hv_max_cell_voltage + bms_pid.hv_min_cell_voltage) / 2.0);
        self.hv_battery
            .update_cell_voltage_deviation((bms_pid.hv_max_cell_voltage - bms_pid.hv_min_cell_voltage) * 100.0);
        self.electric_power_arrow.update_speed(50.0);
        self.electric_power.update_power(bms_pid.hv_battery_current * bms_pid.hv_dc_voltage);
        self.electric_power.update_current(bms_pid.hv_battery_current);
        if bms_pid.hv_battery_current > 0.0 {
            self.electric_power_arrow.update_direction(ArrowDirection::Forward);
        } else {
            self.electric_power_arrow.update_direction(ArrowDirection::Reverse);
        }
        self.hv_battery_current = bms_pid.hv_battery_current;
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        if let Some(last_send) = last_send() {
            self.connection.update_last_send(last_send.elapsed().as_millis() < 250);
        }
        if let Some(last_receive) = last_receive() {
            self.connection.update_last_receive(last_receive.elapsed().as_millis() < 250);
        }
        self.hv_battery.draw(display1).ok();
        self.electric_power.draw(display1).ok();
        self.electric_power_arrow.draw(display1).ok();
        self.connection.draw(display2).ok();

        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
