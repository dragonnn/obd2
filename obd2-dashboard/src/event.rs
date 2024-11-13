use defmt::{unwrap, Format};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{DynPublisher, DynSubscriber, PubSubChannel},
};

pub use crate::tasks::{
    lcd::{LcdEvent, EVENTS as LCD_EVENTS},
    obd2::Obd2Event,
    state::{KiaEvent, EVENTS as KIA_EVENTS},
};

#[derive(Format, Clone)]
pub enum Event {
    Lcd(LcdEvent),
    Obd2(Obd2Event),
    Kia(KiaEvent),
}

static EVENT_BUS: PubSubChannel<CriticalSectionRawMutex, Event, 128, 32, 32> = PubSubChannel::new();

//TODO: make pub to it via without_timeout and not try_send
//pub fn event_bus_pub() -> EventBusPub {
//    unwrap!(EVENT_BUS.dyn_publisher())
//}

pub fn event_bus_sub() -> EventBusSub {
    unwrap!(EVENT_BUS.dyn_subscriber())
}

pub type EventBusPub = DynPublisher<'static, Event>;
pub type EventBusSub = DynSubscriber<'static, Event>;
