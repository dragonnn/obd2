use defmt::*;

use crate::{
    display::widgets::{DebugScroll, Obd2DebugSelector},
    tasks::obd2::Obd2Debug,
    types::{Display1, Display2},
};

#[derive(Default)]
pub struct LcdObd2Pids {
    debug: Obd2DebugSelector,
}

impl LcdObd2Pids {
    pub fn new() -> Self {
        Self { debug: Obd2DebugSelector::new() }
    }

    pub fn handle_obd2_debug(&mut self, event: &Obd2Debug) {
        self.debug.handle_obd2_debug(event);
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        self.debug.draw(display1, display2);
        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
