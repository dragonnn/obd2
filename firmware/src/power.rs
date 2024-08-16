use defmt::warn;
use embassy_time::Duration;
use esp_hal::{
    delay::Delay,
    gpio::{self, InputPin, Pin, RtcPin},
    rtc_cntl::{
        sleep::{Ext1WakeupSource, TimerWakeupSource, WakeupLevel},
        Rtc,
    },
};

use crate::{debug::internal_debug, types::IngGpio};

pub struct Power {
    ing_gpio: IngGpio,
    delay: Delay,
    //rtc: Rtc<'static>,
}

impl Power {
    pub fn new(ing_gpio: IngGpio, delay: Delay, rtc: Rtc<'static>) -> Self {
        Self { ing_gpio, delay /*rtc*/ }
    }

    pub fn deep_sleep(&mut self, duration: Duration) {
        let timer = TimerWakeupSource::new(duration.into());

        //let wakeup_pins: &mut [(&mut dyn RtcPin, WakeupLevel)] = &mut [&mut self.ing_gpio];

        //let rtcio = RtcioWakeupSource::new(wakeup_pins);
        //let rtcio = Ext1WakeupSource::new(wakeup_pins);

        //self.rtc.sleep_deep(&[&timer, &rtcio], &mut self.delay);
    }

    pub fn is_ignition_on(&self) -> bool {
        self.ing_gpio.is_high()
    }

    pub async fn wait_for_ignition_off(&mut self) {
        self.ing_gpio.wait_for_falling_edge().await;
    }
}
