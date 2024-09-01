use defmt::{error, info, Format};
use embassy_time::{with_timeout, Duration};

use crate::{
    debug::internal_debug,
    event::{KiaEvent, KIA_EVENTS},
    obd2::{Obd2, Pid},
    pid,
};

#[derive(Format, PartialEq, Clone)]
pub enum Obd2Event {
    BmsPid(pid::BmsPid),
    IceTemperaturePid(pid::IceTemperaturePid),
}

#[embassy_executor::task]
pub async fn run(mut obd2: Obd2) {
    info!("obd2 task started");
    obd2.init().await;
    info!("obd2 init done");
    loop {
        info!("requesting bms pid");
        obd2.handle_pid::<pid::BmsPid>().await;
        obd2.handle_pid::<pid::IceTemperaturePid>().await;

        #[cfg(debug_assertions)]
        embassy_time::Timer::after(embassy_time::Duration::from_secs(10)).await;
        #[cfg(not(debug_assertions))]
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1000)).await;
    }
}
