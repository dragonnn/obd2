use defmt::{error, info, Format};
use embassy_futures::select::select;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::{with_timeout, Duration};
use serde::{Deserialize, Serialize};
pub use types::Pid as Obd2Event;

static OBD2_SETS: Mutex<CriticalSectionRawMutex, Obd2PidSets> = Mutex::new(Obd2PidSets::None);
static OBD2_SETS_CHANGED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

use crate::{
    debug::internal_debug,
    event::{KiaEvent, KIA_EVENTS},
    obd2::{Obd2, Pid},
    pid,
    tasks::power::get_shutdown_signal,
};

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

#[derive(Format, Clone, Copy)]
pub enum Obd2PidSets {
    IgnitionOn,
    IgnitionOff,
    Charging,
    None,
}

impl Obd2PidSets {
    pub async fn handle(&self, obd2: &mut Obd2) {
        match self {
            Self::IgnitionOn => Self::handle_ignition_on(obd2).await,
            Self::IgnitionOff => Self::handle_ignition_off(obd2).await,
            Self::Charging => Self::handle_charging(obd2).await,

            Self::None => {}
        }
    }

    async fn handle_ignition_on(obd2: &mut Obd2) {
        obd2.handle_pid::<pid::BmsPid>().await;
        obd2.handle_pid::<pid::TransaxlePid>().await;
        obd2.handle_pid::<pid::IceTemperaturePid>().await;
        obd2.handle_pid::<pid::IceFuelRatePid>().await;
        obd2.handle_pid::<pid::VehicleSpeedPid>().await;
        obd2.handle_pid::<pid::AcPid>().await;
        obd2.handle_pid::<pid::HybridDcDcPid>().await;
        obd2.handle_pid::<pid::IcuPid>().await;
        obd2.handle_pid::<pid::Icu2Pid>().await;
        obd2.handle_pid::<pid::IceEnginePid>().await;
        obd2.handle_pid::<pid::OnBoardChargerPid>().await;
    }

    async fn handle_charging(obd2: &mut Obd2) {
        obd2.handle_pid::<pid::BmsPid>().await;
        obd2.handle_pid::<pid::IceTemperaturePid>().await;
        obd2.handle_pid::<pid::IcuPid>().await;
        obd2.handle_pid::<pid::Icu2Pid>().await;
        obd2.handle_pid::<pid::OnBoardChargerPid>().await;
    }

    async fn handle_ignition_off(obd2: &mut Obd2) {
        obd2.handle_pid::<pid::BmsPid>().await;
        obd2.handle_pid::<pid::IcuPid>().await;
        obd2.handle_pid::<pid::Icu2Pid>().await;
    }
}

#[embassy_executor::task]
pub async fn run(mut obd2: Obd2) {
    info!("obd2 task started");
    obd2.init().await;
    info!("obd2 init done");
    OBD2_SETS_CHANGED.wait().await;
    select(
        async {
            loop {
                let sets = *OBD2_SETS.lock().await;
                sets.handle(&mut obd2).await;

                KIA_EVENTS.send(KiaEvent::Obd2LoopEnd).await;

                embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
            }
        },
        get_shutdown_signal().next_message(),
    )
    .await;
    obd2.shutdown().await;
}

pub async fn set_obd2_sets(sets: Obd2PidSets) {
    *OBD2_SETS.lock().await = sets;
    OBD2_SETS_CHANGED.signal(());
}
