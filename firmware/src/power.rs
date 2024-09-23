use defmt::warn;
use embassy_time::Duration;
use embedded_hal::delay::DelayNs;
use esp_hal::{
    delay::Delay,
    gpio::{self, InputPin, Pin, RtcPin, RtcPinWithResistors},
    rtc_cntl::{
        sleep::{Ext1WakeupSource, TimerWakeupSource, WakeupLevel},
        Rtc,
    },
};

use crate::{
    debug::internal_debug,
    types::{IngGpio, Rs},
};

pub struct Power {
    ing_gpio: IngGpio,
    rs_gpio: Rs,
    delay: Delay,
    rtc: Rtc<'static>,
}

impl Power {
    pub fn new(ing_gpio: IngGpio, delay: Delay, rtc: Rtc<'static>, rs_gpio: Rs) -> Self {
        Self { ing_gpio, delay, rtc, rs_gpio }
    }

    pub fn deep_sleep(&mut self, duration: Duration) {
        let timer = TimerWakeupSource::new(duration.into());

        let mut ing_pin = unsafe { esp_hal::gpio::Gpio5::steal() };

        let wakeup_pins: &mut [(&mut dyn RtcPinWithResistors, WakeupLevel)] = &mut [(&mut ing_pin, WakeupLevel::High)];

        let rtcio = Ext1WakeupSource::new(wakeup_pins);
        self.rs_gpio.set_low();
        self.delay.delay_us(100);
        self.rtc.sleep_deep(&[&timer, &rtcio]);
    }

    pub fn is_ignition_on(&self) -> bool {
        self.ing_gpio.is_high()
    }

    pub async fn wait_for_ignition_off(&mut self) {
        self.ing_gpio.wait_for_falling_edge().await;
    }

    pub async fn wait_for_ignition_on(&mut self) {
        self.ing_gpio.wait_for_rising_edge().await;
    }
}
