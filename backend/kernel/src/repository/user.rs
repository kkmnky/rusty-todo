use crate::model::{
    id::UserId,
    user::{
        User,
        event::{CreateUser, DeleteUser},
    },
};
use async_trait::async_trait;
use shared::error::AppResult;

#[mockall::automock]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, event: CreateUser) -> AppResult<User>;
    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>>;
    async fn find_all(&self) -> AppResult<Vec<User>>;
    async fn delete(&self, event: DeleteUser) -> AppResult<()>;
}
