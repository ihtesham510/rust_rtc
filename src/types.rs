use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
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
    #[serde(rename = "get_clients")]
    GetClients,
    #[serde(rename = "get_rooms")]
    GetRooms,
    #[serde(rename = "send_message")]
    SendMessageToRoom { message: String, room: String },
    #[serde(rename = "list_messages")]
    ListRoomMessages { room: String },
    #[serde(rename = "get_room")]
    RoomDetails { room: String },
    #[serde(rename = "list_connections")]
    ListConn,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Room {
    pub room_name: String,
    pub room: String,
    pub users: Vec<String>,
    pub admin: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomMessages {
    pub by: String,
    pub message: String,
}

pub type Rooms = Arc<Mutex<HashMap<String, Room>>>;
pub type Messages = Arc<Mutex<HashMap<String, Vec<RoomMessages>>>>;
pub type Connections = Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>;

#[derive(Clone)]
pub struct AppState {
    pub rooms: Rooms,
    pub messages: Messages,
    pub connections: Connections,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
