use serde::Deserialize;
use serde::Serialize;

use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Config {
    pub configs: BTreeMap<String, Vec<Request>>,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut ret = Config {
            configs: BTreeMap::new(),
        };

        for entry in std::fs::read_dir("configs")? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let content = std::fs::read_to_string(path)?;
            let config: Vec<Request> = ron::from_str(&content)?;
            ret.configs.insert(name, config);
        }

        Ok(ret)
    }

    pub fn find_request(&self, can_id: u32, can_message: &[u8]) -> Option<&Request> {
        for (_, requests) in &self.configs {
            for request in requests {
                if request.can_id == can_id && request.message == can_message {
                    return Some(request);
                }
            }
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub can_id: u32,
    pub message: Vec<u8>,
    pub response: Vec<Response>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Raw(RawResponse),
    Consecutive(ConsecutiveResponse),
}

impl Response {
    pub fn into_raw_responses(&self) -> Vec<RawResponse> {
        match self {
            Response::Raw(raw) => vec![raw.clone()],
            Response::Consecutive(consecutive) => consecutive.into_raw_responses(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawResponse {
    pub can_id: u32,
    pub message: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsecutiveResponse {
    pub can_id: u32,
    pub message: Vec<u8>,
}

impl ConsecutiveResponse {
    pub fn into_raw_responses(&self) -> Vec<RawResponse> {
        let message = &self.message;
        info!("self.message.len(): {}", self.message.len());
        let message_len = message.len();

        let mut ret = Vec::new();
        let mut first_frame = [0; 8];

        first_frame[0] = 0x10 | ((message_len >> 8) as u8);
        first_frame[1] = (message_len & 0xFF) as u8;
        first_frame[2..].copy_from_slice(&message[..6]);

        ret.push(RawResponse {
            can_id: self.can_id,
            message: first_frame.to_vec(),
        });

        info!("First frame: {:x?}", first_frame);

        for (chunk_nr, chunk) in message[6..].chunks(7).enumerate() {
            let mut message = Vec::with_capacity(chunk.len() + 1);
            message.push(0x20 | ((chunk_nr + 1) as u8 & 0x0F));
            message.extend_from_slice(chunk);
            if message.len() < 8 {
                message.resize(8, 0);
            }
            ret.push(RawResponse {
                can_id: self.can_id,
                message,
            });
        }

        info!("Consecutive frames: {:x?}", ret);
        ret
    }
}
