use serde::Deserialize;
use serde::Serialize;

use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Config {
    pub configs: BTreeMap<String, Vec<Request>>,
}

impl Config {
    pub fn new() -> Self {
        let mut ret = Config {
            configs: BTreeMap::new(),
        };

        std::fs::read_dir("configs").unwrap().for_each(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            let name = path.file_name().unwrap().to_str().unwrap().to_string();
            let content = std::fs::read_to_string(path).unwrap();
            let config: Vec<Request> = ron::from_str(&content).unwrap();
            ret.configs.insert(name, config);
        });

        ret
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
pub struct Response {
    pub can_id: u32,
    pub message: Vec<u8>,
}
