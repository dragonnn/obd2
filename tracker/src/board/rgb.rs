use embassy_nrf::{
    gpio::Pin,
    pwm::{Instance, Prescaler, SimplePwm},
    Peripheral,
};
pub struct Rgb<'d, T: Instance>(SimplePwm<'d, T>, bool);

const PWM_LINEAR: [u8; 256] = [
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
    4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 9, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15, 16, 16, 17,
    18, 19, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 30, 31, 32, 34, 35, 36, 38, 39, 41, 43, 44, 46, 48, 50, 52, 54, 56,
    58, 60, 62, 64, 67, 69, 71, 74, 76, 79, 82, 84, 87, 90, 93, 95, 98, 101, 104, 107, 110, 113, 116, 119, 122, 125,
    128, 131, 134, 137, 140, 143, 146, 149, 152, 155, 158, 161, 163, 166, 169, 172, 174, 177, 180, 182, 185, 187, 189,
    192, 194, 196, 198, 200, 202, 204, 206, 208, 210, 212, 213, 215, 217, 218, 220, 221, 222, 224, 225, 226, 228, 229,
    230, 231, 232, 233, 234, 235, 236, 237, 237, 238, 239, 240, 240, 241, 242, 242, 243, 243, 244, 244, 245, 245, 246,
    246, 247, 247, 247, 248, 248, 249, 249, 249, 249, 250, 250, 250, 250, 251, 251, 251, 251, 252, 252, 252, 252, 252,
    252, 252, 253, 253, 253, 253, 253, 253, 253, 253, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254,
    254, 254, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
];

impl<'d, T: Instance> Rgb<'d, T> {
    pub fn new(
        pwm: impl Peripheral<P = T> + 'd,
        r: impl Peripheral<P = impl Pin> + 'd,
        g: impl Peripheral<P = impl Pin> + 'd,
        b: impl Peripheral<P = impl Pin> + 'd,
        inverted: bool,
    ) -> Self {
        let mut pwm = SimplePwm::new_3ch(pwm, r, g, b);
        pwm.set_prescaler(Prescaler::Div1);
        pwm.set_max_duty(u8::MAX as u16);
        pwm.set_duty(0, u8::MAX as u16);
        pwm.set_duty(1, u8::MAX as u16);
        pwm.set_duty(2, u8::MAX as u16);
        Self(pwm, inverted)
    }

    pub fn off(&mut self) {
        self.0.set_duty(0, self.get_duty(0));
        self.0.set_duty(1, self.get_duty(0));
        self.0.set_duty(2, self.get_duty(0));
    }

    fn get_duty(&self, p: u8) -> u16 {
        let p = PWM_LINEAR[p as usize];

        if self.1 {
            (u8::MAX - p) as u16
        } else {
            p as u16
        }
    }

    pub fn r(&mut self, p: u8) {
        self.0.set_duty(0, self.get_duty(p));
    }

    pub fn g(&mut self, p: u8) {
        self.0.set_duty(1, self.get_duty(p));
    }

    pub fn b(&mut self, p: u8) {
        self.0.set_duty(2, self.get_duty(p));
    }

    pub fn rgb(&mut self, rgb: [u8; 3]) {
        self.r(rgb[0]);
        self.g(rgb[1]);
        self.b(rgb[2]);
    }
}
