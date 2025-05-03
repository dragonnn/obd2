use defmt::Format;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

use crate::tasks::lcd::EVENTS as LCD_EVENTS;
use crate::tasks::lcd::LcdEvent;

pub static EVENTS: Channel<CriticalSectionRawMutex, (u8, bool), 64> = Channel::new();

#[derive(Format, PartialEq, Eq, Clone, Copy)]
pub enum Button {
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
}

#[derive(Format, PartialEq, Eq, Clone, Copy)]
pub enum Action {
    Pressed(Button),
    Released(Button),
}

#[embassy_executor::task]
pub async fn run() {
    let events_receiver = EVENTS.receiver();
    loop {
        let (button, pressed) = events_receiver.receive().await;
        info!("Button event: button: {}, pressed: {}", button, pressed);
        match pressed {
            true => match button {
                0 => {
                    info!("Button B0 pressed");
                    LCD_EVENTS
                        .send(LcdEvent::Button(Action::Pressed(Button::B4)))
                        .await;
                }
                1 => {
                    info!("Button B1 pressed");
                    LCD_EVENTS
                        .send(LcdEvent::Button(Action::Pressed(Button::B5)))
                        .await;
                }
                2 => {
                    info!("Button B2 pressed");
                    LCD_EVENTS
                        .send(LcdEvent::Button(Action::Pressed(Button::B3)))
                        .await;
                }
                3 => {
                    info!("Button B3 pressed");
                    LCD_EVENTS
                        .send(LcdEvent::Button(Action::Pressed(Button::B4)))
                        .await;
                }
                4 => {
                    info!("Button B4 pressed");
                    LCD_EVENTS
                        .send(LcdEvent::Button(Action::Pressed(Button::B5)))
                        .await;
                }
                5 => {
                    info!("Button B5 pressed");
                    LCD_EVENTS
                        .send(LcdEvent::Button(Action::Pressed(Button::B1)))
                        .await;
                }
                6 => {
                    info!("Button B6 pressed");
                    LCD_EVENTS
                        .send(LcdEvent::Button(Action::Pressed(Button::B2)))
                        .await;
                }

                _ => {}
            },
            false => {
                // Handle button released
            }
        }
    }
}

pub fn init() {}
