use std::sync::Arc;

use crate::{
    model::{
        auth::{AccessToken, mutations::StoreToken},
        id::UserId,
    },
    repository::auth::AuthRepository,
    service::{jwt::JwtIssuer, password},
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

pub struct LoginUsecase {
    auth_repository: Arc<dyn AuthRepository>,
    jwt_issuer: Arc<JwtIssuer>,
}

impl LoginUsecase {
    pub fn new(auth_repository: Arc<dyn AuthRepository>, jwt_issuer: Arc<JwtIssuer>) -> Self {
        Self {
            auth_repository,
            jwt_issuer,
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

        let access_token = self.jwt_issuer.issue_token(credential.id)?;
        let stored_token = self
            .auth_repository
            .store_token(StoreToken {
                user_id: credential.id,
                access_token,
            })
            .await?;

        Ok(LoginOutput {
            access_token: stored_token,
            expires_in: self.jwt_issuer.ttl(),
            user_id: credential.id,
        })
    }
}
