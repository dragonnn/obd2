use defmt::*;
use embassy_time::Duration;
use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Pull, RtcPinWithResistors},
    rtc_cntl::sleep::{Ext1WakeupSource, TimerWakeupSource, WakeupLevel},
};

use crate::types::{IngGpio, Rs, Rtc};

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

    pub fn deep_sleep(&mut self, duration: Duration) -> ! {
        let timer = TimerWakeupSource::new(duration.into());
        info!("going to deep sleep with timer wakeup: {:?}", defmt::Debug2Format(&timer));
        #[cfg(not(feature = "xiao"))]
        let mut ing_pin = unsafe { esp_hal::gpio::AnyPin::steal(5) };
        #[cfg(feature = "xiao")]
        let mut ing_pin = unsafe { esp_hal::gpio::AnyPin::steal(0) };
        let input = Input::new(ing_pin.reborrow(), InputConfig::default().with_pull(Pull::Up));
        core::mem::drop(input);

        let wakeup_pins: &mut [(&mut dyn RtcPinWithResistors, WakeupLevel)] = &mut [(&mut ing_pin, WakeupLevel::High)];

        let rtcio = Ext1WakeupSource::new(wakeup_pins);
        self.rs_gpio.set_low();
        let mut rtc = unwrap!(self.rtc.try_lock());
        warn!("deep sleep");
        rtc.sleep_deep(&[&timer, &rtcio]);
    }

    pub fn is_ignition_on(&mut self) -> bool {
        self.ing_gpio.is_high()
    }

    pub fn is_ignition_off(&mut self) -> bool {
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
