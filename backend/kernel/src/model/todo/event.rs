use sqlx::types::chrono::{DateTime, Utc};

use crate::model::id::UserId;

pub struct CreateTodo {
    pub title: String,
    pub assignee_user_id: UserId,
    pub due_at: Option<DateTime<Utc>>,
}
