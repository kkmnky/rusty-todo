use shared::error::AppError;

pub mod auth;

pub trait RedisKey {
    type Value: RedisValue + TryFrom<String, Error = AppError>;
    fn inner(&self) -> String;
}

pub trait RedisValue {
    fn inner(&self) -> String;
}
