use defmt::*;
use embassy_time::Duration;
use embedded_hal::delay::DelayNs;
use esp_hal::{
    delay::Delay,
    gpio::{self, InputPin, Pin, RtcPin, RtcPinWithResistors},
    rtc_cntl::sleep::{Ext1WakeupSource, TimerWakeupSource, WakeupLevel},
};

use crate::{
    debug::internal_debug,
    types::{IngGpio, Rs, Rtc},
};

pub struct Power {
    ing_gpio: IngGpio,
    rs_gpio: Rs,
    delay: Delay,
    rtc: Rtc,
}

pub enum Ignition {
    On,
    Off,
}

impl Power {
    pub fn new(ing_gpio: IngGpio, delay: Delay, rtc: Rtc, rs_gpio: Rs) -> Self {
        Self { ing_gpio, delay, rtc, rs_gpio }
    }

    pub fn deep_sleep(&mut self, duration: Duration) {
        let timer = TimerWakeupSource::new(duration.into());
        info!("going to deep sleep with timer wakeup: {:?}", defmt::Debug2Format(&timer));

        let mut ing_pin: esp_hal::gpio::GpioPin<5> = unsafe { esp_hal::gpio::GpioPin::steal() };

        let wakeup_pins: &mut [(&mut dyn RtcPinWithResistors, WakeupLevel)] = &mut [(&mut ing_pin, WakeupLevel::High)];

        let rtcio = Ext1WakeupSource::new(wakeup_pins);
        self.rs_gpio.set_low();
        let mut rtc = unwrap!(self.rtc.try_lock());
        rtc.sleep_deep(&[&timer, &rtcio]);
    }

    pub fn is_ignition_on(&self) -> bool {
        self.ing_gpio.is_high()
    }

    pub fn is_ignition_off(&self) -> bool {
        self.ing_gpio.is_low()
    }

    pub async fn wait_for_ignition_off(&mut self) {
        self.ing_gpio.wait_for_falling_edge().await;
    }

    pub async fn wait_for_ignition_on(&mut self) {
        self.ing_gpio.wait_for_rising_edge().await;
    }

    pub async fn wait_for_ignition_change(&mut self) -> Ignition {
        self.ing_gpio.wait_for_any_edge().await;
        if self.is_ignition_on() {
            warn!("ignition on");
            Ignition::On
        } else {
            warn!("ignition off");
            Ignition::Off
        }
    }

    pub fn rwdt_feed(&mut self) {
        if let Ok(mut rtc) = self.rtc.try_lock() {
            rtc.rwdt.feed();
        } else {
            warn!("failed to feed rwdt");
        }
    }
}
