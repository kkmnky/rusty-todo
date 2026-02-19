use derive_new::new;
use garde::Validate;
use kernel::{
    model::{
        id::{TodoId, UserId},
        todo::Todo,
    },
    usecase::todo::register::RegisterTodoInput,
};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, Serialize)]
pub struct TodoResponse {
    pub id: TodoId,
    pub assignee_user_id: UserId,
    pub title: String,
    pub completed: bool,
    pub due_at: Option<DateTime<Utc>>,
}

impl From<Todo> for TodoResponse {
    fn from(value: Todo) -> Self {
        let Todo {
            id,
            assignee_user_id,
            title,
            completed,
            due_at,
        } = value;
        Self {
            id,
            assignee_user_id,
            title,
            completed,
            due_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TodosResponse {
    pub items: Vec<TodoResponse>,
}

#[derive(Deserialize, Validate, new)]
#[serde(rename_all = "camelCase")]
pub struct RegisterTodoRequest {
    #[garde(length(min = 1))]
    title: String,
    #[garde(skip)]
    assignee_user_id: UserId,
    #[garde(skip)]
    due_at: Option<DateTime<Utc>>,
}

impl From<RegisterTodoRequest> for RegisterTodoInput {
    fn from(value: RegisterTodoRequest) -> Self {
        let RegisterTodoRequest {
            title,
            assignee_user_id,
            due_at,
        } = value;
        Self {
            title,
            assignee_user_id,
            due_at,
        }
    }
}
