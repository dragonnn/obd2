use core::sync::atomic::{AtomicUsize, Ordering};

use defmt::{error, info, Format};
use embassy_futures::select::select;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::{with_timeout, Duration};
use serde::{Deserialize, Serialize};
pub use types::{Pid as Obd2Event, PidError as Obd2Error};

use crate::{tasks::power::ShutdownGuard, types::Mcp2515};
#[embassy_executor::task]
pub async fn run(mut can_listen: Mcp2515) {
    info!("can listen task started");
    let _shutdown_guard = ShutdownGuard::new();
    can_listen.shutdown().await;
}
