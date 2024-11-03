#[derive(Serialize, Deserialize, Debug)]
struct AppData {
    push_notification_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeviceInfo {
    device_id: String,
    app_id: String,
    app_name: String,
    app_version: String,
    device_name: String,
    manufacturer: String,
    model: String,
    os_name: String,
    os_version: String,
    supports_encryption: bool,
    app_data: AppData,
}

#[derive(Serialize, Deserialize, Debug)]
struct WebhookInfo {
    cloudhook_url: String,
    remote_ui_url: String,
    secret: String,
    webhook_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthRequired {
    r#type: String,
    ha_version: String,
}

pub enum AuthState {
    Required,
    Ok,
    Invalid,
}

#[derive(Serialize, Deserialize, Debug)]
struct Auth {
    r#type: String,
    access_token: String,
}
