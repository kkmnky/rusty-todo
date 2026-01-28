use serde::Serialize;

use crate::model::id::UserId;

pub mod mutations;

#[derive(Debug)]
pub struct UserCredential {
    pub id: UserId,
    pub email: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AccessToken(pub String);
