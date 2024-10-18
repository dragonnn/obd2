use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::{
    gpio::{Input, Output},
    peripherals::SERIAL1,
    uarte::{Uarte, UarteRx, UarteTx},
};

pub fn run(
    spawner: &Spawner,
    uarte: Uarte<'static, SERIAL1>,
    uarte_send: Output<'static>,
    uarte_receive: Input<'static>,
) {
    let (send, receive) = uarte.split();
    spawner.spawn(send_task(send, uarte_send)).unwrap();
    spawner.spawn(receive_task(receive, uarte_receive)).unwrap();
}

#[embassy_executor::task]
async fn send_task(send: UarteTx<'static, SERIAL1>, mut uarte_send: Output<'static>) {}

#[embassy_executor::task]
async fn receive_task(receive: UarteRx<'static, SERIAL1>, mut uarte_receive: Input<'static>) {
    loop {
        uarte_receive.wait_for_high().await;
        info!("uarte_receive high");
    }
}
