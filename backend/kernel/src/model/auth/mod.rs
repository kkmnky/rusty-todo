use crate::model::id::UserId;

#[derive(Debug)]
pub struct UserCredential {
    pub id: UserId,
    pub email: String,
    pub password_hash: String,
}
