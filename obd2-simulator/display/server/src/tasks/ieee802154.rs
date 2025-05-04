use embassy_time::Instant;

use core::sync::atomic::{AtomicBool, Ordering};

pub static LAST_SEND: AtomicBool = AtomicBool::new(false);
pub static LAST_RECEIVE: AtomicBool = AtomicBool::new(false);
pub static LAST_POSITION: AtomicBool = AtomicBool::new(false);

pub fn last_send() -> Option<Instant> {
    if LAST_SEND.load(Ordering::SeqCst) {
        Some(Instant::now())
    } else {
        Some(Instant::from_millis(0))
    }
}

pub fn last_receive() -> Option<Instant> {
    if LAST_RECEIVE.load(Ordering::SeqCst) {
        Some(Instant::now())
    } else {
        Some(Instant::from_millis(0))
    }
}

pub fn last_position() -> Option<Instant> {
    if LAST_POSITION.load(Ordering::SeqCst) {
        Some(Instant::now())
    } else {
        Some(Instant::from_millis(0))
    }
}
