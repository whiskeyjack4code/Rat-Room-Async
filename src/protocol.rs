use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "set_username")]
    SetUsername { username: String },

    #[serde(rename = "chat")]
    Chat { message: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "welcome")]
    Welcome { message: String },

    #[serde(rename = "error")]
    Error { message: String },

    #[serde(rename = "system")]
    System { message: String },

    #[serde(rename = "chat")]
    Chat { username: String, message: String },
}