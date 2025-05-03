use crate::tasks::lcd::EVENTS as LCD_EVENTS;
use crate::tasks::lcd::LcdEvent;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use types::Pid;

pub static EVENTS: Channel<CriticalSectionRawMutex, Pid, 64> = Channel::new();

#[embassy_executor::task]
pub async fn run() {
    let events_receiver = EVENTS.receiver();
    loop {
        let pid = events_receiver.receive().await;
        LCD_EVENTS.send(LcdEvent::Obd2Event(pid)).await;
    }
}

pub async fn obd2_init_wait() {}

#[derive(PartialEq, Clone)]
pub struct Obd2Debug {
    pub type_id: &'static str,
    pub data: Option<alloc::vec::Vec<u8>>,
}
