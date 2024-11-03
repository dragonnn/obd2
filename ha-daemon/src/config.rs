use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ha::device::RegisterSensor;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub ha: HaConfig,
    pub kia: KiaConfig,
    pub sensors: HashMap<String, HaSensor>,
}

impl Config {
    pub fn load() -> Self {
        let config = std::fs::read_to_string("Config.toml").unwrap();
        toml::from_str(&config).unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HaConfig {
    pub host: String,
    pub auth: String,
    pub device_id: String,
    pub push_notification_key: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KiaConfig {
    pub port: u16,
    pub timeout: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HaSensor {
    pub ha: RegisterSensor,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum HaSensorHttpMethod {
    Get,
    Post,
    Put,
    Delete,
}
