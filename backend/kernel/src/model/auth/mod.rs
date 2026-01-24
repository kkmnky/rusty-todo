use crate::model::id::UserId;

pub mod event;

#[derive(Debug)]
pub struct UserCredential {
    pub id: UserId,
    pub email: String,
    pub password_hash: String,
}

pub struct AccessToken(pub String);
