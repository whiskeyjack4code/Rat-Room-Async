use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "set_username")]
    SetUsername { username: String },

    #[serde(rename = "chat")]
    Chat { message: String },

    #[serde(rename = "join_room")]
    JoinRoom { room: String },
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
    Chat { username: String, room: String, message: String },

    #[serde(rename = "room_joined")]
    RoomJoined { room: String },
}