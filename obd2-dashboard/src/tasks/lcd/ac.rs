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
pub struct LcdAcState {
    ac_compressor: Icon<embedded_iconoir::icons::size18px::weather::SnowFlake>,
}

impl LcdAcState {
    pub fn new() -> Self {
        warn!("LcdAcState::new()");
        Self { ac_compressor: Icon::new(Point::new(256 - 18, 18 + 18), true) }
    }

    pub fn handle_obd2_event(&mut self, event: &Obd2Event) {
        match event {
            Obd2Event::AcPid(ac_pid) => {}
            _ => {}
        }
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        self.ac_compressor.draw(display2).ok();

        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
