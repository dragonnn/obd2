use defmt::unwrap;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{DynPublisher, DynSubscriber, PubSubChannel},
};

pub use crate::tasks::{
    lcd::{LcdEvent, EVENTS as LCD_EVENTS},
    obd2::Obd2Event,
    state::{KiaEvent, EVENTS as KIA_EVENTS},
};

pub enum Event {
    Lcd(LcdEvent),
    Obd2(Obd2Event),
    Kia(KiaEvent),
}

static EVENT_BUS: PubSubChannel<CriticalSectionRawMutex, LcdEvent, 32, 32, 32> = PubSubChannel::new();

pub fn event_bus_pub() -> DynPublisher<'static, LcdEvent> {
    unwrap!(EVENT_BUS.dyn_publisher())
}

pub fn event_bus_sub() -> DynSubscriber<'static, LcdEvent> {
    unwrap!(EVENT_BUS.dyn_subscriber())
}
