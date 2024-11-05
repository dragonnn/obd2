use serde::{Deserialize, Serialize};

pub mod auth;
pub mod device;
pub mod ws;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum IncomingMessage {
    Auth(auth::IncomingAuth),
    //{"id":8,"type":"result","success":true,"result":{"body":null,"status":200,"headers":{"Content-Type":"application/octet-stream"}}}
    Result {
        id: u64,
        r#type: String,
        success: bool,
    },
    Ping,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum OutgoingMessage {
    Auth(auth::OutgoingAuth),
    DeviceWebHookHandle(device::WebHookHandle),
    Pong,
}

#[derive(Serialize, Debug)]
pub struct UpdateLocation {}
