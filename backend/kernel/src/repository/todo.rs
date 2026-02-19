use crate::model::{
    id::{TodoId, UserId},
    todo::{Todo, event::CreateTodo},
};
use async_trait::async_trait;
use shared::error::AppResult;

#[mockall::automock]
#[async_trait]
pub trait TodoRepository: Send + Sync {
    async fn create(&self, event: CreateTodo) -> AppResult<Todo>;
    async fn find_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Todo>>;
    async fn update_completed(&self, todo_id: TodoId, completed: bool) -> AppResult<()>;
}
