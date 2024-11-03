use serde::{Deserialize, Serialize};

pub mod auth;
pub mod device;
pub mod ws;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum IncomingMessage {
    Auth(auth::IncomingAuth),
    Ping,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum OutgoingMessage {
    Auth(auth::OutgoingAuth),
    DeviceWebHookHandle(device::WebHookHandle),
    Pong,
}
