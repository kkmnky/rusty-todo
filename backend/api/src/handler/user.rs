use axum::{Json, extract::State, http::StatusCode};
use garde::Validate;
use registry::AppRegistry;

use crate::model::user::{CreateUserRequest, UserResponse};
use shared::error::AppResult;

pub async fn register_user(
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<UserResponse>)> {
    req.validate()?;

    let registered_user = registry.user_repository().create(req.into()).await?;

    Ok((StatusCode::CREATED, Json(registered_user.into())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use kernel::model::{id::UserId, user::User};
    use kernel::repository::user::{MockUserRepository, UserRepository};
    use registry::MockAppRegistryExt;
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
        registry.expect_user_repository().return_const(repo_arc.clone());

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
}
