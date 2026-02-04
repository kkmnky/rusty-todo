use std::sync::Arc;

use shared::error::AppResult;

use crate::{model::auth::AccessToken, repository::auth::AuthRepository};

pub struct LogoutUsecase {
    auth_repository: Arc<dyn AuthRepository>,
}

impl LogoutUsecase {
    pub fn new(auth_repository: Arc<dyn AuthRepository>) -> Self {
        Self { auth_repository }
    }

    pub async fn execute(&self, access_token: AccessToken) -> AppResult<()> {
        self.auth_repository.delete_token(access_token).await?;
        Ok(())
    }
}
