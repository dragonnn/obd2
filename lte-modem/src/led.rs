use embassy_nrf::{
    peripherals::PWM0,
    pwm::{SimpleConfig, SimplePwm},
    Peripherals,
};

pub struct Led<'a> {
    pwm: SimplePwm<'a>,
}

impl Led<'_> {
    pub fn new(p: Peripherals) -> Self {
        let mut pwm = SimplePwm::new_3ch(p.PWM0, p.P0_13, p.P0_14, p.P0_16, &SimpleConfig::default());
        Self { pwm }
    }
}
