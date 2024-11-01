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
    let shared_key_bytes = include_bytes!("../../../shared_key.bin");
    let shared_key: SharedKey = SharedKey::new(shared_key_bytes.clone());

    let mut buffer = [0u8; 4096];
    loop {
        let mut vec_buffer = Vec::with_capacity(4096);
        uarte_receive.wait_for_high().await;
        info!("uarte_receive high");
        let result = receive.read_until_idle(&mut buffer).await;
        if let Ok(result) = result {
            info!("uarte_receive read_until_idle {:?} {=[u8]:a}", result, buffer[..result]);
            vec_buffer.extend_from_slice(&buffer[..result]);

            let encrypted_message = EncryptedMessage::deserialize(vec_buffer).unwrap();
            let msg = types::TxFrame::decrypt_owned(&encrypted_message, &shared_key).unwrap();
            info!("uarte_receive decrypted {:?}", msg);
        } else {
            error!("uarte_receive read_until_idle error {:?}", result);
        }
        info!("uarte_receive low");
        uarte_receive.wait_for_low().await;
    }
}
