use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Serialize, Deserialize, Debug)]
pub struct AppData {
    pub push_notification_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceInfo {
    pub device_id: String,
    pub app_id: String,
    pub app_name: String,
    pub app_version: String,
    pub device_name: String,
    pub manufacturer: String,
    pub model: String,
    pub os_name: String,
    pub os_version: String,
    pub supports_encryption: bool,
    pub app_data: AppData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookInfo {
    pub cloudhook_url: Option<String>,
    pub remote_ui_url: Option<String>,
    pub secret: Option<String>,
    pub webhook_id: String,
}

use std::collections::HashMap;

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct WebHookHandle {
    r#type: String,
    #[serde(rename = "id")]
    id: usize,
    webhook_id: String,
    method: String,
    #[serde_as(as = "DisplayFromStr")]
    body: WebHookBody,
    headers: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub enum WebHookBody {
    #[serde(rename = "register_sensor")]
    RegisterSensor(RegisterSensor),
    #[serde(rename = "update_sensor_states")]
    UpdateSensor(UpdateSensor),
}

use std::fmt;
impl fmt::Display for WebHookBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{}", json)
    }
}

use std::str::FromStr;
impl FromStr for WebHookBody {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterSensor {
    pub device_class: String,
    pub icon: String,
    pub name: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub unique_id: String,
    pub unit_of_measurement: String,
    pub state_class: String,
    pub entity_category: String,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateSensor {
    pub icon: String,
    pub state: serde_json::Value,
    pub r#type: String,
    pub unique_id: String,
}

impl WebHookHandle {
    pub fn register(webhook_id: String, register_sensor: RegisterSensor) -> Self {
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_owned(), "application/json".to_owned());
        Self {
            r#type: "webhook/handle".to_owned(),
            id: counter,
            webhook_id,
            method: "POST".to_owned(),
            body: WebHookBody::RegisterSensor(register_sensor),
            headers,
        }
    }

    pub fn update(webhook_id: String, update_sensor: UpdateSensor) -> Self {
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_owned(), "application/json".to_owned());
        Self {
            r#type: "webhook/handle".to_owned(),
            id: counter,
            webhook_id,
            method: "POST".to_owned(),
            body: WebHookBody::UpdateSensor(update_sensor),
            headers,
        }
    }
}
