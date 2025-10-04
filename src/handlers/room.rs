use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::types::{AppState, Room, RoomMessages};

pub async fn join(
    app_state: &AppState,
    user_id: &String,
    room: &String,
    tx: &UnboundedSender<Message>,
) {
    info!("User {} requesting to join room: {}", user_id, room);
    let mut rooms_guard = app_state.rooms.lock().await;
    if let Some(room_data) = rooms_guard.get_mut(room) {
        if !room_data.users.contains(&user_id) {
            room_data.users.push(user_id.clone());
        }

        let room_id = room_data.room.clone();
        let response = serde_json::json!({
            "type": "room_joined",
            "room_id": room_id,
            "room_name": room_data.room_name
        });

        if let Err(e) = tx.send(Message::Text(response.to_string().into())) {
            error!("Error while sending room_id {}: {:?}", room_id, e);
        }

        info!("User {} joined room {}", user_id, room_id);
    } else {
        error!("Room {} not found", room);
        let error_response = serde_json::json!({
            "type": "error",
            "message": "Room not found"
        });
        if let Err(e) = tx.send(Message::Text(error_response.to_string().into())) {
            error!("Error while sending error message: {:?}", e);
        }
    }
}
pub async fn create(
    app_state: &AppState,
    user_id: &String,
    room_name: &String,
    tx: &UnboundedSender<Message>,
) {
    info!("User {} creating room: {}", user_id, room_name);

    let room_id = Uuid::new_v4().to_string();
    let room = Room {
        room_name: room_name.clone(),
        room: room_id.clone(),
        users: vec![user_id.clone()],
        admin: user_id.clone(),
    };

    {
        let mut rooms_guard = app_state.rooms.lock().await;
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

    let connections_guard = app_state.connections.lock().await;
    for (id, client_tx) in connections_guard.iter() {
        if *id != user_id.clone() {
            let _ = client_tx.send(Message::Text(broadcast_msg.to_string().into()));
        }
    }

    info!("Room created: {} ({})", room_name, room_id);
}

pub async fn get(app_state: &AppState, tx: &UnboundedSender<Message>) {
    let mut _rooms: Vec<Room> = vec![];
    for (_, room) in app_state.rooms.lock().await.iter() {
        _rooms.push(room.clone());
    }
    let message = serde_json::to_string(&_rooms.clone()).unwrap();
    tx.send(Message::Text(message.into())).unwrap();
}

pub async fn broadcast(app_state: &AppState, message: String, room: String, by: String) {
    let connections_guard = app_state.connections.lock().await;
    let rooms_guard = app_state.rooms.lock().await;
    let mut room_messages_guard = app_state.messages.lock().await;
    let _room = rooms_guard.get(&room.clone()).unwrap();

    {
        let list = room_messages_guard
            .entry(room.clone())
            .or_insert_with(Vec::new);

        list.push(RoomMessages {
            by: by.clone(),
            message: message.clone(),
        });
    }

    for user in _room.users.iter() {
        if let Some(tx) = connections_guard.get(user) {
            if let Err(err) = tx.send(tokio_tungstenite::tungstenite::Message::Text(
                serde_json::json!({
                    "type":"room_broadcast",
                    "by":by,
                    "message":message
                })
                .to_string()
                .into(),
            )) {
                error!("Error while sending message to {user:?}: {err:?}")
            }
        } else {
            warn!("User {user:?} not found in connections");
        }
    }
}

pub async fn list_messages(app_state: &AppState, room: &String, tx: &UnboundedSender<Message>) {
    let room_messages_guard = app_state.messages.lock().await;
    if let Some(_list) = room_messages_guard.get(room) {
        info!("list room request received");
        let _ = tx.send(Message::Text(
            serde_json::json!({
                "type":"list_messages",
                "messages": _list.to_owned(),
            })
            .to_string()
            .into(),
        ));
    } else {
        let _ = tx.send(Message::Text(
            serde_json::json!({
                "type":"list_messages",
                "messages": [],
            })
            .to_string()
            .into(),
        ));
    }
}

pub async fn details(app_state: &AppState, tx: &UnboundedSender<Message>, room: &String) {
    let rooms_guard = app_state.rooms.lock().await;
    if let Some(_room) = rooms_guard.get(room) {
        tx.send(Message::Text(serde_json::to_string(_room).unwrap().into()))
            .unwrap();
    }
}
