use axum::{Json, extract::State, http::StatusCode};
use garde::Validate;
use kernel::usecase::auth::login::{LoginInput, LoginUsecase};
use registry::AppRegistry;
use shared::error::AppResult;

use crate::model::auth::{AccessTokenResponse, LoginRequest};

pub async fn auth_login(
    State(registry): State<AppRegistry>,
    Json(req): Json<LoginRequest>,
) -> AppResult<(StatusCode, Json<AccessTokenResponse>)> {
    req.validate()?;

    let usecase = LoginUsecase::new(
        registry.auth_repository(),
        registry.jwt_issuer(),
        registry.jwt_issuer().ttl(),
    );

    let result = usecase
        .execute(LoginInput {
            email: req.email,
            password: req.password,
        })
        .await?;

    Ok((
        StatusCode::OK,
        Json(AccessTokenResponse {
            access_token: result.access_token,
            expires_in: result.expires_in,
            user_id: result.user_id,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::auth_login;
    use axum::{Json, extract::State, http::StatusCode};
    use kernel::model::{
        auth::{AccessToken, UserCredential},
        id::UserId,
    };
    use kernel::repository::auth::{AuthRepository, MockAuthRepository};
    use kernel::service::{jwt::JwtIssuer, password};
    use registry::{AppRegistry, MockAppRegistryExt};
    use std::sync::Arc;

    use crate::model::auth::LoginRequest;

    #[tokio::test]
    async fn ログインは200とアクセストークンと期限を返す() {
        let user_id = UserId::new();
        let email = "alice@example.com".to_string();
        let password = "password123".to_string();
        let password_hash = password::hash(&password).expect("hash作成");

        let mut repo = MockAuthRepository::new();
        let email_for_match = email.clone();
        let email_for_return = email.clone();
        let password_hash_for_return = password_hash.clone();
        repo.expect_find_by_email()
            .withf(move |value| value == &email_for_match)
            .returning(move |_| {
                Ok(Some(UserCredential {
                    id: user_id,
                    email: email_for_return.clone(),
                    password_hash: password_hash_for_return.clone(),
                }))
            });

        let stored_token = AccessToken("stored-token".to_string());
        let stored_token_for_return = stored_token.clone();
        repo.expect_store_token()
            .withf(move |event| event.user_id == user_id && !event.access_token.0.is_empty())
            .returning(move |_| Ok(stored_token_for_return.clone()));

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn AuthRepository> = Arc::new(repo);
        registry
            .expect_auth_repository()
            .return_const(repo_arc.clone());
        registry
            .expect_jwt_issuer()
            .return_const(Arc::new(JwtIssuer::new(
                "test-secret".to_string(),
                60_u64 * 60 * 24,
            )));

        let registry: AppRegistry = Arc::new(registry);
        let req = LoginRequest::new(email, password);

        let result = auth_login(State(registry), Json(req)).await;
        let (status, Json(body)) = result.expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.access_token, stored_token);
        assert_eq!(body.expires_in, 60 * 60 * 24);
        assert_eq!(body.user_id, user_id);
    }
}
