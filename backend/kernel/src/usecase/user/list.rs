use std::sync::Arc;

use shared::error::AppResult;

use crate::{model::user::User, repository::user::UserRepository};

pub struct ListUsersUsecase {
    user_repository: Arc<dyn UserRepository>,
}

impl ListUsersUsecase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self) -> AppResult<Vec<User>> {
        self.user_repository.find_all().await
    }
}
