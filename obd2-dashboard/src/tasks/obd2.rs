use defmt::{error, info, Format};
use embassy_futures::select::select;
use embassy_time::{with_timeout, Duration};
use serde::{Deserialize, Serialize};

use crate::{
    debug::internal_debug,
    event::{KiaEvent, KIA_EVENTS},
    obd2::{Obd2, Pid},
    pid,
    tasks::power::get_shutdown_signal,
};

#[derive(Format, PartialEq, Clone)]
pub enum Obd2Event {
    BmsPid(pid::BmsPid),
    IceTemperaturePid(pid::IceTemperaturePid),
    IceFuelRatePid(pid::IceFuelRatePid),
    VehicleSpeedPid(pid::VehicleSpeedPid),
    AcPid(pid::AcPid),
    HybridDcDcPid(pid::HybridDcDcPid),
    Icu(pid::IcuPid),
    IceEnginePid(pid::IceEnginePid),
    TransaxlePid(pid::TransaxlePid),
}

#[derive(PartialEq, Clone)]
pub struct Obd2Debug {
    pub type_id: &'static str,
    pub data: Option<alloc::vec::Vec<u8>>,
}

impl Obd2Debug {
    pub fn new<PID: Pid + core::any::Any>(data: Option<alloc::vec::Vec<u8>>) -> Self {
        Self { type_id: core::any::type_name::<PID>().split("::").last().unwrap_or_default(), data }
    }
}

impl defmt::Format for Obd2Debug {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Obd2Debug {}", self.type_id);
    }
}

#[embassy_executor::task]
pub async fn run(mut obd2: Obd2) {
    info!("obd2 task started");
    obd2.init().await;
    info!("obd2 init done");
    select(
        async {
            loop {
                //error!("obd2 task loop");
                obd2.handle_pid::<pid::BmsPid>().await;
                obd2.handle_pid::<pid::TransaxlePid>().await;
                obd2.handle_pid::<pid::IceTemperaturePid>().await;
                obd2.handle_pid::<pid::IceFuelRatePid>().await;
                obd2.handle_pid::<pid::VehicleSpeedPid>().await;
                obd2.handle_pid::<pid::AcPid>().await;
                obd2.handle_pid::<pid::HybridDcDcPid>().await;
                obd2.handle_pid::<pid::IcuPid>().await;
                obd2.handle_pid::<pid::IceEnginePid>().await;
                //error!("obd2 task loop end");

                #[cfg(debug_assertions)]
                embassy_time::Timer::after(embassy_time::Duration::from_secs(10)).await;
                #[cfg(not(debug_assertions))]
                embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
            }
        },
        get_shutdown_signal().next_message(),
    )
    .await;
    obd2.shutdown().await;
}
