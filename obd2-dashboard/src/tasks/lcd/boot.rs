use defmt::{info, unwrap};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::Instant;
use heapless::FnvIndexMap;

use crate::{
    display::widgets::DebugScroll,
    types::{Display1, Display2},
};

#[derive(Debug, Clone, Copy)]
pub enum BootUp {
    Buttons,
    Obd2,
    CanListen,
}

#[derive(Default)]
pub struct LcdBootState {
    bootup_state: FnvIndexMap<BootUp, bool, 3>,
}

impl LcdBootState {
    pub fn new() -> Self {
        Self { bootup_state: FnvIndexMap::new() }
    }

    pub async fn draw(&mut self, display1: &mut Display1, display2: &mut Display2) {
        unwrap!(display1.flush().await);
        unwrap!(display2.flush().await);
    }
}
