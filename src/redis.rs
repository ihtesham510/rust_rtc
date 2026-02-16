use redis::{Client, Commands, RedisResult};
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};

#[derive(Clone)]
pub struct Redis {
    conn: Arc<Client>,
}

impl Redis {
    pub fn new() -> Self {
        let redis_url = env::var("REDIS_URL").expect("Error REDIS_URL not set");
        let conn = Arc::new(Client::open(redis_url).expect("Error while connecting to the redis"));
        Self { conn }
    }

    pub async fn _get<T>(&self, key: &str) -> RedisResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut client = self.conn.get_connection()?;
        let json_str: String = client.get(key)?;
        let value: T = serde_json::from_str(&json_str)
            .map_err(|_| redis::RedisError::from((redis::ErrorKind::Io, "JSON parse error")))?;
        Ok(value)
    }

    pub async fn _set<T>(&self, key: &str, value: &T) -> RedisResult<()>
    where
        T: Serialize,
    {
        let mut client = self.conn.get_connection()?;
        let json_str = serde_json::to_string(value)
            .map_err(|_| redis::RedisError::from((redis::ErrorKind::Io, "JSON serialize error")))?;
        let _: () = client.set(key, json_str)?;
        Ok(())
    }

    pub async fn _del(&self, key: &str) -> RedisResult<()> {
        let mut client = self.conn.get_connection()?;
        let _: () = client.del(key)?;
        Ok(())
    }

    // List operations
    pub async fn lpush<T>(&self, key: &str, value: &T) -> RedisResult<()>
    where
        T: Serialize,
    {
        let mut client = self.conn.get_connection()?;
        let json_str = serde_json::to_string(value)
            .map_err(|_| redis::RedisError::from((redis::ErrorKind::Io, "JSON serialize error")))?;
        let _: () = client.lpush(key, json_str)?;
        Ok(())
    }

    pub async fn _rpush<T>(&self, key: &str, value: &T) -> RedisResult<()>
    where
        T: Serialize,
    {
        let mut client = self.conn.get_connection()?;
        let json_str = serde_json::to_string(value)
            .map_err(|_| redis::RedisError::from((redis::ErrorKind::Io, "JSON serialize error")))?;
        let _: () = client.rpush(key, json_str)?;
        Ok(())
    }

    pub async fn lrange<T>(&self, key: &str, start: isize, stop: isize) -> RedisResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut client = self.conn.get_connection()?;
        let json_strings: Vec<String> = client.lrange(key, start, stop)?;

        let mut result = Vec::new();
        for json_str in json_strings {
            let value: T = serde_json::from_str(&json_str)
                .map_err(|_| redis::RedisError::from((redis::ErrorKind::Io, "JSON parse error")))?;
            result.push(value);
        }
        Ok(result)
    }

    pub async fn _llen(&self, key: &str) -> RedisResult<usize> {
        let mut client = self.conn.get_connection()?;
        let len: usize = client.llen(key)?;
        Ok(len)
    }

    pub async fn _lpop<T>(&self, key: &str) -> RedisResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut client = self.conn.get_connection()?;
        let json_str: String = client.lpop(key, None)?;
        let value: T = serde_json::from_str(&json_str)
            .map_err(|_| redis::RedisError::from((redis::ErrorKind::Io, "JSON parse error")))?;
        Ok(value)
    }

    pub async fn _rpop<T>(&self, key: &str) -> RedisResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut client = self.conn.get_connection()?;
        let json_str: String = client.rpop(key, None)?;
        let value: T = serde_json::from_str(&json_str)
            .map_err(|_| redis::RedisError::from((redis::ErrorKind::Io, "JSON parse error")))?;
        Ok(value)
    }

    pub async fn get_all<T>(&self, key: &str) -> RedisResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.lrange(key, 0, -1).await
    }
    pub async fn lset_delete(&self, key: &str, index: usize) -> RedisResult<()> {
        let mut client = self.conn.get_connection()?;
        let _: () = client.lset(key, index.try_into().unwrap(), "__DELETE__")?;
        let _: () = client.lrem(key, 1, "__DELETE__")?;
        Ok(())
    }
    pub async fn lset<T>(&self, key: &str, index: usize, value: &T) -> RedisResult<()>
    where
        T: Serialize,
    {
        let mut client = self.conn.get_connection()?;
        let json_str = serde_json::to_string(value)
            .map_err(|_| redis::RedisError::from((redis::ErrorKind::Io, "JSON serialize error")))?;
        let _: () = client.lset(key, index.try_into().unwrap(), json_str)?;
        Ok(())
    }
}
