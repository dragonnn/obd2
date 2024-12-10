use defmt_brtt::DefmtConsumer;

use crate::board::BoardDebugUarteTx;

#[embassy_executor::task]
pub async fn task(
    mut consumer: DefmtConsumer,
    mut uarte_tx_debug: BoardDebugUarteTx,
    panic_message: Option<&'static str>,
) {
    uarte_tx_debug.write(b"Logger task started\n").await.ok();
    if let Some(panic) = panic_message {
        for _ in 0..60 {
            uarte_tx_debug.write(panic.as_bytes()).await.ok();
            embassy_time::Timer::after_secs(1).await;
        }
    }
    loop {
        let grant = consumer.wait_for_log().await;
        let lenght = grant.len();
        //let written_bytes = write_my_log_data_over_usb(&grant).ok();
        // The step below is optional. Dropping the `Grant` releases
        // all read bytes.
        uarte_tx_debug.write(&grant).await.ok();

        grant.release(lenght);
    }
}
