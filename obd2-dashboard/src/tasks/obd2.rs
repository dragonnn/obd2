use core::sync::atomic::{AtomicUsize, Ordering};

use defmt::*;
use embassy_futures::select::select;
pub use embassy_sync::watch::Watch;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::{with_timeout, Duration};
use serde::{Deserialize, Serialize};
pub use types::{Pid as Obd2Event, PidError as Obd2Error};

static OBD2_SETS_CHANGED: Signal<CriticalSectionRawMutex, Obd2PidSets> = Signal::new();
static OBD2_INITED: embassy_sync::watch::Watch<CriticalSectionRawMutex, bool, 10> = Watch::new_with(false);

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

#[derive(Format, Clone, Copy, PartialEq, Eq)]
pub enum Obd2PidSets {
    IgnitionOn,
    IgnitionOff,
    Charging,
    None,
}

impl Obd2PidSets {
    pub async fn handle(&self, obd2: &mut Obd2) -> bool {
        match self {
            Self::IgnitionOn => Self::handle_ignition_on(obd2).await,
            Self::IgnitionOff => Self::handle_ignition_off(obd2).await,
            Self::Charging => Self::handle_charging(obd2).await,

            Self::None => false,
        }
    }

    async fn handle_ignition_on(obd2: &mut Obd2) -> bool {
        let mut ret = true;
        obd2.enable_obd2_pid_periods();
        ret = obd2.handle_pid::<pid::BmsPid>().await && ret;
        ret = obd2.handle_pid::<pid::TransaxlePid>().await && ret;
        ret = obd2.handle_pid::<pid::IceTemperaturePid>().await && ret;
        ret = obd2.handle_pid::<pid::IceFuelRatePid>().await && ret;
        ret = obd2.handle_pid::<pid::VehicleSpeedPid>().await && ret;
        ret = obd2.handle_pid::<pid::AcPid>().await && ret;
        ret = obd2.handle_pid::<pid::HybridDcDcPid>().await && ret;
        ret = obd2.handle_pid::<pid::IcuPid>().await && ret;
        ret = obd2.handle_pid::<pid::Icu2Pid>().await && ret;
        ret = obd2.handle_pid::<pid::Icu3Pid>().await && ret;
        ret = obd2.handle_pid::<pid::IceEnginePid>().await && ret;
        ret = obd2.handle_pid::<pid::OnBoardChargerPid>().await && ret;
        ret = obd2.handle_pid::<pid::Icu1Smk>().await && ret;

        ret
    }

    async fn handle_charging(obd2: &mut Obd2) -> bool {
        let mut ret = true;
        obd2.disable_obd2_pid_periods();

        ret = obd2.handle_pid::<pid::BmsPid>().await && ret;
        ret = obd2.handle_pid::<pid::IceTemperaturePid>().await && ret;
        ret = obd2.handle_pid::<pid::IcuPid>().await && ret;
        ret = obd2.handle_pid::<pid::Icu2Pid>().await && ret;
        ret = obd2.handle_pid::<pid::Icu3Pid>().await && ret;
        ret = obd2.handle_pid::<pid::OnBoardChargerPid>().await && ret;
        ret = obd2.handle_pid::<pid::Icu1Smk>().await && ret;

        ret
    }

    async fn handle_ignition_off(obd2: &mut Obd2) -> bool {
        let mut ret = true;
        obd2.disable_obd2_pid_periods();
        ret = obd2.handle_pid::<pid::IcuPid>().await && ret;
        ret = obd2.handle_pid::<pid::Icu2Pid>().await && ret;
        ret = obd2.handle_pid::<pid::Icu3Pid>().await && ret;
        ret = obd2.handle_pid::<pid::Icu1Smk>().await && ret;
        //ignore obc charger error in ignition off
        obd2.handle_pid::<pid::OnBoardChargerPid>().await;
        ret
    }

    pub async fn loop_delay(&self) {
        let delay = match self {
            Self::Charging => embassy_time::Duration::from_secs(1),
            Self::IgnitionOff => embassy_time::Duration::from_secs(1),
            Self::IgnitionOn | Self::None => embassy_time::Duration::from_millis(100),
        };
        embassy_time::Timer::after(delay).await;
    }
}

#[embassy_executor::task]
pub async fn run(mut obd2: Obd2) {
    info!("obd2 task started");
    let obd2_inited = OBD2_INITED.sender();
    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    {
        with_timeout(Duration::from_secs(30 * 60), async {
            loop {
                if obd2.init().await.is_ok() {
                    break;
                }
                KIA_EVENTS.send(KiaEvent::Obd2Init(false)).await;
            }
        })
        .await
        .ok();
    }
    obd2_inited.send(true);
    KIA_EVENTS.send(KiaEvent::Obd2Init(true)).await;
    embassy_time::Timer::after(Duration::from_millis(100)).await;
    info!("obd2 init done");
    let mut current_sets = OBD2_SETS_CHANGED.wait().await;
    select(
        async {
            loop {
                if let Some(new_sets) = OBD2_SETS_CHANGED.try_take() {
                    error!("obd2 sets changed: {:?}", new_sets);
                    if current_sets != new_sets {
                        current_sets = new_sets;
                        obd2.clear_pids_cache();
                    }
                }
                let all = current_sets.handle(&mut obd2).await;

                KIA_EVENTS.send(KiaEvent::Obd2LoopEnd(current_sets, all)).await;
                current_sets.loop_delay().await;
            }
        },
        get_shutdown_signal().next_message(),
    )
    .await;
    obd2.shutdown().await;
}

pub async fn set_obd2_sets(sets: Obd2PidSets) {
    error!("obd2 sets changed: {:?}", sets);
    OBD2_SETS_CHANGED.signal(sets);
}

pub async fn obd2_init_wait() {
    let mut obd2_inited_recv = unwrap!(OBD2_INITED.receiver());
    while obd2_inited_recv.try_changed_and(|o| *o) != Some(true) {
        match with_timeout(Duration::from_millis(100), obd2_inited_recv.changed()).await {
            Ok(true) => break,
            Err(_) | Ok(false) => {}
        }
    }
}

pub fn obd2_inited() -> bool {
    let mut obd2_inited_recv = unwrap!(OBD2_INITED.receiver());
    obd2_inited_recv.try_changed_and(|o| *o) == Some(true)
}
