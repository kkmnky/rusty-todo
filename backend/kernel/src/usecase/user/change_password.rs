use std::sync::Arc;

use shared::error::AppResult;

use crate::{
    model::{id::UserId, user::event::UpdatePassword},
    repository::user::UserRepository,
};

pub struct ChangePasswordInput {
    pub id: UserId,
    pub current_password: String,
    pub new_password: String,
}

impl From<ChangePasswordInput> for UpdatePassword {
    fn from(value: ChangePasswordInput) -> Self {
        Self {
            id: value.id,
            current_password: value.current_password,
            new_password: value.new_password,
        }
    }
}
pub struct ChangePasswordUsecase {
    user_repository: Arc<dyn UserRepository>,
}

impl ChangePasswordUsecase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, input: ChangePasswordInput) -> AppResult<()> {
        self.user_repository.update_password(input.into()).await?;
        Ok(())
    }
}
