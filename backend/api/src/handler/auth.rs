use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::AUTHORIZATION},
};
use garde::Validate;
use kernel::{
    model::auth::AccessToken,
    service::jwt::VerifiedToken,
    usecase::auth::{
        login::{LoginInput, LoginUsecase},
        logout::LogoutUsecase,
    },
};
use registry::AppRegistry;
use shared::{
    error::{AppError, AppResult},
    logging::mask_email,
};

use crate::model::auth::{AccessTokenResponse, LoginRequest};

pub async fn auth_login(
    State(registry): State<AppRegistry>,
    Json(req): Json<LoginRequest>,
) -> AppResult<(StatusCode, Json<AccessTokenResponse>)> {
    let email_masked = mask_email(&req.email);
    tracing::debug!(
        event.name = "auth.login.attempt",
        attributes.user.email_masked = %email_masked,
        "login attempt"
    );

    req.validate()?;

    let usecase = LoginUsecase::new(registry.auth_repository(), registry.jwt_issuer());

    let result = usecase
        .execute(LoginInput {
            email: req.email,
            password: req.password,
        })
        .await
        .inspect_err(|err| {
            tracing::warn!(
                event.name = "auth.login.failed",
                attributes.user.email_masked = %email_masked,
                error.message = %err,
                "login failed"
            );
        })?;

    Ok((
        StatusCode::OK,
        Json(AccessTokenResponse {
            access_token: result.access_token,
            expires_in: result.expires_in,
            user_id: result.user_id,
        }),
    ))
}

pub async fn auth_logout(
    State(registry): State<AppRegistry>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let access_token = extract_bearer(&headers)?;
    let usecase = LogoutUsecase::new(registry.auth_repository());

    usecase.execute(access_token).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub(crate) fn require_auth(
    registry: &AppRegistry,
    headers: &HeaderMap,
) -> AppResult<VerifiedToken> {
    let access_token = extract_bearer(headers)?;
    let verified_token = registry.jwt_issuer().verify_token(&access_token)?;
    tracing::Span::current().record("user_id", tracing::field::display(verified_token.sub));
    Ok(verified_token)
}

pub(crate) fn extract_bearer(headers: &HeaderMap) -> AppResult<AccessToken> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing bearer token".into()))?;

    let mut parts = value.splitn(2, ' ');
    let scheme = parts.next().unwrap_or("");
    let token = parts.next().map(str::trim_start).filter(|v| !v.is_empty());

    if !scheme.eq_ignore_ascii_case("bearer") || token.is_none() {
        return Err(AppError::Unauthorized("Missing bearer token".into()));
    }

    Ok(AccessToken(token.unwrap().to_string()))
}

#[cfg(test)]
mod tests {
    use super::{auth_login, auth_logout};
    use crate::handler::test_support::{build_auth_header, build_test_jwt_issuer};
    use axum::{
        Json,
        extract::State,
        http::{HeaderMap, StatusCode},
    };
    use kernel::model::{
        auth::{AccessToken, UserCredential},
        id::UserId,
    };
    use kernel::repository::auth::{AuthRepository, MockAuthRepository};
    use kernel::service::{jwt::JwtIssuer, password};
    use registry::{AppRegistry, MockAppRegistryExt};
    use rstest::{fixture, rstest};
    use shared::error::AppError;
    use std::sync::Arc;

    use crate::model::auth::LoginRequest;

    #[fixture]
    fn jwt_issuer() -> Arc<JwtIssuer> {
        build_test_jwt_issuer()
    }

    #[rstest]
    #[tokio::test]
    async fn ログインは200とアクセストークンと期限を返す(
        jwt_issuer: Arc<JwtIssuer>,
    ) {
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
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let req = LoginRequest::new(email, password);

        let result = auth_login(State(registry), Json(req)).await;
        let (status, Json(body)) = result.expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.access_token, stored_token);
        assert_eq!(body.expires_in, 60 * 60 * 24);
        assert_eq!(body.user_id, user_id);
    }

    #[rstest]
    #[tokio::test]
    async fn パスワード不一致で401を返す(jwt_issuer: Arc<JwtIssuer>) {
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

        repo.expect_store_token().times(0);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn AuthRepository> = Arc::new(repo);
        registry
            .expect_auth_repository()
            .return_const(repo_arc.clone());
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let req = LoginRequest::new(email, "wrong-password".to_string());

        let err = auth_login(State(registry), Json(req))
            .await
            .expect_err("パスワード不一致で401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn メールアドレスが存在しないで401を返す(jwt_issuer: Arc<JwtIssuer>) {
        let mut repo = MockAuthRepository::new();
        repo.expect_find_by_email().returning(move |_| Ok(None));

        repo.expect_store_token().times(0);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn AuthRepository> = Arc::new(repo);
        registry
            .expect_auth_repository()
            .return_const(repo_arc.clone());
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);

        let email = "alice@example.com".to_string();
        let password = "password123".to_string();
        let req = LoginRequest::new(email, password);

        let err = auth_login(State(registry), Json(req))
            .await
            .expect_err("メールアドレスが存在しないで401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn ログアウトは204を返す() {
        let token_value = "test-token".to_string();
        let token = AccessToken(token_value.clone());

        let mut repo = MockAuthRepository::new();
        let token_for_match = token.clone();
        repo.expect_delete_token()
            .withf(move |value| value == &token_for_match)
            .returning(|_| Ok(()));

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn AuthRepository> = Arc::new(repo);
        registry
            .expect_auth_repository()
            .return_const(repo_arc.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_auth_header(&token_value);

        let status = auth_logout(State(registry), headers)
            .await
            .expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[rstest]
    #[tokio::test]
    async fn authorizationヘッダがないと401を返す() {
        let mut repo = MockAuthRepository::new();
        repo.expect_delete_token().times(0);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn AuthRepository> = Arc::new(repo);
        registry
            .expect_auth_repository()
            .return_const(repo_arc.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = HeaderMap::new();

        let err = auth_logout(State(registry), headers)
            .await
            .expect_err("Authorizationヘッダなしは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[rstest]
    #[tokio::test]
    async fn 無効なアクセストークンで401を返す() {
        let token_value = "invalid-token".to_string();
        let token = AccessToken(token_value.clone());

        let mut repo = MockAuthRepository::new();
        repo.expect_delete_token()
            .withf(move |value| value == &token)
            .returning(|_| Err(AppError::Unauthorized("Invalid token".into())));

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn AuthRepository> = Arc::new(repo);
        registry
            .expect_auth_repository()
            .return_const(repo_arc.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_auth_header(&token_value);

        let err = auth_logout(State(registry), headers)
            .await
            .expect_err("無効トークンは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }
}
