use sqlx::types::chrono::{DateTime, Utc};

use crate::model::id::{TodoId, UserId};

pub struct CreateTodo {
    pub title: String,
    pub assignee_user_id: UserId,
    pub due_at: Option<DateTime<Utc>>,
}

pub struct UpdateTodoCompleted {
    pub id: TodoId,
    pub completed: bool,
}
