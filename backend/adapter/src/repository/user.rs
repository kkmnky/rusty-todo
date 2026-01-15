use crate::database::ConnectionPool;
use async_trait::async_trait;
use derive_new::new;
use kernel::{
    model::{
        id::UserId,
        user::{User, event::CreateUser},
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
            r#"
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
}

fn hash_password(password: &str) -> AppResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(AppError::from)
}
