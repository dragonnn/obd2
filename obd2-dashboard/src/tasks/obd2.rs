use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use defmt::*;
use embassy_futures::select::{select, Either};
pub use embassy_sync::watch::Watch;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex, signal::Signal};
use embassy_time::{with_timeout, Duration};
use serde::{Deserialize, Serialize};
pub use types::{Pid as Obd2Event, PidError as Obd2Error};

static OBD2_SETS_CHANGED: Signal<CriticalSectionRawMutex, Obd2PidSets> = Signal::new();
static OBD2_INITED: embassy_sync::watch::Watch<CriticalSectionRawMutex, bool, 10> = Watch::new_with(false);
static OBD2_INIT: AtomicBool = AtomicBool::new(false);
static OBD2_CUSTOM_FRAMES: Channel<CriticalSectionRawMutex, types::Obd2Frame, 8> = Channel::new();

use crate::{
    debug::internal_debug,
    event::{KiaEvent, KIA_EVENTS},
    obd2::{Obd2, Pid},
    pid,
    tasks::{ieee802154::extra_txframes_pub, power::get_shutdown_signal},
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
        let mut all_ok = true;
        let mut any_ok = false;

        obd2.enable_obd2_pid_periods();

        let ret = obd2.handle_pid::<pid::BmsPid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::TransaxlePid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::IceTemperaturePid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::IceFuelRatePid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::VehicleSpeedPid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::HybridDcDcPid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::IcuPid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::Icu2Pid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::Icu3Pid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::IceEnginePid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::OnBoardChargerPid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::Icu1Smk>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        let ret = obd2.handle_pid::<pid::AcPid>().await;
        all_ok = ret && all_ok;
        any_ok = ret || any_ok;

        if !any_ok {
            error!("obd2 pid error in handle_ignition_on");
            obd2.reset().await;
        }

        all_ok
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
    let ieee802154_extra_txframes_pub = extra_txframes_pub();
    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    {
        with_timeout(Duration::from_secs(60), async {
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
    OBD2_INIT.store(true, Ordering::Relaxed);
    KIA_EVENTS.send(KiaEvent::Obd2Init(true)).await;
    embassy_time::Timer::after(Duration::from_millis(100)).await;
    info!("obd2 init done");
    let obd2_custom_frames_recv = OBD2_CUSTOM_FRAMES.receiver();
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
                match select(obd2_custom_frames_recv.receive(), current_sets.loop_delay()).await {
                    Either::First(obd2_custom_frame) => {
                        crate::tasks::state::EVENTS.send(KiaEvent::IgnitionOffResetTimeout).await;
                        warn!("obd2 custom frame: {:?}", obd2_custom_frame);
                        if let Ok(response) = obd2.send_custom_frame(obd2_custom_frame).await {
                            ieee802154_extra_txframes_pub.publish(types::TxFrame::Obd2Frame(response)).await;
                        }
                    }
                    Either::Second(_) => {}
                }
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
            Ok(false) => {}
            Err(_) => {
                if OBD2_INIT.load(Ordering::Relaxed) {
                    break;
                }
            }
        }
    }
}

pub fn obd2_inited() -> bool {
    OBD2_INIT.load(Ordering::Relaxed)
}

pub async fn send_custom_frame(frame: types::Obd2Frame) {
    OBD2_CUSTOM_FRAMES.send(frame).await;
}
