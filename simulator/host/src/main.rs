#[macro_use]
extern crate log;

use std::{path::Path, time::Duration};

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher as _};
use notify_debouncer_full::new_debouncer;
use serial2_tokio::SerialPort;

mod config;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let mut config = config::Config::new();
    info!("config: {:?}", config);
    let port = SerialPort::open("/dev/ttyUSB0", 115200).unwrap();
    let mut buffer: [u8; 512] = [0; 512];
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            info!("res: {:?}", res);
        },
        Config::default(),
    )
    .unwrap();
    watcher
        .watch(Path::new("configs"), notify::RecursiveMode::Recursive)
        .unwrap();

    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_millis(100), None, tx).unwrap();

    debouncer
        .watcher()
        .watch(Path::new("configs"), RecursiveMode::Recursive)
        .unwrap();

    loop {
        if rx.try_recv().is_ok() {
            info!("Configs updated");
            config = config::Config::new();
        }

        let read = port.read(&mut buffer).await.unwrap();
        warn!("Read {} bytes in: {:x?}", read, &buffer[..read]);
        match cobs::decode_in_place(&mut buffer[..read]) {
            Ok(decoded) => {
                let can_id = buffer[0] as u32
                    | (buffer[1] as u32) << 8
                    | (buffer[2] as u32) << 16
                    | (buffer[3] as u32) << 24;
                info!("CAN ID: {:x}", can_id);
                let can_dlc = buffer[4] as usize;
                info!("CAN DLC: {}", can_dlc);
                let can_message = &buffer[5..decoded];
                info!("CAN Message: {:x?}", can_message);
                let request = config.find_request(can_id, can_message);
                info!("Request: {:?}", request);
                if let Some(request) = request {
                    for response in &request.response {
                        let mut response_buf = Vec::with_capacity(512);
                        response_buf.push(response.can_id as u8);
                        response_buf.push((response.can_id >> 8) as u8);
                        response_buf.push((response.can_id >> 16) as u8);
                        response_buf.push((response.can_id >> 24) as u8);
                        response_buf.push(response.message.len() as u8);
                        response_buf.extend_from_slice(&response.message);

                        let mut encoded = vec![0; response_buf.len() + 1];
                        cobs::encode(&response_buf, &mut encoded);
                        info!("Response: {:x?}", &encoded);
                        encoded.insert(0, 0);
                        port.write_all(&encoded).await.unwrap();
                    }
                }
            }
            Err(e) => {
                error!("Failed to decode COBS in: {:x?}", &buffer[..read]);
            }
        }
    }
}
