use kernel::model::{
    auth::{AccessToken, UserCredential, event::StoreToken},
    id::UserId,
};
use shared::error::{AppError, AppResult};
use std::str::FromStr;

use crate::redis::model::{RedisKey, RedisValue};

pub struct UserCredentialRow {
    pub id: UserId,
    pub email: String,
    pub password_hash: String,
}

impl TryFrom<UserCredentialRow> for UserCredential {
    type Error = AppError;

    fn try_from(value: UserCredentialRow) -> Result<Self, Self::Error> {
        Ok(UserCredential {
            id: value.id,
            email: value.email,
            password_hash: value.password_hash,
        })
    }
}

pub struct AuthorizationKey(String);

pub struct AuthorizationUserId(UserId);

pub fn from(event: StoreToken) -> (AuthorizationKey, AuthorizationUserId) {
    (
        AuthorizationKey(event.access_token),
        AuthorizationUserId(event.user_id),
    )
}

impl From<AuthorizationKey> for AccessToken {
    fn from(key: AuthorizationKey) -> Self {
        Self(key.0)
    }
}

impl RedisKey for AuthorizationKey {
    type Value = AuthorizationUserId;

    fn inner(&self) -> String {
        self.0.clone()
    }
}

impl RedisValue for AuthorizationUserId {
    fn inner(&self) -> String {
        self.0.to_string()
    }
}

impl TryFrom<String> for AuthorizationUserId {
    type Error = AppError;

    fn try_from(s: String) -> AppResult<Self> {
        Ok(Self(UserId::from_str(&s).map_err(|e| {
            AppError::ConversionEntityError(e.to_string())
        })?))
    }
}
