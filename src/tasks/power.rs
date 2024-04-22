use defmt::error;
use embassy_time::Duration;
use esp_hal::{
    rtc_cntl::{get_reset_reason, get_wakeup_cause, SocResetReason},
    Cpu,
};

use crate::power::Power;

#[embassy_executor::task]
pub async fn run(mut power: Power) {
    embassy_time::Timer::after(embassy_time::Duration::from_secs(60)).await;
    let reason = get_reset_reason(Cpu::ProCpu).unwrap_or(SocResetReason::ChipPowerOn);
    error!("reset reason: {:?}", defmt::Debug2Format(&reason));
    let wake_reason = get_wakeup_cause();
    error!("wake reason: {:?}", defmt::Debug2Format(&wake_reason));

    embassy_time::Timer::after(embassy_time::Duration::from_secs(60)).await;
    defmt::warn!("deep sleep in one second");
    embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    power.deep_sleep(Duration::from_secs(5 * 60));
}
