use alloc::vec::Vec;

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select3, Either3::*};
use embassy_nrf::{
    gpio::{Input, Output},
    peripherals::SERIAL1,
    uarte::{Uarte, UarteRx, UarteTx},
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    mutex::Mutex,
    pubsub::{DynPublisher, DynSubscriber, PubSubChannel},
    signal::Signal,
};
use embassy_time::{with_timeout, Duration, Instant};
use serde_encrypt::{shared_key::SharedKey, traits::SerdeEncryptSharedKey as _, EncryptedMessage};
use types::Modem;

static STATE_CHANNEL: PubSubChannel<CriticalSectionRawMutex, types::State, 16, 8, 2> = PubSubChannel::new();
static CURRENT_STATE: Mutex<CriticalSectionRawMutex, types::State> =
    Mutex::new(types::State::Shutdown(core::time::Duration::from_secs(15 * 60)));
static UARTE_RESET: Signal<CriticalSectionRawMutex, ()> = Signal::new();

use crate::board::{BoardUarteRx, BoardUarteTx};

pub fn run(
    spawner: &Spawner,
    uarte: (BoardUarteTx, BoardUarteRx),
    uarte_send: Output<'static>,
    uarte_receive: Input<'static>,
    uarte_reset: Output<'static>,
) {
    spawner.spawn(send_task(uarte.0, uarte_send)).unwrap();
    spawner.spawn(receive_task(uarte.1, uarte_receive, uarte_reset)).unwrap();
}

#[embassy_executor::task]
async fn send_task(mut send: BoardUarteTx, mut uarte_send: Output<'static>) {
    let mut rx_channel_sub = crate::tasks::modem::link::rx_channel_sub();
    loop {
        let msg = rx_channel_sub.next_message_pure().await;
        warn!("sending {:?}", msg);
        if let Ok(encrypted_message) = msg.to_vec_encrypted() {
            uarte_send.set_high();
            embassy_time::Timer::after(Duration::from_millis(10)).await;
            send.write(&encrypted_message).await.unwrap();
            uarte_send.set_low();
            embassy_time::Timer::after(Duration::from_millis(10)).await;
        }
    }
}

#[embassy_executor::task]
async fn receive_task(mut receive: BoardUarteRx, mut uarte_receive: Input<'static>, mut uarte_reset: Output<'static>) {
    let tx_channel_pub = crate::tasks::modem::link::tx_channel_pub();
    let rx_channel_pub = crate::tasks::modem::link::rx_channel_pub();
    let state_channel_pub = unwrap!(STATE_CHANNEL.publisher());

    let shared_key: SharedKey = SharedKey::new(crate::SHARED_KEY.clone());
    let mut last_communication = Instant::now();

    let mut buffer = [0u8; 1024];
    loop {
        let mut vec_buffer = Vec::with_capacity(1024);
        let mut do_uarte_reset = false;
        match select3(uarte_receive.wait_for_high(), embassy_time::Timer::after_secs(60), UARTE_RESET.wait()).await {
            First(_) => {}
            Second(_) => {
                let battery_state = crate::tasks::battery::State::get().await;
                if battery_state.charging {
                    do_uarte_reset = true;
                } else if last_communication.elapsed().as_secs() > 15 * 60 {
                    do_uarte_reset = true;
                }
            }
            Third(_) => do_uarte_reset = true,
        }

        if do_uarte_reset {
            error!("uarte_receive wait_for_high reset");
            uarte_reset.set_low();
            embassy_time::Timer::after(Duration::from_millis(10)).await;
            uarte_reset.set_high();
        } else {
            let result = receive.read_until_idle(&mut buffer).await;
            if let Ok(result) = result {
                vec_buffer.extend_from_slice(&buffer[..result]);

                match EncryptedMessage::deserialize(vec_buffer) {
                    Ok(encrypted_message) => match types::TxMessage::decrypt_owned(&encrypted_message, &shared_key) {
                        Ok(msg) => {
                            last_communication = Instant::now();
                            if let types::TxFrame::State(state) = &msg.frame {
                                state_channel_pub.publish_immediate(state.clone());
                                {
                                    *CURRENT_STATE.lock().await = state.clone();
                                }
                            }

                            if let types::TxFrame::Modem(Modem::Ping) = &msg.frame {
                                warn!("sending modem pong");
                                rx_channel_pub.publish_immediate(types::RxFrame::TxFrameAck(msg.id).into());
                                rx_channel_pub
                                    .publish_immediate(types::RxMessage::new(types::RxFrame::Modem(Modem::Pong)));
                            } else {
                                if !msg.needs_ack() {
                                    warn!("not remote ack, sending ack from modem");
                                    rx_channel_pub.publish_immediate(types::RxFrame::TxFrameAck(msg.id).into());
                                }
                                tx_channel_pub.publish_immediate(msg);
                            }
                        }
                        Err(e) => {
                            error!("uarte_receive decrypt error {:?}", defmt::Debug2Format(&e));
                        }
                    },
                    Err(e) => {
                        error!("uarte_receive deserialize error {:?}", defmt::Debug2Format(&e));
                    }
                }
            } else {
                error!("uarte_receive read_until_idle error {:?}", result);
            }
            //info!("uarte_receive low");
            uarte_receive.wait_for_low().await;
        }
    }
}

pub fn state_channel_sub() -> DynSubscriber<'static, types::State> {
    unwrap!(STATE_CHANNEL.dyn_subscriber())
}

pub fn state_channel_pub() -> DynPublisher<'static, types::State> {
    unwrap!(STATE_CHANNEL.dyn_publisher())
}

pub async fn current_state() -> types::State {
    CURRENT_STATE.lock().await.clone()
}

pub async fn set_current_state(state: types::State) {
    *CURRENT_STATE.lock().await = state;
}

pub fn reset() {
    UARTE_RESET.signal(());
}
