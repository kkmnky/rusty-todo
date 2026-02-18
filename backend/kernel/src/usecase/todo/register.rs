use std::sync::Arc;

use shared::error::AppResult;
use sqlx::types::chrono::{DateTime, Utc};

use crate::{
    model::{
        id::UserId,
        todo::{Todo, event::CreateTodo},
    },
    repository::todo::TodoRepository,
};

pub struct RegisterTodoInput {
    pub title: String,
    pub assignee_user_id: UserId,
    pub due_at: Option<DateTime<Utc>>,
}

impl From<RegisterTodoInput> for CreateTodo {
    fn from(value: RegisterTodoInput) -> Self {
        Self {
            title: value.title,
            assignee_user_id: value.assignee_user_id,
            due_at: value.due_at,
        }
    }
}

pub struct RegisterTodoUsecase {
    todo_repository: Arc<dyn TodoRepository>,
}

impl RegisterTodoUsecase {
    pub fn new(todo_repository: Arc<dyn TodoRepository>) -> Self {
        Self { todo_repository }
    }

    pub async fn execute(&self, input: RegisterTodoInput) -> AppResult<Todo> {
        self.todo_repository.create(input.into()).await
    }
}
