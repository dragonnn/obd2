use defmt::{expect, info, unwrap};
use defmt_brtt::DefmtConsumer;
use embedded_io_async::Write as _;

#[embassy_executor::task]
pub async fn run(mut usb_serial: crate::types::UsbSerial, mut logger: DefmtConsumer) {
    loop {
        let grant = logger.wait_for_log().await;
        let written_bytes = unwrap!(usb_serial.write(&grant).await);
        grant.release(written_bytes);
    }
}
