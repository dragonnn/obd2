use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use hsmc::Duration;
use hsmc::statechart;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum CarEvent {
    IgnitionOn,
    IgnitionOff,
}

pub struct CarContext {
    lyra_event_sender: LyraSender,
}

statechart! {
    Car {
        context: CarContext;
        events: CarEvent;
        default(Shutdown);
        state Shutdown {
            entry: on_enter;
            on(IgnitionOn) => Driving;
        }

        state Driving {
            entry: on_enter;
            on(IgnitionOff) => Shutdown;
        }
    }
}

impl CarShutdownActions for CarActionContext<'_> {
    async fn on_enter(&mut self) {
        // In a real firmware this might `radio.start_rx().await` or
        // `display.flush().await` — actions are `async fn` under the
        // embassy feature so peripheral calls can run inline.
        info!("idle on_enter_listening");
        self.lyra_event_sender.send(LyraEvent::PowerOff).await.ok();
    }
}

impl CarDrivingActions for CarActionContext<'_> {
    async fn on_enter(&mut self) {
        // In a real firmware this might `radio.start_rx().await` or
        // `display.flush().await` — actions are `async fn` under the
        // embassy feature so peripheral calls can run inline.
        info!("idle on_enter_listening");
        self.lyra_event_sender.send(LyraEvent::PowerOn).await.ok();
    }
}

pub const QN: usize = 8;

pub static EVENT_CHAN: Channel<CriticalSectionRawMutex, CarEvent, QN> = Channel::new();

pub fn sender() -> CarSender {
    CarSender::from_channel(&EVENT_CHAN)
}

#[embassy_executor::task]
pub async fn run() {
    info!("state task");
    let ctx = CarContext {
        lyra_event_sender: lyra_event_sender(),
    };
    let mut machine = Car::new(ctx, &EVENT_CHAN);
    machine.run().await.unwrap();
    // `into_context()` still works after Embassy run exits cleanly.
    let _ctx = machine.into_context();
}
