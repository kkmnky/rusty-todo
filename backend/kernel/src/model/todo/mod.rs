use sqlx::types::chrono::{DateTime, Utc};

use crate::model::id::{TodoId, UserId};

pub mod event;

#[derive(Debug)]
pub struct Todo {
    pub id: TodoId,
    pub user_id: UserId,
    pub title: String,
    pub completed: bool,
    pub due_at: Option<DateTime<Utc>>,
}
