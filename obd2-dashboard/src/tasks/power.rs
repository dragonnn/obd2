use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use defmt::*;
use embassy_futures::select::{select3, Either3::*};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_hal::{
    debugger::debugger_connected,
    reset::SleepSource,
    //rtc_cntl::{get_reset_reason, get_wakeup_cause, SocResetReason},
    Cpu,
};

#[derive(Debug, Clone)]
pub enum PowerEvent {
    Shutdown(embassy_time::Duration),
    RwdtFeed,
}

static SHUTDOWN: PubSubChannel<CriticalSectionRawMutex, (), 1, 16, 1> = PubSubChannel::new();
static SHUTDOWN_GUARD_DROP_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static SHUTDOWN_GUARDS: AtomicUsize = AtomicUsize::new(0);
static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
static POWER_EVENTS: PubSubChannel<CriticalSectionRawMutex, PowerEvent, 1, 1, 16> = PubSubChannel::new();

use crate::{
    debug::internal_debug,
    event::*,
    power::{Ignition, Power},
};

//#[ram(rtc_fast, persistent)]
//static mut LAST_WAKEUP_CAUSE_STR: [u8; 32] = [0; 32];

#[embassy_executor::task]
pub async fn run(mut power: Power) {
    //let sleep_duration = Duration::from_secs(15 * 60);

    /*let reason = get_reset_reason(Cpu::ProCpu).unwrap_or(SocResetReason::ChipPowerOn);
    error!("reset reason: {:?}", defmt::Debug2Format(&reason));
    let wake_reason = get_wakeup_cause();
    error!("wake reason: {:?}", defmt::Debug2Format(&wake_reason));

    let mut sleep_timeout = Duration::from_secs(120);

    if let SleepSource::Timer = wake_reason {
        sleep_timeout = Duration::from_secs(30);
    }*/
    //unsafe {
    //    error!("last wakeup cause: {=[u8]:a}", LAST_WAKEUP_CAUSE_STR);
    //}

    //let wake_reason = get_wakeup_cause();
    //error!("wake reason: {:?}", defmt::Debug2Format(&wake_reason));

    //unsafe {
    //    use core::{fmt::Write, sync::atomic::Ordering, write};
    //    let mut buffer = heapless::String::<32>::new();
    //    write!(buffer, "{:?}", wake_reason);
    //    LAST_WAKEUP_CAUSE_STR[0..buffer.len()].clone_from_slice(buffer.as_bytes());
    //}

    if debugger_connected() {
        KIA_EVENTS.send(KiaEvent::IgnitionOn).await;
        return;
    }

    if power.is_ignition_on() {
        KIA_EVENTS.send(KiaEvent::IgnitionOn).await;
    } else {
        KIA_EVENTS.send(KiaEvent::IgnitionOff).await;
    }

    let mut power_events_sub = unwrap!(POWER_EVENTS.subscriber());

    warn!("power task select");
    loop {
        match select3(power.wait_for_ignition_change(), power_events_sub.next_message_pure(), Timer::after_secs(5))
            .await
        {
            First(ignition) => match ignition {
                Ignition::On => {
                    KIA_EVENTS.send(KiaEvent::IgnitionOn).await;
                }
                Ignition::Off => {
                    KIA_EVENTS.send(KiaEvent::IgnitionOff).await;
                }
            },
            Second(power_event) => match power_event {
                PowerEvent::Shutdown(duration) => {
                    warn!("shutdown event received for {:?}s", duration.as_secs());
                    unwrap!(SHUTDOWN.publisher()).publish_immediate(());
                    let delay_duration = if debugger_connected() {
                        warn!("debugger connected, deep sleeping in 5s");
                        Duration::from_secs(5)
                    } else {
                        warn!("debugger not connected, deep sleeping in 200ms");
                        Duration::from_millis(200)
                    };

                    embassy_time::with_timeout(Duration::from_secs(120), async {
                        SHUTDOWN_REQUESTED.store(true, Ordering::Relaxed);
                        while SHUTDOWN_GUARDS.load(Ordering::Relaxed) != 0 {
                            SHUTDOWN_GUARD_DROP_SIGNAL.wait().await;
                        }
                    })
                    .await
                    .ok();

                    Timer::after(delay_duration).await;
                    if power.is_ignition_on() {
                        warn!("ignition is on, not deep sleeping");
                        esp_hal::reset::software_reset();
                    } else {
                        info!("deep sleeping for {:?}", duration);
                        power.deep_sleep(duration);
                    }
                }
                PowerEvent::RwdtFeed => {
                    power.rwdt_feed();
                }
            },
            Third(_) => {
                if power.is_ignition_on() {
                    KIA_EVENTS.send(KiaEvent::IgnitionOn).await;
                } else {
                    KIA_EVENTS.send(KiaEvent::IgnitionOff).await;
                }
            }
        }
    }
}

pub fn get_shutdown_signal() -> embassy_sync::pubsub::Subscriber<'static, CriticalSectionRawMutex, (), 1, 16, 1> {
    unwrap!(SHUTDOWN.subscriber())
}

pub type PowerEventPublisher = embassy_sync::pubsub::DynPublisher<'static, PowerEvent>;

pub fn get_power_events_pub() -> PowerEventPublisher {
    unwrap!(POWER_EVENTS.dyn_publisher())
}

pub struct ShutdownGuard;

impl ShutdownGuard {
    pub fn new() -> Self {
        SHUTDOWN_GUARDS.fetch_add(1, Ordering::Relaxed);
        Self
    }
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        SHUTDOWN_GUARDS.fetch_sub(1, Ordering::Relaxed);
        let requested = SHUTDOWN_REQUESTED.load(Ordering::Relaxed);
        if requested {
            SHUTDOWN_GUARD_DROP_SIGNAL.signal(());
        }
    }
}
