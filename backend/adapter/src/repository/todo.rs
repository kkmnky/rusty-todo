use async_trait::async_trait;
use derive_new::new;
use kernel::{
    model::{
        id::TodoId,
        todo::{Todo, event::CreateTodo},
    },
    repository::todo::TodoRepository,
};
use shared::error::{AppError, AppResult};

use crate::database::ConnectionPool;

#[derive(new)]
pub struct TodoRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl TodoRepository for TodoRepositoryImpl {
    async fn create(&self, event: CreateTodo) -> AppResult<Todo> {
        let todo_id = TodoId::new();

        let res = sqlx::query!(
            r#"--sql
                INSERT INTO todos (id, user_id, title, completed, due_at)
                SELECT $1, $2, $3, $4, $5
            "#,
            todo_id as _,
            event.user_id as _,
            event.title,
            false,
            event.due_at,
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SqlExecuteError)?;

        if res.rows_affected() == 0 {
            return Err(AppError::NoRowsAffectedError(
                "No todo has been created".into(),
            ));
        }

        Ok(Todo {
            id: todo_id,
            user_id: event.user_id,
            title: event.title,
            completed: false,
            due_at: event.due_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::ConnectionPool;
    use kernel::model::{id::UserId, todo::event::CreateTodo};
    use sqlx::{
        PgPool, Row,
        types::chrono::{DateTime, Utc},
    };
    use shared::error::AppError;
    use std::str::FromStr;

    #[sqlx::test(fixtures("common"))]
    async fn タスクが作成される(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());

        let title = "テストタスク".to_string();
        let user_id =
            UserId::from_str("75ef7d75-3b57-4f54-8e8e-fdb65738690c").expect("user_id取得");
        let event = CreateTodo {
            title: title.clone(),
            user_id,
            due_at: None,
        };

        let todo = repo.create(event).await.expect("作成が成功する");

        assert_eq!(todo.title, title);
        assert_eq!(todo.user_id, user_id);
        assert!(!todo.completed);
        assert!(todo.due_at.is_none());

        let row = sqlx::query("SELECT user_id, title, completed, due_at FROM todos WHERE id = $1")
            .bind(todo.id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let persisted_user_id: UserId = row.try_get("user_id").expect("user_id取得");
        let persisted_title: String = row.try_get("title").expect("title取得");
        let persisted_completed: bool = row.try_get("completed").expect("completed取得");
        let persisted_due_at: Option<DateTime<Utc>> = row.try_get("due_at").expect("due_at取得");

        assert_eq!(persisted_user_id, user_id);
        assert_eq!(persisted_title, todo.title);
        assert_eq!(persisted_completed, todo.completed);
        assert!(persisted_due_at.is_none());
    }

    #[sqlx::test]
    async fn タスク作成は存在しないユーザidで失敗する(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let event = CreateTodo {
            title: "存在しないユーザのタスク".to_string(),
            user_id: UserId::new(),
            due_at: None,
        };

        let err = repo
            .create(event)
            .await
            .expect_err("存在しないユーザIDでは失敗する");

        assert!(matches!(err, AppError::SqlExecuteError(_)));
    }
}
