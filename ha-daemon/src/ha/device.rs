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
    #[serde(rename = "update_location")]
    UpdateLocation(UpdateLocation),
}

#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct UpdateLocation {
    pub gps: (f64, f64),
    pub gps_accuracy: i32,
    pub battery: u8,
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
    pub device_class: Option<String>,
    pub icon: String,
    pub name: String,
    #[serde(default)]
    #[serde(serialize_with = "serialize_option_with_round_down")]
    pub state: Option<serde_json::Value>,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub unique_id: String,
    pub unit_of_measurement: Option<String>,
    pub state_class: Option<String>,
    pub entity_category: String,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateSensor {
    pub icon: String,
    #[serde(serialize_with = "serialize_with_round_down")]
    pub state: serde_json::Value,
    pub r#type: String,
    pub unique_id: String,
}

fn serialize_option_with_round_down<S>(
    value: &Option<serde_json::Value>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if let Some(value) = value {
        serialize_with_round_down(value, serializer)
    } else {
        serializer.serialize_none()
    }
}

fn serialize_with_round_down<S>(value: &serde_json::Value, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    fn round_down_to_precision(value: f64, precision: usize) -> f64 {
        let factor = 10f64.powi(precision as i32);
        (value * factor).floor() / factor
    }

    if let serde_json::Value::Number(n) = value {
        if let Some(n) = n.as_f64() {
            serializer.serialize_f64(round_down_to_precision(n, 2))
        } else {
            serializer.serialize_none()
        }
    } else {
        serializer.serialize_some(value)
    }
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

    pub fn update_location(webhook_id: String, update_location: UpdateLocation) -> Self {
        let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_owned(), "application/json".to_owned());
        Self {
            r#type: "webhook/handle".to_owned(),
            id: counter,
            webhook_id,
            method: "POST".to_owned(),
            body: WebHookBody::UpdateLocation(update_location),
            headers,
        }
    }
}
