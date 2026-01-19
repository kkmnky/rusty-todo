use crate::model::user::{User, event::CreateUser};
use async_trait::async_trait;
use shared::error::AppResult;

#[mockall::automock]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, event: CreateUser) -> AppResult<User>;
    async fn find_all(&self) -> AppResult<Vec<User>>;
}
