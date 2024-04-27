use defmt::unwrap;

use crate::{
    display::widgets::DebugScroll,
    types::{Display1, Display2},
};

#[derive(Default)]
pub struct LcdDebugState {
    debug: DebugScroll,
}

impl LcdDebugState {
    pub fn new() -> Self {
        Self { debug: DebugScroll::new() }
    }

    pub fn add_line(&mut self, line: &str) {
        self.debug.add_line(line);
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        self.debug.draw(display1, display2);
        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
