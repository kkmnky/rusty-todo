use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use garde::Validate;
use kernel::model::{id::UserId, user::event::DeleteUser};
use registry::AppRegistry;

use crate::model::user::{CreateUserRequest, UserResponse, UsersResponse};
use shared::error::AppResult;

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
) -> AppResult<(StatusCode, Json<UsersResponse>)> {
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
    Path(user_id): Path<UserId>,
) -> AppResult<StatusCode> {
    registry
        .user_repository()
        .delete(DeleteUser { id: user_id })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Path;
    use axum::extract::State;
    use kernel::model::{id::UserId, user::User};
    use kernel::repository::user::{MockUserRepository, UserRepository};
    use registry::MockAppRegistryExt;
    use shared::error::AppError;
    use std::sync::Arc;

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

        let registry: AppRegistry = Arc::new(registry);

        let (status, Json(body)) = list_users(State(registry))
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

        let registry: AppRegistry = Arc::new(registry);

        let status = delete_user(State(registry), Path(user_id))
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
}
