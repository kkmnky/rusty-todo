use std::sync::Arc;

use crate::{
    model::{auth::AccessToken, auth::mutations::StoreToken, id::UserId},
    repository::auth::AuthRepository,
    service::password,
};
use shared::error::{AppError, AppResult};

pub struct LoginInput {
    pub email: String,
    pub password: String,
}

pub struct LoginOutput {
    pub access_token: AccessToken,
    pub expires_in: u64,
    pub user_id: UserId,
}

pub trait AccessTokenGenerator: Send + Sync {
    fn generate(&self, user_id: UserId, expires_in: u64) -> AppResult<AccessToken>;
}

pub struct LoginUsecase {
    auth_repository: Arc<dyn AuthRepository>,
    token_generator: Arc<dyn AccessTokenGenerator>,
    expires_in: u64,
}

impl LoginUsecase {
    pub fn new(
        auth_repository: Arc<dyn AuthRepository>,
        token_generator: Arc<dyn AccessTokenGenerator>,
        expires_in: u64,
    ) -> Self {
        Self {
            auth_repository,
            token_generator,
            expires_in,
        }
    }

    pub async fn execute(&self, input: LoginInput) -> AppResult<LoginOutput> {
        let credential = self
            .auth_repository
            .find_by_email(input.email)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;

        let verified = password::verify(&input.password, &credential.password_hash)?;
        if !verified {
            return Err(AppError::Unauthorized("Invalid credentials".into()));
        }

        let access_token = self
            .token_generator
            .generate(credential.id, self.expires_in)?;
        let stored_token = self
            .auth_repository
            .store_token(StoreToken {
                user_id: credential.id,
                access_token,
            })
            .await?;

        Ok(LoginOutput {
            access_token: stored_token,
            expires_in: self.expires_in,
            user_id: credential.id,
        })
    }
}
