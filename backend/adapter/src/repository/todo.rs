use async_trait::async_trait;
use derive_new::new;
use kernel::{
    model::{
        id::{TodoId, UserId},
        todo::{Todo, event::CreateTodo},
    },
    repository::todo::TodoRepository,
};
use shared::error::{AppError, AppResult};

use crate::database::{ConnectionPool, model::todo::TodoRow};

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
            event.assignee_user_id as _,
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
            assignee_user_id: event.assignee_user_id,
            title: event.title,
            completed: false,
            due_at: event.due_at,
        })
    }

    async fn find_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Todo>> {
        let rows = sqlx::query_as!(
            TodoRow,
            r#"--sql
                SELECT
                    id,
                    user_id,
                    title,
                    completed,
                    due_at,
                    created_at,
                    updated_at
                FROM todos
                WHERE user_id = $1
                ORDER BY created_at DESC
            "#,
            user_id as _,
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SqlExecuteError)?;

        let todos = rows
            .into_iter()
            .map(Todo::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(todos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::ConnectionPool;
    use kernel::model::{
        id::{TodoId, UserId},
        todo::event::CreateTodo,
    };
    use shared::error::AppError;
    use sqlx::{
        PgPool, Row,
        types::chrono::{DateTime, Utc},
    };
    use std::str::FromStr;

    #[sqlx::test(fixtures("common"))]
    async fn タスクが作成される(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());

        let title = "テストタスク".to_string();
        let assignee_user_id =
            UserId::from_str("75ef7d75-3b57-4f54-8e8e-fdb65738690c").expect("user_id取得");
        let event = CreateTodo {
            title: title.clone(),
            assignee_user_id,
            due_at: None,
        };

        let todo = repo.create(event).await.expect("作成が成功する");

        assert_eq!(todo.title, title);
        assert_eq!(todo.assignee_user_id, assignee_user_id);
        assert!(!todo.completed);
        assert!(todo.due_at.is_none());

        let row = sqlx::query("SELECT user_id, title, completed, due_at FROM todos WHERE id = $1")
            .bind(todo.id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let persisted_assignee_user_id: UserId = row.try_get("user_id").expect("user_id取得");
        let persisted_title: String = row.try_get("title").expect("title取得");
        let persisted_completed: bool = row.try_get("completed").expect("completed取得");
        let persisted_due_at: Option<DateTime<Utc>> = row.try_get("due_at").expect("due_at取得");

        assert_eq!(persisted_assignee_user_id, assignee_user_id);
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
            assignee_user_id: UserId::new(),
            due_at: None,
        };

        let err = repo
            .create(event)
            .await
            .expect_err("存在しないユーザIDでは失敗する");

        assert!(matches!(err, AppError::SqlExecuteError(_)));
    }

    #[sqlx::test(fixtures("common", "todo"))]
    async fn 自分のタスク一覧をuser_idで取得できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let target_user_id =
            UserId::from_str("75ef7d75-3b57-4f54-8e8e-fdb65738690c").expect("user_id取得");
        let target_old_todo_id: TodoId = "10f0d6f2-c464-4f4c-92f0-6d87f7324f11"
            .parse()
            .expect("todo_id取得");
        let target_new_todo_id: TodoId = "67d4895c-b538-4c81-846d-c3f08d41ecbe"
            .parse()
            .expect("todo_id取得");

        let todos = repo
            .find_by_user_id(target_user_id)
            .await
            .expect("一覧取得が成功する");

        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].id, target_new_todo_id);
        assert_eq!(todos[0].title, "target-new");
        assert_eq!(todos[0].assignee_user_id, target_user_id);
        assert_eq!(todos[1].id, target_old_todo_id);
        assert_eq!(todos[1].title, "target-old");
        assert_eq!(todos[1].assignee_user_id, target_user_id);
    }

    #[sqlx::test(fixtures("common"))]
    async fn 自分のタスク一覧取得は対象なしで空配列を返す(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let target_user_id =
            UserId::from_str("75ef7d75-3b57-4f54-8e8e-fdb65738690c").expect("user_id取得");

        let todos = repo
            .find_by_user_id(target_user_id)
            .await
            .expect("一覧取得が成功する");

        assert!(todos.is_empty());
    }
}
