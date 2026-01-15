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
    use registry::MockAppRegistryExt;
    use std::sync::Arc;

    #[tokio::test]
    async fn ユーザ追加は201と必要項目を返す() {
        let registry: AppRegistry = Arc::new(MockAppRegistryExt::new());
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
