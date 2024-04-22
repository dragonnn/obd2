use embassy_time::Duration;
use esp_hal::{
    delay::Delay,
    gpio,
    rtc_cntl::{
        sleep::{RtcioWakeupSource, TimerWakeupSource, WakeupLevel},
        Rtc,
    },
};

use crate::types::IngGpio;

pub struct Power {
    ing_gpio: IngGpio,
    delay: Delay,
    rtc: Rtc<'static>,
}

impl Power {
    pub fn new(ing_gpio: IngGpio, delay: Delay, rtc: Rtc<'static>) -> Self {
        Self { ing_gpio, delay, rtc }
    }

    pub fn deep_sleep(&mut self, duration: Duration) {
        let timer = TimerWakeupSource::new(duration.into());

        let wakeup_pins: &mut [(&mut dyn gpio::RTCPinWithResistors, WakeupLevel)] =
            &mut [(&mut self.ing_gpio, WakeupLevel::High)];

        let rtcio = RtcioWakeupSource::new(wakeup_pins);

        self.rtc.sleep_deep(&[&timer, &rtcio], &mut self.delay);
    }
}
