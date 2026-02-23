use std::sync::Arc;

use shared::error::AppResult;

use crate::{
    model::{id::TodoId, todo::event::DeleteTodo},
    repository::todo::TodoRepository,
};

pub struct DeleteTodoInput {
    pub id: TodoId,
}

impl From<DeleteTodoInput> for DeleteTodo {
    fn from(value: DeleteTodoInput) -> Self {
        Self { id: value.id }
    }
}

pub struct DeleteTodoUsecase {
    todo_repository: Arc<dyn TodoRepository>,
}

impl DeleteTodoUsecase {
    pub fn new(todo_repository: Arc<dyn TodoRepository>) -> Self {
        Self { todo_repository }
    }

    pub async fn execute(&self, input: DeleteTodoInput) -> AppResult<()> {
        self.todo_repository.delete(input.into()).await?;
        Ok(())
    }
}
