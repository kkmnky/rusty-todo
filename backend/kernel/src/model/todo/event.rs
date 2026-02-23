use sqlx::types::chrono::{DateTime, Utc};

use crate::model::id::{TodoId, UserId};

pub struct CreateTodo {
    pub title: String,
    pub assignee_user_id: UserId,
    pub due_at: Option<DateTime<Utc>>,
}

pub struct UpdateTodo {
    pub id: TodoId,
    pub title: String,
    pub assignee_user_id: UserId,
    pub due_at: Option<DateTime<Utc>>,
}

impl UpdateTodo {
    pub fn new(
        id: TodoId,
        title: String,
        assignee_user_id: UserId,
        due_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            title,
            assignee_user_id,
            due_at,
        }
    }
}

pub struct UpdateTodoCompleted {
    pub id: TodoId,
    pub completed: bool,
}

pub struct DeleteTodo {
    pub id: TodoId,
}

impl DeleteTodo {
    pub fn new(id: TodoId) -> Self {
        Self { id }
    }
}
