use defmt::error;
use embassy_time::Duration;
use esp_hal::{
    rtc_cntl::{get_reset_reason, get_wakeup_cause, SocResetReason},
    Cpu,
};

use crate::power::Power;

#[embassy_executor::task]
pub async fn run(mut power: Power) {
    embassy_time::Timer::after(embassy_time::Duration::from_secs(5)).await;
    let reason = get_reset_reason(Cpu::ProCpu).unwrap_or(SocResetReason::ChipPowerOn);
    error!("reset reason: {:?}", defmt::Debug2Format(&reason));
    let wake_reason = get_wakeup_cause();
    error!("wake reason: {:?}", defmt::Debug2Format(&wake_reason));

    if power.is_ignition_on() {
        /*let old_power = true;
        loop {
            let new_power = power.is_ignition_on();
            if new_power != old_power {
                error!("ignition state changed: {}", new_power);
                break;
            }
            embassy_time::Timer::after(embassy_time::Duration::from_millis(50)).await;
        }*/

        //power.wait_for_ignition_off().await;
        power.wait_for_ignition_off().await;
        error!("got low ignition signal");
    } else {
        defmt::warn!("ignition is off");
    }

    defmt::warn!("deep sleep in one second");
    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    power.deep_sleep(Duration::from_secs(5 * 60));
}
