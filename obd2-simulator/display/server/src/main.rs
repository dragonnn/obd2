#![feature(impl_trait_in_assoc_type)]
#![feature(trivial_bounds)]

#[macro_use]
extern crate defmt;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};

use embassy_executor::Executor;
use static_cell::StaticCell;

extern crate alloc;

mod ipc_client;

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", COUNT.fetch_add(1, Ordering::Relaxed));

pub mod lcd {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/obd2-dashboard/src/tasks/lcd/mod.rs"
    ));
}

mod display {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/obd2-dashboard/src/display/mod.rs"
    ));
}

mod locks {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/obd2-dashboard/src/locks.rs"
    ));
}

mod debug {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/obd2-dashboard/src/debug.rs"
    ));
}

mod dummy_display;
mod hal;
mod tasks;
mod types;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static SERIAL: StaticCell<DefmtSerial> = StaticCell::new();

pub struct DefmtSerial {
    sender: Sender<u8>,
}

impl DefmtSerial {
    pub fn new(sender: Sender<u8>) -> Self {
        DefmtSerial { sender }
    }
}

impl defmt_serial::EraseWrite for DefmtSerial {
    fn write(&mut self, buf: &[u8]) {
        for byte in buf {
            self.sender.send(*byte).unwrap();
        }
    }

    fn flush(&mut self) {
        std::println!("Flushing");
    }
}

fn main() {
    let mut args: Vec<String> = std::env::args().collect();

    let bin = args.pop().unwrap();

    let (tx, rx): (Sender<u8>, Receiver<u8>) = channel();
    defmt_serial::defmt_serial(SERIAL.init(DefmtSerial::new(tx)));

    std::thread::spawn(move || {
        let bin = std::fs::read(bin).unwrap();
        let defmt_table = defmt_decoder::Table::parse(&bin).unwrap().unwrap();
        let defmt_locs = defmt_table.get_locations(&bin).unwrap();
        let mut decoder = defmt_table.new_stream_decoder();
        let mut config = defmt_decoder::log::format::FormatterConfig::default();
        config.is_timestamp_available = true;
        config.format = defmt_decoder::log::format::FormatterFormat::Default {
            with_location: true,
        };
        let formatter = defmt_decoder::log::format::Formatter::new(config);

        loop {
            let byte = rx.recv().unwrap();
            decoder.received(&[byte]);
            if let Ok(frame) = decoder.decode() {
                let mut file = None;
                let mut line = None;
                let mut module = None;

                if let Some(location) = defmt_locs.get(&frame.index()) {
                    file = Some(location.file.to_str().unwrap());
                    line = Some(location.line as u32);
                    module = Some(location.module.to_string());
                }

                let out =
                    formatter.format_frame(frame, file, line, module.as_ref().map(|s| s.as_str()));
                std::println!("{}", out);
            }
        }
    });

    let executor = EXECUTOR.init(Executor::new());

    let (display_tx, display_rx) = tokio::sync::mpsc::unbounded_channel();

    let display1 = dummy_display::DummyDisplay::new(ipc::DisplayIndex::Index0, display_tx.clone());
    let display2 = dummy_display::DummyDisplay::new(ipc::DisplayIndex::Index1, display_tx.clone());

    ipc_client::start(display_rx);

    executor.run(|spawner| {
        spawner.spawn(run()).unwrap();

        spawner
            .spawn(tasks::lcd::run(display1, display2, None))
            .ok();
        spawner.spawn(tasks::buttons::run()).ok();
        spawner.spawn(tasks::obd2::run()).ok();
    });
}

use crate::debug::internal_debug;

#[embassy_executor::task]
async fn run() {
    tasks::lcd::EVENTS.send(tasks::lcd::LcdEvent::Main).await;
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
        internal_debug!("simulator tick");
    }
}
