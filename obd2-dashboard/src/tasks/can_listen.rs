use core::sync::atomic::{AtomicUsize, Ordering};

use defmt::{error, info, Format};
use embassy_futures::select::select;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::{with_timeout, Duration};
use serde::{Deserialize, Serialize};
pub use types::{Pid as Obd2Event, PidError as Obd2Error};

use crate::{
    mcp2515::{clock_16mhz, OperationMode, CANINTE, CLKPRE, RXB0CTRL, RXB1CTRL, RXM},
    tasks::power::ShutdownGuard,
    types::Mcp2515,
};
#[embassy_executor::task]
pub async fn run(mut can_listen: Mcp2515) {
    return;
    info!("can listen task started");
    embassy_time::Timer::after(Duration::from_secs(10)).await;
    let _shutdown_guard = ShutdownGuard::new();
    let config = crate::mcp2515::Config::default()
        .mode(OperationMode::NormalOperation)
        .bitrate(clock_16mhz::CNF_500K_BPS)
        .set_clk_prescaler(CLKPRE::SystemClockDiv2)
        .receive_buffer_0(RXB0CTRL::default().with_rxm(RXM::ReceiveAny).with_bukt(true))
        .receive_buffer_1(RXB1CTRL::default().with_rxm(RXM::ReceiveAny));

    can_listen.apply_config(&config, false).await.ok();

    let interputs_config = CANINTE::default().with_rx0ie(true).with_rx1ie(true);
    can_listen.apply_interrupts_config(interputs_config).await.ok();
    with_timeout(Duration::from_secs(120), can_listen.shutdown()).await.ok();
}
