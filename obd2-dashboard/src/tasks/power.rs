use defmt::*;
use embassy_futures::select::{select, Either::*};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::{Duration, Timer};
use esp_hal::{
    debugger::debugger_connected,
    reset::SleepSource,
    rtc_cntl::{get_reset_reason, get_wakeup_cause, SocResetReason},
    Cpu,
};

#[derive(Debug, Clone)]
pub enum PowerEvent {
    Shutdown(embassy_time::Duration),
    RwdtFeed,
}

static SHUTDOWN: PubSubChannel<CriticalSectionRawMutex, (), 1, 16, 1> = PubSubChannel::new();
static POWER_EVENTS: PubSubChannel<CriticalSectionRawMutex, PowerEvent, 1, 1, 16> = PubSubChannel::new();

use crate::{
    debug::internal_debug,
    event::*,
    power::{Ignition, Power},
};

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

    if power.is_ignition_on() {
        KIA_EVENTS.send(KiaEvent::IgnitionOn).await;
    } else {
        KIA_EVENTS.send(KiaEvent::IgnitionOff).await;
    }

    let mut power_events_sub = unwrap!(POWER_EVENTS.subscriber());

    warn!("power task select");
    loop {
        match select(power.wait_for_ignition_change(), power_events_sub.next_message_pure()).await {
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
                    warn!("shutdown event received for {:?}", duration.as_secs());
                    unwrap!(SHUTDOWN.publisher()).publish(()).await;
                    let duration = if debugger_connected() {
                        warn!("debugger connected, deep sleeping in 5s");
                        Duration::from_secs(5)
                    } else {
                        warn!("debugger not connected, deep sleeping in 200ms");
                        Duration::from_millis(200)
                    };
                    Timer::after(duration).await;
                    if power.is_ignition_on() {
                        warn!("ignition is on, not deep sleeping");
                        esp_hal::reset::software_reset();
                    } else {
                        power.deep_sleep(duration);
                    }
                }
                PowerEvent::RwdtFeed => {
                    power.rwdt_feed();
                }
            },
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
