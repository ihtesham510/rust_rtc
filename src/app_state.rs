use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tracing::error;

use crate::{
    redis::Redis,
    types::{Connections, Room, RoomMessage},
};

#[derive(Clone)]
pub struct AppState {
    pub redis: Redis,
    pub connections: Connections,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            redis: Redis::new(),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn _delete_users_rooms(&self, user: String) {
        let rooms = self.get_rooms().await;
        for room in rooms {
            if room.admin == user {
                self.del_room(room.room).await;
            }
        }
    }
    pub async fn add_to_room(&self, room: String, user: String) -> Room {
        let mut r = self.get_room(room.clone()).await.clone();
        let i = self.get_room_index(room).await;
        r.users.push(user);
        self.redis.lset("rooms", i, &r.clone()).await.unwrap();
        r
    }
    pub async fn remove_from_room(&self, room: String, user: String) -> Option<Room> {
        let mut r = self.get_room(room.clone()).await.clone();
        let i = self.get_room_index(room.clone()).await;

        if r.users.len() == 1 && r.admin == user {
            self.del_room(room).await;
            return None;
        }

        let user_index = r.users.iter().position(|u| u.to_owned() == user);
        match user_index {
            Some(index) => {
                if r.admin == user {
                    r.users.remove(index);
                    r.admin = r.users.iter().next().unwrap().to_owned();
                } else {
                    r.users.remove(index);
                }
                self.redis
                    .lset("rooms", i, &r.clone())
                    .await
                    .expect("Error while setting room");
            }
            None => {
                error!("User not found in the room");
            }
        }

        return Some(r);
    }
    pub async fn add_message(&self, room: String, message: RoomMessage) {
        let mut r = self.get_room(room.clone()).await.clone();
        let i = self.get_room_index(room).await;
        r.messages.push(message);
        self.redis.lset("rooms", i, &r).await.unwrap();
    }
    pub async fn get_room(&self, room: String) -> Room {
        let rooms = self.get_rooms().await;
        let i = rooms
            .iter()
            .position(|r| r.room == room)
            .expect("Room index not found");
        rooms.get(i).expect("Error room not found").clone()
    }
    pub async fn get_room_index(&self, room: String) -> usize {
        let rooms = self.get_rooms().await;
        rooms
            .iter()
            .position(|r| r.room == room)
            .expect("error room not found")
    }
    pub async fn _get_connections(&self) -> Vec<String> {
        let connection_guard = self.connections.lock().await;
        let mut vec = Vec::new();
        for conn in connection_guard.iter() {
            let (user, _itx) = conn;
            vec.push(user.clone());
        }
        vec
    }
    pub async fn remove_from_rooms(&self, user: String) {
        let rooms = self.get_rooms().await;
        for room in rooms.iter() {
            if room.users.iter().find(|u| **u == user.clone()).is_some() {
                self.remove_from_room(room.room.clone(), user.clone()).await;
            }
        }
    }
    pub async fn get_rooms(&self) -> Vec<Room> {
        let res: Vec<Room> = self.redis.get_all("rooms").await.unwrap();
        res
    }
    pub async fn create_room(&self, room: Room) {
        self.redis.lpush("rooms", &room).await.unwrap();
    }
    pub async fn del_room(&self, room: String) {
        let rooms = self.get_rooms().await;
        let index = rooms.iter().position(|r| r.room == room);
        match index {
            Some(i) => self.redis.lset_delete("rooms", i).await.unwrap(),
            None => error!("room not found"),
        }
    }
}
