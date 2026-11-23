//! Shared UART RPC definitions for the nRF manager and Lyra application.
//!
//! Keep hardware drivers and endpoint handlers out of this crate so it can be
//! used unchanged by both the no_std firmware and the Linux client.

#![no_std]

use postcard_rpc::{TopicDirection, endpoints, topics};

pub const MAX_ECHO_BYTES: usize = 128;

/// Arbitrary bytes echoed unchanged, including zero bytes.
pub type EchoPayload = heapless::Vec<u8, MAX_ECHO_BYTES>;

/// Echoed unchanged in the matching reply or acknowledgement. IDs must not be
/// reused across power cycles within one nRF boot.
pub type ControlId = u64;

endpoints! {
    list = ENDPOINTS;
    | EndpointTy   | RequestTy   | ResponseTy  | Path        |
    | ----------   | ---------   | ----------  | ----        |
    | EchoEndpoint | EchoPayload | EchoPayload | "lyra/echo" |
}

topics! {
    list = TOPICS_IN;
    direction = TopicDirection::ToServer;
    | TopicTy          | MessageTy | Path                |
    | -------          | --------- | ----                |
    | EchoReplyTopic   | ControlId | "lyra/echo/reply"   |
    | ShutdownAckTopic | ControlId | "lyra/shutdown/ack" |
}

topics! {
    list = TOPICS_OUT;
    direction = TopicDirection::ToClient;
    | TopicTy              | MessageTy | Path                    |
    | -------              | --------- | ----                    |
    | EchoProbeTopic       | ControlId | "lyra/echo/probe"       |
    | ShutdownRequestTopic | ControlId | "lyra/shutdown/request" |
}
