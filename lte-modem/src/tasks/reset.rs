use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{with_timeout, Duration};

static RESET_SIGNAL: Signal<CriticalSectionRawMutex, heapless::String<64>> = Signal::new();
static RESET_GUARDS: AtomicUsize = AtomicUsize::new(0);
static RESET_REQUESTED: AtomicBool = AtomicBool::new(false);

#[embassy_executor::task]
pub async fn task() {
    let reason = RESET_SIGNAL.wait().await;

    with_timeout(Duration::from_secs(60), async {
        while RESET_GUARDS.load(Ordering::Relaxed) != 0 {
            RESET_SIGNAL.wait().await;
        }
    })
    .await
    .ok();

    panic!("reset: {:?}", reason);
}

pub struct ResetGuard;

impl ResetGuard {
    pub fn new() -> Self {
        RESET_GUARDS.fetch_add(1, Ordering::Relaxed);
        Self
    }
}

impl Drop for ResetGuard {
    fn drop(&mut self) {
        RESET_GUARDS.fetch_sub(1, Ordering::Relaxed);
        let requested = RESET_REQUESTED.load(Ordering::Relaxed);
        if requested {
            RESET_SIGNAL.signal(heapless::String::new());
        }
    }
}

pub fn request_reset(reason: heapless::String<64>) {
    RESET_REQUESTED.store(true, Ordering::Relaxed);
    RESET_SIGNAL.signal(reason);
}
