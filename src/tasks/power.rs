use defmt::error;
use embassy_time::{Duration, Timer};
use esp_hal::{
    rtc_cntl::{get_reset_reason, get_wakeup_cause, SocResetReason},
    Cpu,
};

use crate::{event::*, power::Power};

#[embassy_executor::task]
pub async fn run(mut power: Power) {
    embassy_time::Timer::after(embassy_time::Duration::from_secs(5)).await;
    let reason = get_reset_reason(Cpu::ProCpu).unwrap_or(SocResetReason::ChipPowerOn);
    error!("reset reason: {:?}", defmt::Debug2Format(&reason));
    let wake_reason = get_wakeup_cause();
    error!("wake reason: {:?}", defmt::Debug2Format(&wake_reason));

    let mut sleep_timeout = Duration::from_secs(1);

    if power.is_ignition_on() {
        KIA_EVENTS.send(KiaEvent::InitIgnitionOn).await;
        power.wait_for_ignition_off().await;
        error!("got low ignition signal");
    } else {
        KIA_EVENTS.send(KiaEvent::InitIgnitionOff).await;
        sleep_timeout = Duration::from_secs(60);
        defmt::warn!("ignition is off");
    }

    defmt::warn!("deep sleep in one second");
    KIA_EVENTS.send(KiaEvent::Shutdown).await;
    Timer::after(sleep_timeout).await;
    power.deep_sleep(Duration::from_secs(5 * 60));
}
