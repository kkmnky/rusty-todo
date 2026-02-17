use crate::model::todo::{Todo, event::CreateTodo};
use async_trait::async_trait;
use shared::error::AppResult;

#[mockall::automock]
#[async_trait]
pub trait TodoRepository: Send + Sync {
    async fn create(&self, event: CreateTodo) -> AppResult<Todo>;
}
