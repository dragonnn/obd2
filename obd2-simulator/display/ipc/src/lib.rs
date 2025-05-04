use remoc::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use std::time::Duration;

use types::Pid;

/// TCP port the server is listening on.
pub const TCP_PORT: u16 = 9871;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[repr(u8)]
pub enum DisplayIndex {
    Index0 = 0,
    Index1 = 1,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Ieee802154State {
    LastSend(bool),
    LastReceive(bool),
    LastPosition(bool),
}

/// Remote counting service.
#[rtc::remote]
pub trait Ipc {
    /// Increase the counter's value by the provided number.
    async fn display_flush(
        &mut self,
        index: DisplayIndex,
        data: Vec<u8>,
    ) -> Result<(), rtc::CallError>;

    async fn buttons(&mut self) -> Result<rch::mpsc::Receiver<(u8, bool)>, rtc::CallError>;
    async fn obd2_pids(&mut self) -> Result<rch::mpsc::Receiver<Pid>, rtc::CallError>;
    async fn ieee802154(&mut self) -> Result<rch::mpsc::Receiver<Ieee802154State>, rtc::CallError>;
}
