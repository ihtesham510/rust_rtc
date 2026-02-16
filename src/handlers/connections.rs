use crate::{app_state::AppState, handlers, types::ClientMessages};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::unbounded_channel;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};
use uuid::Uuid;

pub async fn handle_connection(stream: TcpStream, app_state: AppState) {
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
        let mut conn = app_state.connections.lock().await;
        conn.insert(user_id.clone(), tx.clone());
    }

    let user_id_clone = user_id.clone();
    let connections_clone = app_state.connections.clone();
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
    while let Some(message_result) = read.next().await {
        match message_result {
            Ok(Message::Text(text)) => {
                info!("Received from {}: {}", user_id, text);
                let result = serde_json::from_str::<ClientMessages>(&text);
                match result {
                    Ok(message) => match message {
                        ClientMessages::Info => {
                            let _ = tx.send(Message::Text(
                                serde_json::json!({
                                    "type":"info",
                                    "user_id":user_id.clone(),
                                })
                                .to_string()
                                .into(),
                            ));
                        }

                        ClientMessages::Join { room } => {
                            handlers::room::join(&app_state, &user_id, &room, &tx).await;
                        }
                        ClientMessages::Create { room_name } => {
                            handlers::room::create(&app_state, &user_id, &room_name, &tx).await;
                        }
                        ClientMessages::GetRooms => {
                            handlers::room::get(&app_state, &tx).await;
                        }
                        ClientMessages::SendMessageToRoom { message, room } => {
                            handlers::room::broadcast(
                                &app_state,
                                message.clone(),
                                room,
                                user_id.clone(),
                            )
                            .await
                        }
                        ClientMessages::ListRoomMessages { room } => {
                            handlers::room::list_messages(&app_state, &room, &tx).await;
                        }
                        ClientMessages::RoomDetails { room } => {
                            handlers::room::details(&app_state, &tx, &room).await;
                        }
                        ClientMessages::LeaveRoom { room, user } => {
                            handlers::room::leave_room(&app_state, &tx, room, user).await;
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

    let mut connections_guard = app_state.connections.lock().await;
    app_state.remove_from_rooms(user_id.clone()).await;
    connections_guard.remove(&user_id);
    info!("Removed user {} from connections", user_id);

    {
        let rooms = app_state.get_rooms().await;
        for mut room in rooms {
            room.users.retain(|id| id != &user_id);
        }
    }

    info!("Connection ended for user: {}", user_id);
}
