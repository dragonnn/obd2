use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::{
    gpio::{Input, Output},
    peripherals::SERIAL1,
    uarte::{Uarte, UarteRx, UarteTx},
};

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
    let mut buffer = [0u8; 4096];
    loop {
        uarte_receive.wait_for_high().await;
        info!("uarte_receive high");
        let result = receive.read_until_idle(&mut buffer).await;
        info!("uarte_receive read_until_idle {:?}", result);
        uarte_receive.wait_for_low().await;
    }
}
