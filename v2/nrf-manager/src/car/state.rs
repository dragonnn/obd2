use hsmc::Duration;
use hsmc::statechart;

#[derive(Debug, Clone)]
pub enum CarEvent {
    IgnitionOn,
    IgnitionOff,
}

pub struct CarData {}

statechart! {
    Car {
        context: CarData;
        events: CarEvent;
        default(Shutdown);
        state Shutdown {
            entry: on_enter;
            on(IgnitionOn) => Driving;
        }

        state Driving {
            entry: on_enter;
        }
    }
}

impl CarShutdownActions for CarActionContext<'_> {
    async fn on_enter(&mut self) {
        // In a real firmware this might `radio.start_rx().await` or
        // `display.flush().await` — actions are `async fn` under the
        // embassy feature so peripheral calls can run inline.
        info!("idle on_enter_listening");
    }
}

impl CarDrivingActions for CarActionContext<'_> {
    async fn on_enter(&mut self) {
        // In a real firmware this might `radio.start_rx().await` or
        // `display.flush().await` — actions are `async fn` under the
        // embassy feature so peripheral calls can run inline.
        info!("idle on_enter_listening");
    }
}

#[embassy_executor::task]
pub async fn run() {}
