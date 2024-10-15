use embassy_nrf::gpio::{AnyPin, Input, Pull};

pub struct Button(Input<'static, AnyPin>);

impl Button {
    pub async fn new(pin: AnyPin) -> Self {
        let pin = Input::new(pin, Pull::Up);
        Self(pin)
    }

    pub async fn pressed(&mut self) {
        self.0.wait_for_any_edge().await;
    }
}
