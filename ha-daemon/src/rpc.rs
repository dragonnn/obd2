//! This library crate defines the remote counting service.
//!
//! The client and server depend on it.

use remoc::prelude::*;
use std::time::Duration;

/// TCP port the server is listening on.
pub const TCP_PORT: u16 = 49672;

/*/// Increasing the counter failed.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum IncreaseError {
    /// An overflow would occur.
    Overflow {
        /// The current value of the counter.
        current_value: u32,
    },
    /// The RTC call failed.
    Call(rtc::CallError),
}

impl From<rtc::CallError> for IncreaseError {
    fn from(err: rtc::CallError) -> Self {
        Self::Call(err)
    }
}*/

/// Remote counting service.
#[rtc::remote]
pub trait Rpc {
    async fn send_custom_frame(
        &self,
        frame: types::Obd2Frame,
    ) -> Result<
        (
            rch::oneshot::Receiver<()>,
            rch::oneshot::Receiver<types::Obd2Frame>,
        ),
        rtc::CallError,
    >;
}
