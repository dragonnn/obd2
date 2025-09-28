use embassy_nrf::{
    gpio::{AnyPin, Pin},
    pwm::{Instance, Prescaler, SimplePwm},
    Peri,
};
pub struct Buzzer<'d, T: Instance>(SimplePwm<'d, T>);

impl<'d, T: Instance> Buzzer<'d, T> {
    pub fn new(pwm: Peri<'static, T>, ch0: Peri<'static, AnyPin>) -> Self {
        let mut pwm = SimplePwm::new_1ch(pwm, ch0);
        pwm.set_prescaler(Prescaler::Div2);
        pwm.set_max_duty(32767);
        Self(pwm)
    }

    pub fn prescaler(&mut self, prescaler: Prescaler) {
        self.0.set_prescaler(prescaler);
    }

    pub fn on(&mut self) {
        self.0.set_duty(0, 32767 / 2);
    }

    pub fn off(&mut self) {
        self.0.set_duty(0, 32767);
    }
}
