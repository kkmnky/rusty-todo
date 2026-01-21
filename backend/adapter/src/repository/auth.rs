use async_trait::async_trait;
use derive_new::new;
use shared::error::{AppError, AppResult};

use crate::database::{ConnectionPool, model::auth::UserCredentialRow};
use kernel::{model::auth::UserCredential, repository::auth::AuthRepository};

#[derive(new)]
pub struct AuthRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl AuthRepository for AuthRepositoryImpl {
    async fn find_by_email(&self, email: String) -> AppResult<Option<UserCredential>> {
        let row = sqlx::query_as!(
            UserCredentialRow,
            r#"--sql
                SELECT
                    id,
                    email,
                    password_hash
                FROM users WHERE email = $1
            "#,
            email
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SqlExecuteError)?;

        match row {
            Some(row) => Ok(Some(UserCredential::try_from(row)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::connect_database_with;
    use crate::repository::user::UserRepositoryImpl;
    use kernel::model::user::event::CreateUser;
    use kernel::repository::user::UserRepository;
    use shared::config::AppConfig;
    use sqlx::Row;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn 認証情報はメール指定で取得できる() {
        let cfg = AppConfig::new().expect("DATABASE_* 環境変数が必要");
        let pool = connect_database_with(&cfg);
        let user_repo = UserRepositoryImpl::new(pool.clone());
        let auth_repo = AuthRepositoryImpl::new(pool.clone());

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
        let user = user_repo.create(event).await.expect("作成が成功する");

        let row = sqlx::query("SELECT password_hash FROM users WHERE id = $1")
            .bind(user.id)
            .fetch_one(pool.inner_ref())
            .await
            .expect("DBから取得できる");
        let password_hash: String = row.try_get("password_hash").expect("password_hash取得");

        let credential = auth_repo
            .find_by_email(email.clone())
            .await
            .expect("取得が成功する")
            .expect("認証情報が存在する");

        assert_eq!(credential.id, user.id);
        assert_eq!(credential.email, email);
        assert_eq!(credential.password_hash, password_hash);
    }
}
