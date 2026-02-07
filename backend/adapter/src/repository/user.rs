use crate::database::{ConnectionPool, model::user::UserRow};
use async_trait::async_trait;
use derive_new::new;
use kernel::{
    model::{
        id::UserId,
        user::{
            User,
            event::{CreateUser, DeleteUser, UpdatePassword},
        },
    },
    repository::user::UserRepository,
    service::password,
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
        let hash_password = password::hash(&event.password)?;

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

    async fn update_password(&self, event: UpdatePassword) -> AppResult<()> {
        let mut tx = self.db.begin().await?;
        let row = sqlx::query!(
            r#"--sql
                SELECT password_hash FROM users WHERE id = $1
            "#,
            event.id as _
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(AppError::SqlExecuteError)?;

        let current_password_hash = row
            .ok_or_else(|| AppError::EntityNotFoundError("User not found".into()))?
            .password_hash;

        if !password::verify(&event.current_password, &current_password_hash)? {
            return Err(AppError::Unauthorized("Invalid current password".into()));
        }

        let new_password_hash = password::hash(&event.new_password)?;
        sqlx::query!(
            r#"--sql
                UPDATE users SET password_hash = $1 WHERE id = $2
            "#,
            new_password_hash,
            event.id as _
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::SqlExecuteError)?;

        tx.commit().await.map_err(AppError::TransactionError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::ConnectionPool;
    use kernel::model::id::UserId;
    use kernel::model::user::event::CreateUser;
    use sqlx::{PgPool, Row};
    use std::str::FromStr;

    #[sqlx::test]
    async fn ユーザが作成される(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());

        let name = "Alice".to_string();
        let email = "alice@example.com".to_string();
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
        assert!(
            password::verify("password123", &password_hash).expect("hash検証"),
            "hash検証"
        );
    }

    #[sqlx::test(fixtures("common"))]
    async fn ユーザ作成は同一メールで失敗する(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());

        let name = "Bob".to_string();
        let email = "common-fixtures@example.com".to_string();
        let password = "password123".to_string();
        let repo = UserRepositoryImpl::new(pool.clone());

        let err = repo
            .create(CreateUser {
                name,
                email,
                password,
            })
            .await
            .expect_err("重複は失敗");

        assert!(matches!(err, AppError::SqlExecuteError(_)));
    }

    #[sqlx::test]
    async fn ユーザ一覧は作成前後で1件増える(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = UserRepositoryImpl::new(pool.clone());

        let name = "Alice".to_string();
        let email = "alice@example.com".to_string();
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

    #[sqlx::test(fixtures("common"))]
    async fn ユーザ削除で対象が消える(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = UserRepositoryImpl::new(pool.clone());

        let user_id =
            UserId::from_str("75ef7d75-3b57-4f54-8e8e-fdb65738690c").expect("user_id取得");

        repo.delete(DeleteUser { id: user_id })
            .await
            .expect("削除が成功する");

        let row = sqlx::query("SELECT COUNT(*) as count FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let count: i64 = row.try_get("count").expect("count取得");

        assert_eq!(count, 0);
    }

    #[sqlx::test]
    async fn 存在しないユーザは削除できない(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = UserRepositoryImpl::new(pool);
        let event = DeleteUser { id: UserId::new() };

        let err = repo
            .delete(event)
            .await
            .expect_err("存在しないため失敗する");

        assert!(matches!(err, AppError::EntityNotFoundError(_)));
    }

    #[sqlx::test(fixtures("common"))]
    async fn ユーザ取得はid指定で取得できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = UserRepositoryImpl::new(pool.clone());

        let user_id =
            UserId::from_str("75ef7d75-3b57-4f54-8e8e-fdb65738690c").expect("user_id取得");
        let name = "Fixtures".to_string();
        let email = "common-fixtures@example.com".to_string();

        let found = repo
            .find_by_id(user_id)
            .await
            .expect("取得が成功する")
            .expect("ユーザが存在する");

        assert_eq!(found.id, user_id);
        assert_eq!(found.name, name);
        assert_eq!(found.email, email);
    }

    #[sqlx::test]
    async fn ユーザ取得は存在しないidならnoneを返す(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = UserRepositoryImpl::new(pool);

        let result = repo
            .find_by_id(UserId::new())
            .await
            .expect("取得が成功する");

        assert!(result.is_none());
    }

    #[sqlx::test(fixtures("common"))]
    async fn パスワード更新でハッシュが更新される(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = UserRepositoryImpl::new(pool.clone());

        let user_id =
            UserId::from_str("75ef7d75-3b57-4f54-8e8e-fdb65738690c").expect("user_id取得");

        let row = sqlx::query("SELECT password_hash FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let old_password_hash: String = row.try_get("password_hash").expect("password_hash取得");

        let update_event = UpdatePassword {
            id: user_id,
            current_password: "password123".to_string(),
            new_password: "password456".to_string(),
        };
        repo.update_password(update_event)
            .await
            .expect("更新が成功する");

        let row = sqlx::query("SELECT password_hash FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let new_password_hash: String = row.try_get("password_hash").expect("password_hash取得");

        assert_ne!(old_password_hash, new_password_hash);
        assert!(
            password::verify("password456", &new_password_hash).expect("hash検証"),
            "hash検証"
        );
        assert!(
            !password::verify("password123", &new_password_hash).expect("hash検証"),
            "hash検証"
        );
    }

    #[sqlx::test]
    async fn パスワード更新は存在しないユーザで失敗する(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = UserRepositoryImpl::new(pool);

        let event = UpdatePassword {
            id: UserId::new(),
            current_password: "password123".to_string(),
            new_password: "password456".to_string(),
        };

        let err = repo
            .update_password(event)
            .await
            .expect_err("存在しないユーザは失敗する");

        assert!(matches!(err, AppError::EntityNotFoundError(_)));
    }

    #[sqlx::test(fixtures("common"))]
    async fn パスワード更新は現在パスワード不一致で失敗する(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let repo = UserRepositoryImpl::new(pool.clone());

        let user_id =
            UserId::from_str("75ef7d75-3b57-4f54-8e8e-fdb65738690c").expect("user_id取得");

        let event = UpdatePassword {
            id: user_id,
            current_password: "wrong-password".to_string(),
            new_password: "new-password".to_string(),
        };

        let err = repo
            .update_password(event)
            .await
            .expect_err("不一致は失敗する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }
}
