use alloc::vec::Vec;

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::{
    gpio::{Input, Output},
    peripherals::SERIAL1,
    uarte::{Uarte, UarteRx, UarteTx},
};
use embassy_time::{with_timeout, Duration};
use serde_encrypt::{shared_key::SharedKey, traits::SerdeEncryptSharedKey as _, EncryptedMessage};
use types::Modem;

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
        if let Ok(encrypted_message) = types::RxMessage::new(msg).to_vec_encrypted() {
            uarte_send.set_high();
            embassy_time::Timer::after(Duration::from_millis(10)).await;
            warn!("uarte_send high");
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

    let shared_key: SharedKey = SharedKey::new(crate::SHARED_KEY.clone());

    let mut buffer = [0u8; 1024];
    loop {
        let mut vec_buffer = Vec::with_capacity(1024);
        match with_timeout(Duration::from_secs(60), uarte_receive.wait_for_high()).await {
            Ok(_) => {}
            Err(_) => {
                let battery_state = crate::tasks::battery::State::get().await;
                if battery_state.charging {
                    error!("uarte_receive wait_for_high timeout");
                    uarte_reset.set_low();
                    embassy_time::Timer::after(Duration::from_millis(10)).await;
                    uarte_reset.set_high();
                }
            }
        }
        let result = receive.read_until_idle(&mut buffer).await;
        if let Ok(result) = result {
            vec_buffer.extend_from_slice(&buffer[..result]);

            match EncryptedMessage::deserialize(vec_buffer) {
                Ok(encrypted_message) => match types::TxMessage::decrypt_owned(&encrypted_message, &shared_key) {
                    Ok(msg) => {
                        if let types::TxFrame::Modem(Modem::Ping) = msg.frame {
                            rx_channel_pub.publish_immediate(types::RxFrame::Modem(Modem::Pong));
                        } else {
                            tx_channel_pub.publish_immediate(msg.frame);
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
