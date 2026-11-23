use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use hsmc::Duration;
use hsmc::statechart;

use crate::board::LyraPower;

#[derive(Debug, Clone)]
pub enum LyraEvent {
    PowerOn,
    Shutdown,
}

pub struct LyraContext {
    lyra_power: LyraPower,
}

statechart! {
    Lyra {
        context: LyraContext;
        events: LyraEvent;
        default(Shutdown);
        state Shutdown {
            entry: on_enter;
            on(PowerOn) => PowerOn;
        }

        state PowerOn {
            entry: on_enter;
            on(Shutdown) => Shutdown;
        }
    }
}

impl LyraShutdownActions for LyraActionContext<'_> {
    async fn on_enter(&mut self) {
        self.lyra_power.power_off().await;
    }
}

impl LyraPowerOnActions for LyraActionContext<'_> {
    async fn on_enter(&mut self) {
        self.lyra_power.power_on().await;
    }
}

pub const QN: usize = 8;

pub static EVENT_CHAN: Channel<CriticalSectionRawMutex, LyraEvent, QN> = Channel::new();

pub fn sender() -> LyraSender {
    LyraSender::from_channel(&EVENT_CHAN)
}

#[embassy_executor::task]
pub async fn run(lyra_power: LyraPower) {
    info!("state task");
    let ctx = LyraContext { lyra_power };
    let mut machine = Lyra::new(ctx, &EVENT_CHAN);
    machine.run().await.unwrap();
    // `into_context()` still works after Embassy run exits cleanly.
    let _ctx = machine.into_context();
}
