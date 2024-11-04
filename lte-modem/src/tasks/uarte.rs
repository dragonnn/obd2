use alloc::vec::Vec;

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::{
    gpio::{Input, Output},
    peripherals::SERIAL1,
    uarte::{Uarte, UarteRx, UarteTx},
};
use serde_encrypt::{shared_key::SharedKey, traits::SerdeEncryptSharedKey as _, EncryptedMessage};

use crate::board::{BoardUarteRx, BoardUarteTx};

pub fn run(
    spawner: &Spawner,
    uarte: (BoardUarteTx, BoardUarteRx),
    uarte_send: Output<'static>,
    uarte_receive: Input<'static>,
) {
    spawner.spawn(send_task(uarte.0, uarte_send)).unwrap();
    spawner.spawn(receive_task(uarte.1, uarte_receive)).unwrap();
}

#[embassy_executor::task]
async fn send_task(send: BoardUarteTx, mut uarte_send: Output<'static>) {}

#[embassy_executor::task]
async fn receive_task(mut receive: BoardUarteRx, mut uarte_receive: Input<'static>) {
    let tx_channel_pub = crate::tasks::modem::link::tx_channel_pub();

    let shared_key: SharedKey = SharedKey::new(crate::SHARED_KEY.clone());

    let mut buffer = [0u8; 1024];
    loop {
        let mut vec_buffer = Vec::with_capacity(1024);
        uarte_receive.wait_for_high().await;
        let result = receive.read_until_idle(&mut buffer).await;
        if let Ok(result) = result {
            vec_buffer.extend_from_slice(&buffer[..result]);

            match EncryptedMessage::deserialize(vec_buffer) {
                Ok(encrypted_message) => match types::TxFrame::decrypt_owned(&encrypted_message, &shared_key) {
                    Ok(msg) => {
                        tx_channel_pub.publish(msg).await;
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
