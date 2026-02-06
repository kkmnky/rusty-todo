use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use garde::Validate;
use kernel::model::{
    id::UserId,
    user::event::{DeleteUser, UpdatePassword},
};
use registry::AppRegistry;

use crate::handler::auth::require_auth;
use crate::model::user::{ChangePasswordRequest, CreateUserRequest, UserResponse, UsersResponse};
use shared::error::{AppError, AppResult};

pub async fn register_user(
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    req.validate()?;

    let registered_user = registry.user_repository().create(req.into()).await?;

    Ok((StatusCode::CREATED, Json(registered_user.into())))
}

pub async fn list_users(
    State(registry): State<AppRegistry>,
    headers: HeaderMap,
) -> AppResult<(StatusCode, Json<UsersResponse>)> {
    require_auth(&registry, &headers)?;

    let items = registry
        .user_repository()
        .find_all()
        .await?
        .into_iter()
        .map(UserResponse::from)
        .collect();

    Ok((StatusCode::OK, Json(UsersResponse { items })))
}

pub async fn delete_user(
    State(registry): State<AppRegistry>,
    Path(user_id): Path<String>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    require_auth(&registry, &headers)?;

    let user_id: UserId = user_id.parse()?;
    registry
        .user_repository()
        .delete(DeleteUser { id: user_id })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_current_user(
    State(registry): State<AppRegistry>,
    headers: HeaderMap,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    let verified_token = require_auth(&registry, &headers)?;
    let user_id = verified_token.sub;

    let user = registry
        .user_repository()
        .find_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::EntityNotFoundError("user not found".into()))?;

    Ok((StatusCode::OK, Json(user.into())))
}

pub async fn change_password(
    State(registry): State<AppRegistry>,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;
    let verified_token = require_auth(&registry, &headers)?;

    let event = UpdatePassword {
        id: verified_token.sub,
        current_password: req.current_password,
        new_password: req.new_password,
    };

    registry.user_repository().update_password(event).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::{Path, State},
        http::{HeaderMap, HeaderValue, header::AUTHORIZATION},
    };
    use kernel::repository::user::{MockUserRepository, UserRepository};
    use kernel::{
        model::{id::UserId, user::User},
        service::jwt::JwtIssuer,
    };
    use registry::MockAppRegistryExt;
    use shared::error::AppError;
    use std::sync::Arc;

    fn build_auth_header(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        let value = HeaderValue::from_str(&format!("Bearer {}", token)).expect("header生成");
        headers.insert(AUTHORIZATION, value);
        headers
    }

    fn build_valid_auth_header(issuer: &JwtIssuer) -> HeaderMap {
        let token = issuer.issue_token(UserId::new()).expect("jwt生成");
        build_auth_header(&token.0)
    }

    fn build_auth_header_for_user(issuer: &JwtIssuer, user_id: UserId) -> HeaderMap {
        let token = issuer.issue_token(user_id).expect("jwt生成");
        build_auth_header(&token.0)
    }

    #[tokio::test]
    async fn ユーザ追加は201と必要項目を返す() {
        let mut repo = MockUserRepository::new();
        repo.expect_create().returning(|event| {
            Ok(User {
                id: UserId::new(),
                name: event.name,
                email: event.email,
            })
        });

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry
            .expect_user_repository()
            .return_const(repo_arc.clone());

        let registry: AppRegistry = Arc::new(registry);
        let req = CreateUserRequest::new(
            "Alice".to_string(),
            "alice@example.com".to_string(),
            "password123".to_string(),
        );

        let result = register_user(State(registry), Json(req)).await;
        let (status, Json(body)) = result.expect("正常系は成功を期待する");
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body.name, "Alice");
        assert_eq!(body.email, "alice@example.com");
    }

    #[tokio::test]
    async fn ユーザ一覧は200とユーザ配列を返す() {
        let mut repo = MockUserRepository::new();
        repo.expect_find_all().returning(|| {
            Ok(vec![
                User {
                    id: UserId::new(),
                    name: "Alice".to_string(),
                    email: "alice@example.com".to_string(),
                },
                User {
                    id: UserId::new(),
                    name: "Bob".to_string(),
                    email: "bob@example.com".to_string(),
                },
            ])
        });

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);
        let jwt_issuer = Arc::new(JwtIssuer::new("test-secret".to_string(), 60_u64 * 60));
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_valid_auth_header(&jwt_issuer);

        let (status, Json(body)) = list_users(State(registry), headers)
            .await
            .expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.items.len(), 2);
        assert_eq!(body.items[0].name, "Alice");
        assert_eq!(body.items[0].email, "alice@example.com");
        assert_eq!(body.items[1].name, "Bob");
        assert_eq!(body.items[1].email, "bob@example.com");
    }

    #[tokio::test]
    async fn ユーザ削除は204を返す() {
        let user_id = UserId::new();
        let mut repo = MockUserRepository::new();
        repo.expect_delete()
            .withf(move |event| event.id == user_id)
            .returning(|_event| Ok(()));

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);
        let jwt_issuer = Arc::new(JwtIssuer::new("test-secret".to_string(), 60_u64 * 60));
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_valid_auth_header(&jwt_issuer);

        let status = delete_user(State(registry), Path(user_id.to_string()), headers)
            .await
            .expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn ユーザ追加はemail不正で失敗する() {
        let registry = MockAppRegistryExt::new();
        let registry: AppRegistry = Arc::new(registry);
        let req = CreateUserRequest::new(
            "Alice".to_string(),
            "invalid-email".to_string(),
            "password123".to_string(),
        );

        let err = register_user(State(registry), Json(req))
            .await
            .expect_err("バリデーションは失敗する");

        assert!(matches!(err, AppError::ValidationError(_)));
    }

    #[tokio::test]
    async fn ユーザ追加はリポジトリ失敗でエラーになる() {
        let mut repo = MockUserRepository::new();
        repo.expect_create()
            .returning(|_event| Err(AppError::SqlExecuteError(sqlx::Error::RowNotFound)));

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);
        let req = CreateUserRequest::new(
            "Alice".to_string(),
            "alice@example.com".to_string(),
            "password123".to_string(),
        );

        let err = register_user(State(registry), Json(req))
            .await
            .expect_err("リポジトリ失敗はエラーになる");

        assert!(matches!(err, AppError::SqlExecuteError(_)));
    }

    #[tokio::test]
    async fn ユーザ削除は存在しないidで失敗する() {
        let user_id = UserId::new();
        let mut repo = MockUserRepository::new();
        repo.expect_delete()
            .withf(move |event| event.id == user_id)
            .returning(|_event| Err(AppError::EntityNotFoundError("not found".into())));

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);
        let jwt_issuer = Arc::new(JwtIssuer::new("test-secret".to_string(), 60_u64 * 60));
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_valid_auth_header(&jwt_issuer);

        let err = delete_user(State(registry), Path(user_id.to_string()), headers)
            .await
            .expect_err("存在しないユーザは失敗する");

        assert!(matches!(err, AppError::EntityNotFoundError(_)));
    }

    #[tokio::test]
    async fn ユーザ削除は不正なidで失敗する() {
        let mut registry = MockAppRegistryExt::new();
        let jwt_issuer = Arc::new(JwtIssuer::new("test-secret".to_string(), 60_u64 * 60));
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_valid_auth_header(&jwt_issuer);

        let err = delete_user(State(registry), Path("invalid".to_string()), headers)
            .await
            .expect_err("不正なIDは失敗する");

        assert!(matches!(err, AppError::ConvertToUuidError(_)));
    }

    #[tokio::test]
    async fn ユーザ一覧はauthorizationヘッダがないと401を返す() {
        let mut repo = MockUserRepository::new();
        repo.expect_find_all().times(0);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);
        let headers = HeaderMap::new();

        let err = list_users(State(registry), headers)
            .await
            .expect_err("Authorizationヘッダなしは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn ユーザ一覧は不正jwtで401を返す() {
        let mut repo = MockUserRepository::new();
        repo.expect_find_all().times(0);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);
        let jwt_issuer = Arc::new(JwtIssuer::new("correct-secret".to_string(), 60_u64 * 60));
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let wrong_issuer = JwtIssuer::new("wrong-secret".to_string(), 60_u64 * 60);
        let wrong_token = wrong_issuer.issue_token(UserId::new()).expect("jwt生成");
        let headers = build_auth_header(&wrong_token.0);

        let err = list_users(State(registry), headers)
            .await
            .expect_err("不正JWTは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn ユーザ削除はauthorizationヘッダがないと401を返す() {
        let user_id = UserId::new();
        let mut repo = MockUserRepository::new();
        repo.expect_delete().times(0);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);
        let headers = HeaderMap::new();

        let err = delete_user(State(registry), Path(user_id.to_string()), headers)
            .await
            .expect_err("Authorizationヘッダなしは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn 自分情報取得は200とユーザ情報を返す() {
        let user_id = UserId::new();
        let mut repo = MockUserRepository::new();
        let user_id_for_match = user_id;
        repo.expect_find_by_id()
            .withf(move |value| *value == user_id_for_match)
            .returning(move |_| {
                Ok(Some(User {
                    id: user_id,
                    name: "Alice".to_string(),
                    email: "alice@example.com".to_string(),
                }))
            });

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);
        let jwt_issuer = Arc::new(JwtIssuer::new("test-secret".to_string(), 60_u64 * 60));
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_auth_header_for_user(&jwt_issuer, user_id);

        let (status, Json(body)) = get_current_user(State(registry), headers)
            .await
            .expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.id, user_id);
        assert_eq!(body.name, "Alice");
        assert_eq!(body.email, "alice@example.com");
    }

    #[tokio::test]
    async fn 自分情報取得はauthorizationヘッダがないと401を返す() {
        let mut repo = MockUserRepository::new();
        repo.expect_find_by_id().times(0);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);

        let registry: AppRegistry = Arc::new(registry);
        let headers = HeaderMap::new();

        let err = get_current_user(State(registry), headers)
            .await
            .expect_err("Authorizationヘッダなしは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn 自分情報取得は不正jwtで401を返す() {
        let mut repo = MockUserRepository::new();
        repo.expect_find_by_id().times(0);

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);
        let jwt_issuer = Arc::new(JwtIssuer::new("correct-secret".to_string(), 60_u64 * 60));
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let wrong_issuer = JwtIssuer::new("wrong-secret".to_string(), 60_u64 * 60);
        let wrong_token = wrong_issuer.issue_token(UserId::new()).expect("jwt生成");
        let headers = build_auth_header(&wrong_token.0);

        let err = get_current_user(State(registry), headers)
            .await
            .expect_err("不正JWTは401を期待する");

        assert!(matches!(err, AppError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn 自分情報取得はユーザが存在しないと404を返す() {
        let user_id = UserId::new();
        let mut repo = MockUserRepository::new();
        let user_id_for_match = user_id;
        repo.expect_find_by_id()
            .withf(move |value| *value == user_id_for_match)
            .returning(|_| Ok(None));

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);
        let jwt_issuer = Arc::new(JwtIssuer::new("test-secret".to_string(), 60_u64 * 60));
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_auth_header_for_user(&jwt_issuer, user_id);

        let err = get_current_user(State(registry), headers)
            .await
            .expect_err("存在しないユーザは404を期待する");

        assert!(matches!(err, AppError::EntityNotFoundError(_)));
    }

    #[tokio::test]
    async fn パスワード更新は204を返す() {
        let user_id = UserId::new();
        let mut repo = MockUserRepository::new();
        let user_id_for_match = user_id;
        repo.expect_update_password()
            .withf(move |event| {
                event.id == user_id_for_match
                    && event.current_password == "old-password"
                    && event.new_password == "new-password"
            })
            .returning(|_| Ok(()));

        let mut registry = MockAppRegistryExt::new();
        let repo_arc: Arc<dyn UserRepository> = Arc::new(repo);
        registry.expect_user_repository().return_const(repo_arc);
        let jwt_issuer = Arc::new(JwtIssuer::new("test-secret".to_string(), 60_u64 * 60));
        registry
            .expect_jwt_issuer()
            .return_const(jwt_issuer.clone());

        let registry: AppRegistry = Arc::new(registry);
        let headers = build_auth_header_for_user(&jwt_issuer, user_id);
        let req =
            ChangePasswordRequest::new("old-password".to_string(), "new-password".to_string());

        let status = change_password(State(registry), headers, Json(req))
            .await
            .expect("正常系は成功を期待する");

        assert_eq!(status, StatusCode::NO_CONTENT);
    }
}
