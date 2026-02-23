use async_trait::async_trait;
use derive_new::new;
use kernel::{
    model::{
        id::{TodoId, UserId},
        todo::{
            Todo,
            event::{CreateTodo, UpdateTodo, UpdateTodoCompleted},
        },
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

    async fn find_by_id(&self, id: TodoId) -> AppResult<Option<Todo>> {
        let row = sqlx::query_as!(
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
                FROM todos WHERE id = $1
            "#,
            id as _,
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SqlExecuteError)?;

        match row {
            Some(row) => Ok(Some(Todo::try_from(row)?)),
            None => Ok(None),
        }
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

    async fn update(&self, event: UpdateTodo) -> AppResult<Todo> {
        let row = sqlx::query_as!(
            TodoRow,
            r#"--sql
                UPDATE todos
                SET title = $1,
                    user_id = $2,
                    due_at = $3
                WHERE id = $4
                RETURNING
                    id,
                    user_id,
                    title,
                    completed,
                    due_at,
                    created_at,
                    updated_at
            "#,
            event.title,
            event.assignee_user_id as _,
            event.due_at,
            event.id as _,
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SqlExecuteError)?
        .ok_or_else(|| AppError::EntityNotFoundError("No todo has been updated".into()))?;

        let todo = Todo::try_from(row)?;

        Ok(todo)
    }

    async fn update_completed(&self, event: UpdateTodoCompleted) -> AppResult<Todo> {
        let row = sqlx::query_as!(
            TodoRow,
            r#"--sql
                UPDATE todos
                SET completed = $1
                WHERE id = $2
                RETURNING
                    id,
                    user_id,
                    title,
                    completed,
                    due_at,
                    created_at,
                    updated_at
            "#,
            event.completed,
            event.id as _,
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SqlExecuteError)?
        .ok_or_else(|| AppError::EntityNotFoundError("No todo has been updated".into()))?;

        let todo = Todo::try_from(row)?;

        Ok(todo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::ConnectionPool;
    use kernel::model::{
        id::{TodoId, UserId},
        todo::event::{CreateTodo, UpdateTodo},
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

    #[sqlx::test(fixtures("common", "todo"))]
    async fn タスク取得はid指定で取得できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let target_todo_id: TodoId = "10f0d6f2-c464-4f4c-92f0-6d87f7324f11"
            .parse()
            .expect("todo_id取得");
        let target_user_id: UserId = "75ef7d75-3b57-4f54-8e8e-fdb65738690c"
            .parse()
            .expect("user_id取得");

        let found = repo
            .find_by_id(target_todo_id)
            .await
            .expect("取得が成功する")
            .expect("タスクが存在する");

        assert_eq!(found.id, target_todo_id);
        assert_eq!(found.assignee_user_id, target_user_id);
        assert_eq!(found.title, "target-old");
        assert!(!found.completed);
        assert!(found.due_at.is_none());
    }

    #[sqlx::test(fixtures("common"))]
    async fn タスク取得は存在しないidならnoneを返す(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());

        let found = repo
            .find_by_id(TodoId::new())
            .await
            .expect("取得が成功する");

        assert!(found.is_none());
    }

    #[sqlx::test(fixtures("common", "todo"))]
    async fn タスク完了状態を未完了から完了に更新できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let target_todo_id: TodoId = "10f0d6f2-c464-4f4c-92f0-6d87f7324f11"
            .parse()
            .expect("todo_id取得");
        let target_user_id: UserId = "75ef7d75-3b57-4f54-8e8e-fdb65738690c"
            .parse()
            .expect("user_id取得");
        let event = UpdateTodoCompleted {
            id: target_todo_id,
            completed: true,
        };

        let updated_todo = repo.update_completed(event).await.expect("更新が成功する");

        assert_eq!(updated_todo.id, target_todo_id);
        assert_eq!(updated_todo.assignee_user_id, target_user_id);
        assert_eq!(updated_todo.title, "target-old");
        assert!(updated_todo.completed);
        assert!(updated_todo.due_at.is_none());

        let row = sqlx::query("SELECT user_id, title, completed, due_at FROM todos WHERE id = $1")
            .bind(target_todo_id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let persisted_assignee_user_id: UserId = row.try_get("user_id").expect("user_id取得");
        let persisted_title: String = row.try_get("title").expect("title取得");
        let persisted_completed: bool = row.try_get("completed").expect("completed取得");
        let persisted_due_at: Option<DateTime<Utc>> = row.try_get("due_at").expect("due_at取得");

        assert_eq!(persisted_assignee_user_id, updated_todo.assignee_user_id);
        assert_eq!(persisted_title, updated_todo.title);
        assert_eq!(persisted_completed, updated_todo.completed);
        assert_eq!(persisted_due_at, updated_todo.due_at);
    }

    #[sqlx::test(fixtures("common", "todo"))]
    async fn タスク完了状態を完了から未完了に更新できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let target_todo_id: TodoId = "67d4895c-b538-4c81-846d-c3f08d41ecbe"
            .parse()
            .expect("todo_id取得");
        let target_user_id: UserId = "75ef7d75-3b57-4f54-8e8e-fdb65738690c"
            .parse()
            .expect("user_id取得");

        sqlx::query("UPDATE todos SET completed = true WHERE id = $1")
            .bind(target_todo_id)
            .execute(pool.inner_ref())
            .await
            .expect("初期状態を完了にできる");

        let event = UpdateTodoCompleted {
            id: target_todo_id,
            completed: false,
        };
        let updated_todo = repo.update_completed(event).await.expect("更新が成功する");

        assert_eq!(updated_todo.id, target_todo_id);
        assert_eq!(updated_todo.assignee_user_id, target_user_id);
        assert_eq!(updated_todo.title, "target-new");
        assert!(!updated_todo.completed);
        assert!(updated_todo.due_at.is_none());

        let row = sqlx::query("SELECT user_id, title, completed, due_at FROM todos WHERE id = $1")
            .bind(target_todo_id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let persisted_assignee_user_id: UserId = row.try_get("user_id").expect("user_id取得");
        let persisted_title: String = row.try_get("title").expect("title取得");
        let persisted_completed: bool = row.try_get("completed").expect("completed取得");
        let persisted_due_at: Option<DateTime<Utc>> = row.try_get("due_at").expect("due_at取得");

        assert_eq!(persisted_assignee_user_id, updated_todo.assignee_user_id);
        assert_eq!(persisted_title, updated_todo.title);
        assert_eq!(persisted_completed, updated_todo.completed);
        assert_eq!(persisted_due_at, updated_todo.due_at);
    }

    #[sqlx::test(fixtures("common"))]
    async fn タスク完了状態更新は存在しないtodo_idで失敗する(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());

        let event = UpdateTodoCompleted {
            id: TodoId::new(),
            completed: true,
        };
        let err = repo
            .update_completed(event)
            .await
            .expect_err("存在しないtodo_idでは失敗する");

        assert!(matches!(err, AppError::EntityNotFoundError(_)));
    }

    #[sqlx::test(fixtures("common", "todo"))]
    async fn タスク編集でtitleのみ更新できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let target_todo_id: TodoId = "10f0d6f2-c464-4f4c-92f0-6d87f7324f11"
            .parse()
            .expect("todo_id取得");
        let target_user_id: UserId = "75ef7d75-3b57-4f54-8e8e-fdb65738690c"
            .parse()
            .expect("user_id取得");

        let event = UpdateTodo {
            id: target_todo_id,
            title: "edited-title".to_string(),
            due_at: None,
            assignee_user_id: target_user_id,
        };

        let updated_todo = repo.update(event).await.expect("更新が成功する");

        assert_eq!(updated_todo.id, target_todo_id);
        assert_eq!(updated_todo.assignee_user_id, target_user_id);
        assert_eq!(updated_todo.title, "edited-title");
        assert!(!updated_todo.completed);
        assert!(updated_todo.due_at.is_none());

        let row = sqlx::query("SELECT user_id, title, due_at FROM todos WHERE id = $1")
            .bind(target_todo_id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let persisted_user_id: UserId = row.try_get("user_id").expect("user_id取得");
        let persisted_title: String = row.try_get("title").expect("title取得");
        let persisted_due_at: Option<DateTime<Utc>> = row.try_get("due_at").expect("due_at取得");

        assert_eq!(persisted_user_id, updated_todo.assignee_user_id);
        assert_eq!(persisted_title, updated_todo.title);
        assert_eq!(persisted_due_at, updated_todo.due_at);
    }

    #[sqlx::test(fixtures("common", "todo"))]
    async fn タスク編集でdue_atのみ更新できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let target_todo_id: TodoId = "10f0d6f2-c464-4f4c-92f0-6d87f7324f11"
            .parse()
            .expect("todo_id取得");
        let target_user_id: UserId = "75ef7d75-3b57-4f54-8e8e-fdb65738690c"
            .parse()
            .expect("user_id取得");
        let due_at = DateTime::parse_from_rfc3339("2026-02-01T09:30:00Z")
            .expect("due_at作成")
            .with_timezone(&Utc);

        let event = UpdateTodo {
            id: target_todo_id,
            title: "target-old".to_string(),
            due_at: Some(due_at),
            assignee_user_id: target_user_id,
        };

        let updated_todo = repo.update(event).await.expect("更新が成功する");

        assert_eq!(updated_todo.id, target_todo_id);
        assert_eq!(updated_todo.assignee_user_id, target_user_id);
        assert_eq!(updated_todo.title, "target-old");
        assert_eq!(updated_todo.due_at, Some(due_at));

        let row = sqlx::query("SELECT title, due_at FROM todos WHERE id = $1")
            .bind(target_todo_id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let persisted_title: String = row.try_get("title").expect("title取得");
        let persisted_due_at: Option<DateTime<Utc>> = row.try_get("due_at").expect("due_at取得");

        assert_eq!(persisted_title, updated_todo.title);
        assert_eq!(persisted_due_at, updated_todo.due_at);
    }

    #[sqlx::test(fixtures("common", "todo"))]
    async fn タスク編集でassignee_user_idのみ更新できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let target_todo_id: TodoId = "10f0d6f2-c464-4f4c-92f0-6d87f7324f11"
            .parse()
            .expect("todo_id取得");
        let changed_assignee_user_id: UserId = "f0f6de0a-8e7f-4ca3-a0ed-2db4e8d51056"
            .parse()
            .expect("user_id取得");

        let event = UpdateTodo {
            id: target_todo_id,
            title: "target-old".to_string(),
            due_at: None,
            assignee_user_id: changed_assignee_user_id,
        };

        let updated_todo = repo.update(event).await.expect("更新が成功する");

        assert_eq!(updated_todo.id, target_todo_id);
        assert_eq!(updated_todo.assignee_user_id, changed_assignee_user_id);
        assert_eq!(updated_todo.title, "target-old");
        assert!(updated_todo.due_at.is_none());

        let row = sqlx::query("SELECT user_id, title, due_at FROM todos WHERE id = $1")
            .bind(target_todo_id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let persisted_user_id: UserId = row.try_get("user_id").expect("user_id取得");
        let persisted_title: String = row.try_get("title").expect("title取得");
        let persisted_due_at: Option<DateTime<Utc>> = row.try_get("due_at").expect("due_at取得");

        assert_eq!(persisted_user_id, updated_todo.assignee_user_id);
        assert_eq!(persisted_title, updated_todo.title);
        assert_eq!(persisted_due_at, updated_todo.due_at);
    }

    #[sqlx::test(fixtures("common", "todo"))]
    async fn タスク編集で複数項目を同時更新できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let target_todo_id: TodoId = "10f0d6f2-c464-4f4c-92f0-6d87f7324f11"
            .parse()
            .expect("todo_id取得");
        let changed_assignee_user_id: UserId = "f0f6de0a-8e7f-4ca3-a0ed-2db4e8d51056"
            .parse()
            .expect("user_id取得");
        let due_at = DateTime::parse_from_rfc3339("2026-03-01T12:00:00Z")
            .expect("due_at作成")
            .with_timezone(&Utc);

        let event = UpdateTodo {
            id: target_todo_id,
            title: "edited-all-fields".to_string(),
            due_at: Some(due_at),
            assignee_user_id: changed_assignee_user_id,
        };

        let updated_todo = repo.update(event).await.expect("更新が成功する");

        assert_eq!(updated_todo.id, target_todo_id);
        assert_eq!(updated_todo.assignee_user_id, changed_assignee_user_id);
        assert_eq!(updated_todo.title, "edited-all-fields");
        assert_eq!(updated_todo.due_at, Some(due_at));

        let row = sqlx::query("SELECT user_id, title, due_at FROM todos WHERE id = $1")
            .bind(target_todo_id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let persisted_user_id: UserId = row.try_get("user_id").expect("user_id取得");
        let persisted_title: String = row.try_get("title").expect("title取得");
        let persisted_due_at: Option<DateTime<Utc>> = row.try_get("due_at").expect("due_at取得");

        assert_eq!(persisted_user_id, updated_todo.assignee_user_id);
        assert_eq!(persisted_title, updated_todo.title);
        assert_eq!(persisted_due_at, updated_todo.due_at);
    }

    #[sqlx::test(fixtures("common", "todo"))]
    async fn タスク編集でdue_atをnullで解除できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let target_todo_id: TodoId = "10f0d6f2-c464-4f4c-92f0-6d87f7324f11"
            .parse()
            .expect("todo_id取得");
        let target_user_id: UserId = "75ef7d75-3b57-4f54-8e8e-fdb65738690c"
            .parse()
            .expect("user_id取得");

        let existing_due_at = DateTime::parse_from_rfc3339("2026-04-01T08:00:00Z")
            .expect("due_at作成")
            .with_timezone(&Utc);
        sqlx::query("UPDATE todos SET due_at = $1 WHERE id = $2")
            .bind(existing_due_at)
            .bind(target_todo_id)
            .execute(pool.inner_ref())
            .await
            .expect("初期状態を設定できる");

        let event = UpdateTodo {
            id: target_todo_id,
            title: "target-old".to_string(),
            due_at: None,
            assignee_user_id: target_user_id,
        };

        let updated_todo = repo.update(event).await.expect("更新が成功する");

        assert_eq!(updated_todo.id, target_todo_id);
        assert_eq!(updated_todo.title, "target-old");
        assert!(updated_todo.due_at.is_none());

        let row = sqlx::query("SELECT due_at FROM todos WHERE id = $1")
            .bind(target_todo_id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let persisted_due_at: Option<DateTime<Utc>> = row.try_get("due_at").expect("due_at取得");

        assert_eq!(persisted_due_at, updated_todo.due_at);
    }

    #[sqlx::test(fixtures("common", "todo"))]
    async fn タスク編集は存在しないtodo_idで失敗する(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());

        let event = UpdateTodo {
            id: TodoId::new(),
            title: "edited-title".to_string(),
            due_at: None,
            assignee_user_id: UserId::new(),
        };
        let err = repo
            .update(event)
            .await
            .expect_err("存在しないtodo_idでは失敗する");

        assert!(matches!(err, AppError::EntityNotFoundError(_)));
    }

    #[sqlx::test(fixtures("common", "todo"))]
    async fn タスク編集は存在しないassignee_user_idで失敗する(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = TodoRepositoryImpl::new(pool.clone());
        let target_todo_id: TodoId = "10f0d6f2-c464-4f4c-92f0-6d87f7324f11"
            .parse()
            .expect("todo_id取得");

        let event = UpdateTodo {
            id: target_todo_id,
            title: "target-old".to_string(),
            due_at: None,
            assignee_user_id: UserId::new(),
        };
        let err = repo
            .update(event)
            .await
            .expect_err("存在しないassignee_user_idでは失敗する");

        assert!(matches!(err, AppError::SqlExecuteError(_)));
    }
}
