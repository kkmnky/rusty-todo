use std::sync::Arc;

use shared::error::AppResult;

use crate::{
    model::{id::UserId, todo::Todo},
    repository::todo::TodoRepository,
};

pub struct ListMyTodosUsecase {
    todo_repository: Arc<dyn TodoRepository>,
}

impl ListMyTodosUsecase {
    pub fn new(todo_repository: Arc<dyn TodoRepository>) -> Self {
        Self { todo_repository }
    }

    pub async fn execute(&self, user_id: UserId) -> AppResult<Vec<Todo>> {
        self.todo_repository.find_by_user_id(user_id).await
    }
}
