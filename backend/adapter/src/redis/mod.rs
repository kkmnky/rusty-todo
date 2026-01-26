use redis::{AsyncCommands, Client};
use shared::{config::RedisConfig, error::AppResult};

use crate::redis::model::{RedisKey, RedisValue};

pub mod model;

pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    pub fn new(config: &RedisConfig) -> AppResult<Self> {
        let client = Client::open(format!("redis://{}:{}", config.host, config.port))?;
        Ok(Self { client })
    }

    pub async fn set_ex<T: RedisKey>(&self, key: &T, value: &T::Value, ttl: u64) -> AppResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.set_ex(key.inner(), value.inner(), ttl).await?;
        Ok(())
    }

    pub async fn get<T: RedisKey>(&self, key: &T) -> AppResult<Option<T::Value>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let result: Option<String> = conn.get(key.inner()).await?;
        result.map(T::Value::try_from).transpose()
    }

    pub async fn ttl<T: RedisKey>(&self, key: &T) -> AppResult<i64> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let ttl: i64 = conn.ttl(key.inner()).await?;
        Ok(ttl)
    }

    pub async fn delete<T: RedisKey>(&self, key: &T) -> AppResult<i64> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let deleted_count: i64 = conn.del(key.inner()).await?;
        Ok(deleted_count)
    }
}
