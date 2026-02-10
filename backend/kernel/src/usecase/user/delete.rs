use std::sync::Arc;

use shared::error::AppResult;

use crate::{
    model::{id::UserId, user::event::DeleteUser},
    repository::user::UserRepository,
};

pub struct DeleteUserInput {
    pub id: UserId,
}

impl From<DeleteUserInput> for DeleteUser {
    fn from(value: DeleteUserInput) -> Self {
        Self { id: value.id }
    }
}

pub struct DeleteUserUsecase {
    user_repository: Arc<dyn UserRepository>,
}

impl DeleteUserUsecase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, input: DeleteUserInput) -> AppResult<()> {
        self.user_repository.delete(input.into()).await?;
        Ok(())
    }
}
