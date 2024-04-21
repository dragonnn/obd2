use embedded_io_async::Write as _;

#[embassy_executor::task]
pub async fn run(mut usb_serial: crate::types::UsbSerial) {
    usb_serial.write(b"").await;
    crate::defmt_serial::defmt_serial();

    loop {
        let mut buf = [0u8; 128];
        let n = crate::defmt_serial::PIPE.read(&mut buf).await;
        usb_serial.write(&buf[..n]).await;
    }
}
