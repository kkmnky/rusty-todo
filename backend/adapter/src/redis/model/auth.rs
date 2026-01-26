use kernel::model::{
    auth::{AccessToken, event::StoreToken},
    id::UserId,
};
use shared::error::{AppError, AppResult};
use std::str::FromStr;

use crate::redis::model::{RedisKey, RedisValue};

pub struct AuthorizationKey(AccessToken);

pub struct AuthorizationUserId(UserId);

pub fn from(event: StoreToken) -> (AuthorizationKey, AuthorizationUserId) {
    (
        AuthorizationKey(event.access_token),
        AuthorizationUserId(event.user_id),
    )
}

impl From<AuthorizationKey> for AccessToken {
    fn from(key: AuthorizationKey) -> Self {
        key.0
    }
}

impl From<AccessToken> for AuthorizationKey {
    fn from(token: AccessToken) -> Self {
        Self(token)
    }
}

impl RedisKey for AuthorizationKey {
    type Value = AuthorizationUserId;

    fn inner(&self) -> String {
        self.0 .0.clone()
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
