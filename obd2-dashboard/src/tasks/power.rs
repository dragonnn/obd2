use defmt::{error, unwrap, warn};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::{Duration, Timer};
use esp_hal::{
    reset::SleepSource,
    rtc_cntl::{get_reset_reason, get_wakeup_cause, SocResetReason},
    Cpu,
};

static SHUTDOWN: PubSubChannel<CriticalSectionRawMutex, (), 1, 16, 1> = PubSubChannel::new();

use crate::{debug::internal_debug, event::*, power::Power};

#[embassy_executor::task]
pub async fn run(mut power: Power) {
    let mut sleep_duration = Duration::from_secs(5 * 60);

    let reason = get_reset_reason(Cpu::ProCpu).unwrap_or(SocResetReason::ChipPowerOn);
    error!("reset reason: {:?}", defmt::Debug2Format(&reason));
    let wake_reason = get_wakeup_cause();
    error!("wake reason: {:?}", defmt::Debug2Format(&wake_reason));

    let mut sleep_timeout = Duration::from_secs(5);

    //testing
    if esp_hal::debugger::debugger_connected() {
        warn!("debugger connected");
        KIA_EVENTS.send(KiaEvent::InitIgnitionOn).await;
        Timer::after(Duration::from_secs(5)).await;
        sleep_duration = Duration::from_secs(5);
        //return;
    }

    loop {
        if power.is_ignition_on() {
            internal_debug!("wait for ignition off");
            KIA_EVENTS.send(KiaEvent::InitIgnitionOn).await;
            power.wait_for_ignition_off().await;
            error!("got low ignition signal");
            internal_debug!("got ignition off");
            break;
        } else {
            KIA_EVENTS.send(KiaEvent::InitIgnitionOff).await;
            if let SleepSource::Timer = wake_reason {
                sleep_timeout = Duration::from_millis(200);
            } else {
                sleep_timeout = Duration::from_secs(60);
            }
            if embassy_time::with_timeout(sleep_timeout, power.wait_for_ignition_on()).await.is_err() {
                defmt::warn!("ignition is off");
                internal_debug!("ignition already off");
                break;
            }
            defmt::warn!("ignition is off");
            internal_debug!("ignition already off");
        }
    }

    KIA_EVENTS.send(KiaEvent::Shutdown).await;
    unwrap!(SHUTDOWN.publisher()).publish(()).await;
    Timer::after(sleep_timeout).await;
    defmt::warn!("deep sleep in 100ms");
    Timer::after(Duration::from_millis(100)).await;
    if power.is_ignition_on() {
        defmt::warn!("ignition is on, not deep sleeping");
        esp_hal::reset::software_reset();
    } else {
        power.deep_sleep(sleep_duration);
    }
}

pub fn get_shutdown_signal() -> embassy_sync::pubsub::Subscriber<'static, CriticalSectionRawMutex, (), 1, 16, 1> {
    unwrap!(SHUTDOWN.subscriber())
}
