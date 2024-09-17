use defmt::*;

use crate::{
    display::widgets::DebugScroll,
    tasks::obd2::Obd2Debug,
    types::{Display1, Display2},
};

#[derive(Default)]
pub struct LcdObd2Pids {
    debug: DebugScroll,
}

impl LcdObd2Pids {
    pub fn new() -> Self {
        error!("LcdObd2Pids::new");
        Self { debug: DebugScroll::new() }
    }

    pub fn handle_obd2_debug(&mut self, event: &Obd2Debug) {}

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        self.debug.draw(display1, display2);
        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
