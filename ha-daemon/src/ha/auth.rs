use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct IncomingAuth {
    pub r#type: AuthState,
    pub ha_version: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OutgoingAuth {
    r#type: String,
    access_token: String,
}

impl OutgoingAuth {
    pub fn new(access_token: String) -> Self {
        Self {
            r#type: "auth".to_owned(),
            access_token,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum AuthState {
    #[serde(rename = "auth_required")]
    Required,
    #[serde(rename = "auth_ok")]
    Ok,
    #[serde(rename = "auth_invalid")]
    Invalid,
}
