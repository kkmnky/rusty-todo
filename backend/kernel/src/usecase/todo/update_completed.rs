use std::sync::Arc;

use shared::error::AppResult;

use crate::{
    model::{
        id::TodoId,
        todo::{Todo, event::UpdateTodoCompleted},
    },
    repository::todo::TodoRepository,
};

pub struct UpdateTodoCompletedInput {
    pub id: TodoId,
    pub completed: bool,
}

impl From<UpdateTodoCompletedInput> for UpdateTodoCompleted {
    fn from(value: UpdateTodoCompletedInput) -> Self {
        Self {
            id: value.id,
            completed: value.completed,
        }
    }
}

pub struct UpdateTodoCompletedUsecase {
    todo_repository: Arc<dyn TodoRepository>,
}

impl UpdateTodoCompletedUsecase {
    pub fn new(todo_repository: Arc<dyn TodoRepository>) -> Self {
        Self { todo_repository }
    }

    pub async fn execute(&self, input: UpdateTodoCompletedInput) -> AppResult<Todo> {
        let todo = self.todo_repository.update_completed(input.into()).await?;
        Ok(todo)
    }
}
