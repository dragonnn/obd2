use embassy_nrf::{peripherals::PWM0, pwm::SimplePwm, Peripherals};

pub struct Led<'a> {
    pwm: SimplePwm<'a, PWM0>,
}

impl Led<'_> {
    pub fn new(p: Peripherals) -> Self {
        let mut pwm = SimplePwm::new_3ch(p.PWM0, p.P0_13, p.P0_14, p.P0_16);
        Self { pwm }
    }
}
