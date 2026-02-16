use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    types::{Room, RoomMessage},
};

pub async fn join(
    app_state: &AppState,
    user_id: &String,
    room: &String,
    tx: &UnboundedSender<Message>,
) {
    let room_data = app_state
        .add_to_room(room.to_owned(), user_id.to_owned())
        .await;
    let room_id = room_data.room.clone();
    let response = serde_json::json!({
        "type": "room_joined",
        "room_id": room_id,
        "room_name": room_data.room_name
    });

    if let Err(e) = tx.send(Message::Text(response.to_string().into())) {
        error!("Error while sending room_id {}: {:?}", room_id, e);
    }
}
pub async fn create(
    app_state: &AppState,
    user_id: &String,
    room_name: &String,
    tx: &UnboundedSender<Message>,
) {
    let room_id = Uuid::new_v4().to_string();
    let room = Room {
        room_name: room_name.clone(),
        room: room_id.clone(),
        messages: vec![],
        users: vec![user_id.clone()],
        admin: user_id.clone(),
    };
    app_state.create_room(room).await;

    let response = serde_json::json!({
        "type": "room_created",
        "room_id": room_id,
        "room_name": room_name
    });

    if let Err(e) = tx.send(Message::Text(response.to_string().into())) {
        error!("Error while sending room_id {}: {:?}", room_id, e);
    }

    info!("User {} creating room: {}", user_id, room_name);

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

pub async fn broadcast_to_all(app_state: &AppState, message: String) {
    let connections_guard = app_state.connections.lock().await;
    for conn in connections_guard.iter() {
        let (_, tx) = conn;
        if let Err(err) = tx.send(Message::Text(message.clone().into())) {
            error!("Error while sending message {}", err.to_string());
        }
    }
}

pub async fn get(app_state: &AppState, tx: &UnboundedSender<Message>) {
    let mut _rooms: Vec<Room> = app_state.get_rooms().await;
    let message = serde_json::to_string(&_rooms.clone()).unwrap();
    tx.send(Message::Text(message.into())).unwrap();
}

pub async fn broadcast_message(app_state: &AppState, message: String, room: String, by: String) {
    let connections_guard = app_state.connections.lock().await;
    let _room = app_state.get_room(room.clone()).await;
    let room_message = RoomMessage {
        by: by.clone(),
        message: message.clone(),
    };
    app_state.add_message(room.clone(), room_message).await;

    for user in _room.users.iter() {
        if let Some(tx) = connections_guard.get(user) {
            if let Err(err) = tx.send(Message::Text(
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
    info!("list room request received");
    let _room = app_state.get_room(room.to_owned()).await;
    let _list = _room.messages;
    let _ = tx
        .send(Message::Text(
            serde_json::json!({
                "type":"list_messages",
                "messages": _list.to_owned(),
            })
            .to_string()
            .into(),
        ))
        .unwrap();
}

pub async fn details(app_state: &AppState, tx: &UnboundedSender<Message>, room: &String) {
    let _room = app_state.get_room(room.to_owned()).await;
    tx.send(Message::Text(serde_json::to_string(&_room).unwrap().into()))
        .unwrap();
}

pub async fn leave_room(
    app_state: &AppState,
    tx: &UnboundedSender<Message>,
    room: String,
    user: String,
) {
    let room = app_state.remove_from_room(room, user).await;
    match room {
        Some(r) => {
            if let Err(err) = tx.send(Message::Text(
                serde_json::json!({
                    "type":"room_left",
                    "room":r.room
                })
                .to_string()
                .into(),
            )) {
                error!("Error while sending message {}", err.to_string());
            }
        }
        None => {
            let rooms = app_state.get_rooms().await;
            broadcast_to_all(
                app_state,
                serde_json::json!({
                    "type":"rooms_available",
                    "rooms":rooms
                })
                .to_string(),
            )
            .await;
        }
    }
}
