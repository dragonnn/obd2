use defmt::{expect, info, unwrap};
use defmt_brtt::DefmtConsumer;
use embassy_time::{with_timeout, Duration};
use embedded_io_async::Write as _;
use heapless::Vec;

#[embassy_executor::task]
pub async fn run(mut usb_serial: crate::types::UsbSerial, mut logger: DefmtConsumer) {
    loop {
        let mut buf: Vec<u8, 512> = Vec::new();
        {
            let grant = logger.wait_for_log().await;
            for b in grant.iter() {
                if buf.push(*b).is_err() {
                    break;
                }
            }
            grant.release(buf.len());
        }
        usb_serial.write_all(buf.as_slice()).await;
    }
}
