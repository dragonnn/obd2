use core::sync::atomic::AtomicBool;

use defmt::*;

use crate::{
    display::widgets::{DebugScroll, Obd2DebugSelector},
    tasks::obd2::Obd2Debug,
    types::{Display1, Display2},
};

static OBD2_DEBUG_PIDS_ENABLED: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub struct LcdObd2Pids {
    debug: Obd2DebugSelector,
}

impl defmt::Format for LcdObd2Pids {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "LcdObd2Pids {{  }}");
    }
}

impl LcdObd2Pids {
    pub fn new() -> Self {
        OBD2_DEBUG_PIDS_ENABLED.store(true, core::sync::atomic::Ordering::Relaxed);
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

impl Drop for LcdObd2Pids {
    fn drop(&mut self) {
        OBD2_DEBUG_PIDS_ENABLED.store(false, core::sync::atomic::Ordering::Relaxed);
    }
}

pub fn obd2_debug_pids_enabled() -> bool {
    OBD2_DEBUG_PIDS_ENABLED.load(core::sync::atomic::Ordering::Relaxed)
}
