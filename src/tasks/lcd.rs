use defmt::{unwrap, Format};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embedded_graphics::geometry::{Point, Size};

use crate::{
    display::widgets::{Battery, BatteryOrientation},
    types::Sh1122,
};

pub static STATE: Signal<CriticalSectionRawMutex, State> = Signal::new();

#[derive(Format, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    PowerOff,
    Main,
}

impl State {
    pub async fn apply(&self, previous: &Self, display1: &mut Sh1122<10>, display2: &mut Sh1122<1>) {
        match self {
            State::PowerOff => {
                unwrap!(display1.sleep(true).await);
                unwrap!(display2.sleep(true).await);
            }
            State::Main => {
                if *previous == State::PowerOff {
                    unwrap!(display1.sleep(false).await);
                    unwrap!(display2.sleep(false).await);
                }

                //test code;
                let mut battery2 = Battery::new(
                    Point::new(9, 1),
                    Size::new(128, 62),
                    BatteryOrientation::HorizontalRight,
                    Some(Size::new(8, 32)),
                    4,
                    true,
                );
                unwrap!(battery2.draw(display2));
                unwrap!(display1.flush().await);
                unwrap!(display2.flush().await);
            }
        }
    }
}

#[embassy_executor::task]
pub async fn run(mut display1: Sh1122<10>, mut display2: Sh1122<1>) {
    unwrap!(display1.init(None).await);
    unwrap!(display2.init(None).await);

    display1.clear();
    display2.clear();

    unwrap!(display1.flush().await);
    unwrap!(display2.flush().await);
    let mut current_state = State::PowerOff;
    current_state.apply(&current_state, &mut display1, &mut display2).await;
    loop {
        let new_state = STATE.wait().await;
        if new_state != current_state {
            new_state.apply(&current_state, &mut display1, &mut display2).await;
            current_state = new_state;
        }
    }
}
