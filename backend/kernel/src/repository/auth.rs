use crate::model::auth::UserCredential;
use async_trait::async_trait;
use shared::error::AppResult;

#[mockall::automock]
#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn find_by_email(&self, email: String) -> AppResult<Option<UserCredential>>;
}
