use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use shared::error::{AppError, AppResult};

use crate::{
    database::{ConnectionPool, model::auth::UserCredentialRow},
    redis::{
        RedisClient,
        model::auth::{AuthorizationKey, from},
    },
};
use kernel::{
    model::auth::{AccessToken, UserCredential, mutations::StoreToken},
    repository::auth::AuthRepository,
};

#[derive(new)]
pub struct AuthRepositoryImpl {
    db: ConnectionPool,
    kv_store: Arc<RedisClient>,
    ttl: u64,
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

    async fn store_token(&self, event: StoreToken) -> AppResult<AccessToken> {
        let (key, value) = from(event);
        self.kv_store.set_ex(&key, &value, self.ttl).await?;
        Ok(key.into())
    }

    async fn delete_token(&self, access_token: AccessToken) -> AppResult<()> {
        let key: AuthorizationKey = access_token.into();
        let deleted_count = self.kv_store.delete(&key).await?;
        if deleted_count == 0 {
            return Err(AppError::Unauthorized("Invalid token".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::ConnectionPool;
    use crate::redis::model::RedisValue;
    use kernel::model::id::UserId;
    use kernel::service::password;
    use shared::config::RedisConfig;
    use sqlx::PgPool;
    use std::{
        str::FromStr,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn redis_client() -> Arc<RedisClient> {
        let host = std::env::var("REDIS_HOST").expect("REDIS_HOST");
        let port = std::env::var("REDIS_PORT")
            .expect("REDIS_PORT")
            .parse()
            .expect("REDIS_PORT");
        let config = RedisConfig { host, port };
        Arc::new(RedisClient::new(&config).expect("Redis接続が成功する"))
    }

    fn auth_ttl() -> u64 {
        std::env::var("AUTH_TOKEN_TTL")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(3600)
    }

    #[sqlx::test(fixtures("common"))]
    async fn 認証情報はメール指定で取得できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool);
        let kv_store = redis_client();
        let auth_repo = AuthRepositoryImpl::new(pool.clone(), kv_store, auth_ttl());

        let user_id =
            UserId::from_str("75ef7d75-3b57-4f54-8e8e-fdb65738690c").expect("user_id取得");
        let email = "common-fixtures@example.com".to_string();
        let password = "password123".to_string();

        let credential = auth_repo
            .find_by_email(email.clone())
            .await
            .expect("取得が成功する")
            .expect("認証情報が存在する");

        assert_eq!(credential.id, user_id);
        assert_eq!(credential.email, email);
        assert!(password::verify(&password, &credential.password_hash).expect("hash検証"));
    }

    #[sqlx::test]
    async fn 認証情報は存在しないメールならnoneを返す(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let kv_store = redis_client();
        let auth_repo = AuthRepositoryImpl::new(pool.clone(), kv_store, auth_ttl());

        let result = auth_repo
            .find_by_email("not-found@example.com".to_string())
            .await
            .expect("取得が成功する");

        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn アクセストークンは保存できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let kv_store = redis_client();
        let auth_repo = AuthRepositoryImpl::new(pool, kv_store, auth_ttl());

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("timestamp")
            .as_nanos();
        let user_id = UserId::new();
        let token = AccessToken(format!("test-token-{}", unique));
        let event = StoreToken {
            user_id,
            access_token: token.clone(),
        };

        let stored = auth_repo.store_token(event).await.expect("保存が成功する");

        assert_eq!(stored, token);

        let (key, _value) = from(StoreToken {
            user_id,
            access_token: token.clone(),
        });
        let value = auth_repo.kv_store.get(&key).await.expect("token取得");
        let ttl = auth_repo.kv_store.ttl(&key).await.expect("ttl取得");
        let value = value.map(|value| value.inner());

        assert_eq!(value, Some(user_id.to_string()));
        assert!(ttl > 0);
        assert!(ttl <= auth_ttl() as i64);
    }

    #[sqlx::test]
    async fn アクセストークンは削除できる(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let kv_store = redis_client();
        let auth_repo = AuthRepositoryImpl::new(pool, kv_store, auth_ttl());

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("timestamp")
            .as_nanos();
        let user_id = UserId::new();
        let token = AccessToken(format!("test-token-{}", unique));
        let event = StoreToken {
            user_id,
            access_token: token.clone(),
        };

        auth_repo.store_token(event).await.expect("保存が成功する");

        let (key, _value) = from(StoreToken {
            user_id,
            access_token: token.clone(),
        });
        let stored = auth_repo.kv_store.get(&key).await.expect("token取得");
        assert!(stored.is_some());

        auth_repo
            .delete_token(token.clone())
            .await
            .expect("削除が成功する");

        let value = auth_repo.kv_store.get(&key).await.expect("token取得");
        assert!(value.is_none());

        let ttl = auth_repo.kv_store.ttl(&key).await.expect("ttl取得");
        assert_eq!(ttl, -2);
    }

    #[sqlx::test]
    async fn 無効なアクセストークンは削除できない(pool: PgPool) {
        let pool = ConnectionPool::new(pool.clone());
        let kv_store = redis_client();
        let auth_repo = AuthRepositoryImpl::new(pool, kv_store, auth_ttl());

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("timestamp")
            .as_nanos();
        let token = AccessToken(format!("test-token-{}", unique));

        let err = auth_repo
            .delete_token(token)
            .await
            .expect_err("無効トークンの削除は失敗する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }
}
