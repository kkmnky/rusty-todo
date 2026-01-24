use crate::database::{ConnectionPool, model::user::UserRow};
use async_trait::async_trait;
use derive_new::new;
use kernel::{
    model::{
        id::UserId,
        user::{
            User,
            event::{CreateUser, DeleteUser},
        },
    },
    repository::user::UserRepository,
};
use shared::error::{AppError, AppResult};

#[derive(new)]
pub struct UserRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn create(&self, event: CreateUser) -> AppResult<User> {
        let user_id = UserId::new();
        let hash_password = hash_password(&event.password)?;

        let res = sqlx::query!(
            r#"--sql
                INSERT INTO users (id, name, email, password_hash)
                SELECT $1, $2, $3, $4
            "#,
            user_id as _,
            event.name,
            event.email,
            hash_password,
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SqlExecuteError)?;

        if res.rows_affected() == 0 {
            return Err(AppError::NoRowsAffectedError(
                "No user has been created".into(),
            ));
        }

        Ok(User {
            id: user_id,
            name: event.name,
            email: event.email,
        })
    }

    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
        let row = sqlx::query_as!(
            UserRow,
            r#"--sql
                SELECT
                    id,
                    name,
                    email,
                    created_at,
                    updated_at
                FROM users WHERE id = $1
            "#,
            id as _,
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SqlExecuteError)?;

        match row {
            Some(row) => Ok(Some(User::try_from(row)?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> AppResult<Vec<User>> {
        let users = sqlx::query_as!(
            UserRow,
            r#"--sql
                SELECT
                    id,
                    name,
                    email,
                    created_at,
                    updated_at
                FROM users
                ORDER BY created_at DESC
            "#,
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SqlExecuteError)?
        .into_iter()
        .filter_map(|row| User::try_from(row).ok())
        .collect();

        Ok(users)
    }

    async fn delete(&self, event: DeleteUser) -> AppResult<()> {
        let res = sqlx::query!(
            r#"--sql
                DELETE FROM users WHERE id = $1
            "#,
            event.id as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SqlExecuteError)?;

        if res.rows_affected() == 0 {
            return Err(AppError::EntityNotFoundError(
                "No user has been deleted".into(),
            ));
        }

        Ok(())
    }
}

fn hash_password(password: &str) -> AppResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connect_database_with;
    use kernel::model::id::UserId;
    use kernel::model::user::event::CreateUser;
    use shared::config::AppConfig;
    use sqlx::Row;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn ユーザが作成される() {
        let cfg = AppConfig::new().expect("DATABASE_* 環境変数が必要");
        let pool = connect_database_with(&cfg.database);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("timestamp")
            .as_nanos();
        let name = "Alice".to_string();
        let email = format!("alice+{}@example.com", unique);
        let repo = UserRepositoryImpl::new(pool.clone());
        let event = CreateUser {
            name: name.clone(),
            email: email.clone(),
            password: "password123".to_string(),
        };

        let user = repo.create(event).await.expect("作成が成功する");

        assert_eq!(user.name, name);
        assert_eq!(user.email, email);

        let row = sqlx::query("SELECT name, email, password_hash FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");

        let name: String = row.try_get("name").expect("name取得");
        let email: String = row.try_get("email").expect("email取得");
        let password_hash: String = row.try_get("password_hash").expect("password_hash取得");

        assert_eq!(name, user.name);
        assert_eq!(email, user.email);
        assert_ne!(password_hash, "password123");
        assert!(bcrypt::verify("password123", &password_hash).expect("hash検証"));
    }

    #[tokio::test]
    async fn ユーザ作成は同一メールで失敗する() {
        let cfg = AppConfig::new().expect("DATABASE_* 環境変数が必要");
        let pool = connect_database_with(&cfg.database);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("timestamp")
            .as_nanos();
        let email = format!("alice+{}@example.com", unique);
        let repo = UserRepositoryImpl::new(pool.clone());

        let first = CreateUser {
            name: "Alice".to_string(),
            email: email.clone(),
            password: "password123".to_string(),
        };
        repo.create(first).await.expect("初回作成");

        let second = CreateUser {
            name: "Bob".to_string(),
            email,
            password: "password123".to_string(),
        };
        let err = repo.create(second).await.expect_err("重複は失敗");

        assert!(matches!(err, AppError::SqlExecuteError(_)));
    }

    #[tokio::test]
    async fn ユーザ一覧は作成前後で1件増える() {
        let cfg = AppConfig::new().expect("DATABASE_* 環境変数が必要");
        let pool = connect_database_with(&cfg.database);
        let repo = UserRepositoryImpl::new(pool.clone());

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("timestamp")
            .as_nanos();
        let name = "Alice".to_string();
        let email = format!("alice+{}@example.com", unique);
        let before = repo.find_all().await.expect("一覧取得");
        let before_count = before.iter().filter(|user| user.email == email).count();
        let event = CreateUser {
            name: name.clone(),
            email: email.clone(),
            password: "password123".to_string(),
        };

        repo.create(event).await.expect("作成が成功する");

        let after = repo.find_all().await.expect("一覧取得");
        let after_count = after.iter().filter(|user| user.email == email).count();

        assert_eq!(after_count, before_count + 1);

        let created = after
            .iter()
            .find(|user| user.email == email)
            .expect("作成ユーザが含まれる");
        assert_eq!(created.name, name);
        assert_eq!(created.email, email);
    }

    #[tokio::test]
    async fn ユーザ削除で対象が消える() {
        let cfg = AppConfig::new().expect("DATABASE_* 環境変数が必要");
        let pool = connect_database_with(&cfg.database);
        let repo = UserRepositoryImpl::new(pool.clone());

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("timestamp")
            .as_nanos();
        let name = "Alice".to_string();
        let email = format!("alice+{}@example.com", unique);
        let event = CreateUser {
            name: name.clone(),
            email: email.clone(),
            password: "password123".to_string(),
        };
        let user = repo.create(event).await.expect("作成が成功する");

        repo.delete(DeleteUser { id: user.id })
            .await
            .expect("削除が成功する");

        let row = sqlx::query("SELECT COUNT(*) as count FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let count: i64 = row.try_get("count").expect("count取得");

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn 存在しないユーザは削除できない() {
        let cfg = AppConfig::new().expect("DATABASE_* 環境変数が必要");
        let pool = connect_database_with(&cfg.database);
        let repo = UserRepositoryImpl::new(pool);
        let event = DeleteUser { id: UserId::new() };

        let err = repo
            .delete(event)
            .await
            .expect_err("存在しないため失敗する");

        assert!(matches!(err, AppError::EntityNotFoundError(_)));
    }

    #[tokio::test]
    async fn ユーザ取得はid指定で取得できる() {
        let cfg = AppConfig::new().expect("DATABASE_* 環境変数が必要");
        let pool = connect_database_with(&cfg.database);
        let repo = UserRepositoryImpl::new(pool.clone());

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("timestamp")
            .as_nanos();
        let name = "Alice".to_string();
        let email = format!("alice+{}@example.com", unique);
        let event = CreateUser {
            name: name.clone(),
            email: email.clone(),
            password: "password123".to_string(),
        };
        let user = repo.create(event).await.expect("作成が成功する");

        let found = repo
            .find_by_id(user.id)
            .await
            .expect("取得が成功する")
            .expect("ユーザが存在する");

        assert_eq!(found.id, user.id);
        assert_eq!(found.name, name);
        assert_eq!(found.email, email);
    }

    #[tokio::test]
    async fn ユーザ取得は存在しないidならnoneを返す() {
        let cfg = AppConfig::new().expect("DATABASE_* 環境変数が必要");
        let pool = connect_database_with(&cfg.database);
        let repo = UserRepositoryImpl::new(pool);

        let result = repo
            .find_by_id(UserId::new())
            .await
            .expect("取得が成功する");

        assert!(result.is_none());
    }
}
