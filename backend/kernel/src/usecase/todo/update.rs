use std::sync::Arc;

use shared::error::{AppError, AppResult};
use sqlx::types::chrono::{DateTime, Utc};

use crate::{
    model::{
        id::{TodoId, UserId},
        todo::{Todo, event::UpdateTodo},
    },
    repository::todo::TodoRepository,
};

pub struct UpdateTodoInput {
    pub id: TodoId,
    pub title: Option<String>,
    pub assignee_user_id: Option<UserId>,
    pub due_at: Option<Option<DateTime<Utc>>>,
}

pub struct UpdateTodoUsecase {
    todo_repository: Arc<dyn TodoRepository>,
}

impl UpdateTodoUsecase {
    pub fn new(todo_repository: Arc<dyn TodoRepository>) -> Self {
        Self { todo_repository }
    }

    pub async fn execute(&self, input: UpdateTodoInput) -> AppResult<Todo> {
        let old_todo = self
            .todo_repository
            .find_by_id(input.id)
            .await?
            .ok_or_else(|| AppError::EntityNotFoundError("Todo not found".into()))?;

        let updated_todo = self
            .todo_repository
            .update(UpdateTodo {
                id: input.id,
                title: input.title.unwrap_or(old_todo.title),
                assignee_user_id: input.assignee_user_id.unwrap_or(old_todo.assignee_user_id),
                due_at: input.due_at.unwrap_or(old_todo.due_at),
            })
            .await?;

        Ok(updated_todo)
    }
}
