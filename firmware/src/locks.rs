use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

pub static SPI_BUS: Mutex<CriticalSectionRawMutex, ()> = Mutex::new(());
