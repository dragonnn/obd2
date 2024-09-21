use core::sync::atomic::AtomicBool;

use defmt::*;
use embedded_graphics::prelude::*;

use crate::{
    display::widgets::{DebugScroll, Obd2DebugSelector, Slider},
    tasks::obd2::Obd2Debug,
    types::{Display1, Display2},
};

#[derive(Default)]
pub struct LcdSettingsState {
    contrast_slider: Slider,
}

impl LcdSettingsState {
    pub fn new() -> Self {
        error!("LcdSettingsState::new()");
        Self { contrast_slider: Slider::new(Point::new(128, 0), Size::new(128, 10)) }
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        self.contrast_slider.draw(display1).ok();
        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
