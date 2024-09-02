use defmt::{error, warn};
use embassy_time::{Duration, Timer};
use esp_hal::{
    rtc_cntl::{get_reset_reason, get_wakeup_cause, SocResetReason},
    Cpu,
};

use crate::{debug::internal_debug, event::*, power::Power};

#[embassy_executor::task]
pub async fn run(mut power: Power) {
    let reason = get_reset_reason(Cpu::ProCpu).unwrap_or(SocResetReason::ChipPowerOn);
    error!("reset reason: {:?}", defmt::Debug2Format(&reason));
    let wake_reason = get_wakeup_cause();
    error!("wake reason: {:?}", defmt::Debug2Format(&wake_reason));

    let mut sleep_timeout = Duration::from_secs(5);
    //testing
    if esp_hal::debugger::debugger_connected() {
        warn!("debugger connected");
        KIA_EVENTS.send(KiaEvent::InitIgnitionOn).await;
        return;
    }

    if power.is_ignition_on() {
        internal_debug!("wait for ignition off");
        KIA_EVENTS.send(KiaEvent::InitIgnitionOn).await;
        power.wait_for_ignition_off().await;
        error!("got low ignition signal");
        internal_debug!("got ignition off");
    } else {
        KIA_EVENTS.send(KiaEvent::InitIgnitionOff).await;
        sleep_timeout = Duration::from_secs(60);
        defmt::warn!("ignition is off");
        internal_debug!("ignition already off");
    }

    KIA_EVENTS.send(KiaEvent::Shutdown).await;
    Timer::after(sleep_timeout).await;
    defmt::warn!("deep sleep in 100ms");
    Timer::after(Duration::from_secs(5)).await;
    power.deep_sleep(Duration::from_secs(5 * 60));
}
