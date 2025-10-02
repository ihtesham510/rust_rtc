use std::{collections::HashMap, sync::Arc};

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::sync::mpsc::unbounded_channel;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum ClientMessages {
    #[serde(rename = "join")]
    Join { room: String },
    #[serde(rename = "create_room")]
    Create { room_name: String },
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "get_rooms")]
    GetRooms,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Room {
    room_name: String,
    room: String,
    users: Vec<String>,
    admin: String,
}

type Rooms = Arc<Mutex<HashMap<String, Room>>>;
type Connections = Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let addr = "127.0.0.1:4000";
    let listener = TcpListener::bind(addr).await?;
    info!("WebSocket server running on ws://{}", addr);

    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));
    let connections: Connections = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, addr)) = listener.accept().await {
        info!("New connection from: {}", addr);
        tokio::spawn(handle_connection(
            stream,
            rooms.clone(),
            connections.clone(),
        ));
    }

    Ok(())
}

async fn broadcast_to_all(connections: &Connections, message: Message) {
    let connections_guard = connections.lock().await;
    let mut failed_users = Vec::new();

    for (user_id, tx) in connections_guard.iter() {
        if let Err(e) = tx.send(message.clone()) {
            error!("Failed to broadcast to user {}: {:?}", user_id, e);
            failed_users.push(user_id.clone());
        }
    }

    if !failed_users.is_empty() {
        info!("Failed to send to {} users", failed_users.len());
    }
}
async fn broadcast_to_room(
    connections: &Connections,
    message: Message,
    rooms: &Rooms,
    room: String,
) {
    let connections_guard = connections.lock().await;
    let rooms_guard = rooms.lock().await;
    let _room = rooms_guard.get(&room.clone()).unwrap();
    for user in _room.users.iter() {
        let tx = connections_guard.get(user).unwrap();
        match tx.send(message.clone()) {
            Ok(_) => return,
            Err(err) => error!("Error while sending message : {err:?}"),
        }
    }
}

async fn handle_connection(stream: TcpStream, rooms: Rooms, connections: Connections) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => {
            info!("WebSocket handshake successful");
            ws
        }
        Err(e) => {
            info!("Failed to accept WebSocket: {}", e);
            return;
        }
    };

    let user_id = Uuid::new_v4().to_string();
    info!("User connected: {}", user_id);

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = unbounded_channel::<Message>();

    {
        let mut conn = connections.lock().await;
        conn.insert(user_id.clone(), tx.clone());
    }

    // Spawn task to handle outgoing messages
    let user_id_clone = user_id.clone();
    let connections_clone = connections.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(err) = write.send(msg).await {
                error!(
                    "Error while sending message to {}: {:?}",
                    user_id_clone, err
                );
                connections_clone.lock().await.remove(&user_id_clone);
                break;
            }
        }
    });

    // Send welcome message
    if let Err(e) = tx.send(Message::Text(
        "Welcome to the WebSocket server!".to_string().into(),
    )) {
        error!("Failed to send welcome message: {}", e);
        connections.lock().await.remove(&user_id);
        return;
    }

    while let Some(message_result) = read.next().await {
        match message_result {
            Ok(Message::Text(text)) => {
                info!("Received from {}: {}", user_id, text);
                let result = serde_json::from_str::<ClientMessages>(&text);
                match result {
                    Ok(message) => match message {
                        ClientMessages::Join { room } => {
                            info!("User {} requesting to join room: {}", user_id, room);
                            let mut rooms_guard = rooms.lock().await;
                            if let Some(room_data) = rooms_guard.get_mut(&room) {
                                if !room_data.users.contains(&user_id) {
                                    room_data.users.push(user_id.clone());
                                }

                                let room_id = room_data.room.clone();
                                let response = serde_json::json!({
                                    "type": "room_joined",
                                    "room_id": room_id,
                                    "room_name": room_data.room_name
                                });

                                if let Err(e) = tx.send(Message::Text(response.to_string().into()))
                                {
                                    error!("Error while sending room_id {}: {:?}", room_id, e);
                                }

                                info!("User {} joined room {}", user_id, room_id);
                            } else {
                                error!("Room {} not found", room);
                                let error_response = serde_json::json!({
                                    "type": "error",
                                    "message": "Room not found"
                                });
                                if let Err(e) =
                                    tx.send(Message::Text(error_response.to_string().into()))
                                {
                                    error!("Error while sending error message: {:?}", e);
                                }
                            }
                        }
                        ClientMessages::Create { room_name } => {
                            info!("User {} creating room: {}", user_id, room_name);

                            let room_id = Uuid::new_v4().to_string();
                            let room = Room {
                                room_name: room_name.clone(),
                                room: room_id.clone(),
                                users: vec![user_id.clone()],
                                admin: user_id.clone(),
                            };

                            {
                                let mut rooms_guard = rooms.lock().await;
                                rooms_guard.insert(room_id.clone(), room);
                            }

                            let response = serde_json::json!({
                                "type": "room_created",
                                "room_id": room_id,
                                "room_name": room_name
                            });

                            if let Err(e) = tx.send(Message::Text(response.to_string().into())) {
                                error!("Error while sending room_id {}: {:?}", room_id, e);
                            }

                            let broadcast_msg = serde_json::json!({
                                "type": "room_available",
                                "room_id": room_id,
                                "room_name": room_name
                            });

                            let connections_guard = connections.lock().await;
                            for (id, client_tx) in connections_guard.iter() {
                                if *id != user_id {
                                    let _ = client_tx
                                        .send(Message::Text(broadcast_msg.to_string().into()));
                                }
                            }

                            info!("Room created: {} ({})", room_name, room_id);
                        }
                        ClientMessages::Info => {
                            let _ = tx.send(Message::Text(user_id.clone().into()));
                        }
                        ClientMessages::GetRooms => {
                            let mut _rooms: Vec<Room> = vec![];
                            for (_, room) in rooms.lock().await.iter() {
                                _rooms.push(room.clone());
                            }
                            let message = serde_json::to_string(&_rooms.clone()).unwrap();
                            tx.send(Message::Text(message.into())).unwrap();
                        }
                    },
                    Err(e) => {
                        warn!("Invalid message received from {}: {}", user_id, e);
                        let echo = Message::Text(format!("Echo: {}", text).into());
                        if let Err(e) = tx.send(echo) {
                            error!("Failed to send echo: {}", e);
                            break;
                        }
                    }
                }
            }
            Ok(Message::Binary(data)) => {
                info!(
                    "Received binary data from {}: {} bytes",
                    user_id,
                    data.len()
                );
            }
            Ok(Message::Ping(data)) => {
                info!("Received ping from {}", user_id);
                if let Err(e) = tx.send(Message::Pong(data)) {
                    info!("Failed to send pong: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                info!("Connection closed by client: {}", user_id);
                break;
            }
            Ok(_) => {}
            Err(e) => {
                error!("WebSocket error for {}: {}", user_id, e);
                break;
            }
        }
    }

    {
        let mut connections_guard = connections.lock().await;
        connections_guard.remove(&user_id);
        info!("Removed user {} from connections", user_id);
    }

    {
        let mut rooms_guard = rooms.lock().await;
        for room in rooms_guard.values_mut() {
            room.users.retain(|id| id != &user_id);
        }
    }

    info!("Connection ended for user: {}", user_id);
}
