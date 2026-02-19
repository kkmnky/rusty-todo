use chrono::{DateTime, Utc};
use kernel::model::{
    id::{TodoId, UserId},
    todo::Todo,
};
use shared::error::AppError;

pub struct TodoRow {
    pub id: TodoId,
    pub user_id: UserId,
    pub title: String,
    pub completed: bool,
    pub due_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<TodoRow> for Todo {
    type Error = AppError;

    fn try_from(value: TodoRow) -> Result<Self, Self::Error> {
        Ok(Todo {
            id: value.id,
            assignee_user_id: value.user_id,
            title: value.title,
            completed: value.completed,
            due_at: value.due_at,
        })
    }
}
