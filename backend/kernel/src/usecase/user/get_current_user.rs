use std::sync::Arc;

use shared::error::{AppError, AppResult};

use crate::{
    model::{id::UserId, user::User},
    repository::user::UserRepository,
};

pub struct GetCurrentUserUsecase {
    user_repository: Arc<dyn UserRepository>,
}

impl GetCurrentUserUsecase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, user_id: UserId) -> AppResult<User> {
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::EntityNotFoundError("user not found".into()))?;
        Ok(user)
    }
}
