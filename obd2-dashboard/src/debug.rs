use defmt::*;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, TrySendError},
};
use heapless::String;

pub const DEBUG_STRING_LEN: usize = 120;
pub const DEBUG_CHANNEL_LEN: usize = 16;

static DEBUG_CHANNEL: Channel<CriticalSectionRawMutex, String<DEBUG_STRING_LEN>, DEBUG_CHANNEL_LEN> = Channel::new();

pub fn debug(string: String<DEBUG_STRING_LEN>) {
    if let Err(TrySendError::Full(string)) = DEBUG_CHANNEL.try_send(string) {
        let _ = DEBUG_CHANNEL.try_receive();
        DEBUG_CHANNEL.try_send(string).ok();
    }
}

macro_rules! internal_debug {
    ($($arg:tt)*) => {
        {
            use heapless::String as InternalDebugString;
            use core::fmt::Write as _;
            let mut string = InternalDebugString::new();
            core::write!(&mut string, "{:.1}: ", embassy_time::Instant::from_ticks(0).elapsed().as_millis() as f64 / 1000.0).ok();
            core::write!(&mut string, $($arg)*).ok();

            crate::debug::debug(string);
        }
    };
    () => {

    };
}

pub async fn receive() -> String<DEBUG_STRING_LEN> {
    DEBUG_CHANNEL.receive().await
}

pub(crate) use internal_debug;
