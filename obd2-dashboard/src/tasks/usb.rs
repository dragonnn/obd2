use defmt::info;
use embedded_io_async::Write as _;

#[embassy_executor::task]
pub async fn run(mut usb_serial: crate::types::UsbSerial) {
    loop {
        usb_serial.write(b"hello world").await;
        info!("USB: hello world");
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
