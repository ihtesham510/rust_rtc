use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, mpsc::UnboundedSender};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessages {
    #[serde(rename = "join")]
    Join { room: String },
    #[serde(rename = "create_room")]
    Create { room_name: String },
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "get_rooms")]
    GetRooms,
    #[serde(rename = "send_message")]
    SendMessageToRoom { message: String, room: String },
    #[serde(rename = "list_messages")]
    ListRoomMessages { room: String },
    #[serde(rename = "get_room")]
    RoomDetails { room: String },
    #[serde(rename = "leave_room")]
    LeaveRoom { room: String, user: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Room {
    pub room_name: String,
    pub room: String,
    pub messages: Vec<RoomMessage>,
    pub users: Vec<String>,
    pub admin: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomMessage {
    pub by: String,
    pub message: String,
}

pub type Connections = Arc<Mutex<HashMap<String, UnboundedSender<Message>>>>;
