use embassy_nrf::{
    gpio::{AnyPin, Pin},
    pwm::{DutyCycle, Instance, Prescaler, SimpleConfig, SimplePwm},
    Peri,
};
pub struct Buzzer<'d>(SimplePwm<'d>);

impl<'d> Buzzer<'d> {
    pub fn new<T: Instance>(pwm: Peri<'static, T>, ch0: Peri<'static, AnyPin>) -> Self {
        let mut pwm = SimplePwm::new_1ch(pwm, ch0, &SimpleConfig::default());
        pwm.set_prescaler(Prescaler::Div2);
        pwm.set_max_duty(32767);
        Self(pwm)
    }

    pub fn prescaler(&mut self, prescaler: Prescaler) {
        self.0.set_prescaler(prescaler);
    }

    pub fn on(&mut self) {
        self.0.set_duty(0, DutyCycle::normal(32767 / 2));
    }

    pub fn off(&mut self) {
        self.0.set_duty(0, DutyCycle::normal(32767));
    }
}
