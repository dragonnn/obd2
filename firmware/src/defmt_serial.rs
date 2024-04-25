use core::sync::atomic::{AtomicBool, Ordering};

use defmt::global_logger;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pipe::Pipe};

static mut ENCODER: defmt::Encoder = defmt::Encoder::new();
static TAKEN: AtomicBool = AtomicBool::new(false);
static mut CS_RESTORE: critical_section::RestoreState = critical_section::RestoreState::invalid();

pub static PIPE: Pipe<CriticalSectionRawMutex, 1024> = Pipe::new();

static mut INITIALIZED: bool = false;

pub fn defmt_serial() {
    unsafe {
        critical_section::with(|_| {
            INITIALIZED = true;
        });
    }
}

pub fn release() {
    unsafe {
        critical_section::with(|_| {
            if TAKEN.load(Ordering::Relaxed) {
                panic!("defmt logger taken reentrantly"); // I don't think this actually is
                                                          // possible.
            }

            INITIALIZED = false;
        });
    }
}

#[global_logger]
struct GlobalSerialLogger;

unsafe impl defmt::Logger for GlobalSerialLogger {
    fn acquire() {
        let restore = unsafe { critical_section::acquire() };

        if TAKEN.load(Ordering::Relaxed) {
            panic!("defmt logger taken reentrantly");
        }

        TAKEN.store(true, Ordering::Relaxed);

        unsafe {
            CS_RESTORE = restore;
        }

        unsafe { ENCODER.start_frame(write_serial) }
    }

    unsafe fn release() {
        ENCODER.end_frame(write_serial);
        TAKEN.store(false, Ordering::Relaxed);

        let restore = CS_RESTORE;
        critical_section::release(restore);
    }

    unsafe fn write(bytes: &[u8]) {
        ENCODER.write(bytes, write_serial);
    }

    unsafe fn flush() {}
}

/// Write to serial using proxy function. We must ensure this function is not called
/// several times in parallel.
fn write_serial(remaining: &[u8]) {
    PIPE.try_write(remaining).ok();
}
