use kernel::model::{auth::UserCredential, id::UserId};
use shared::error::AppError;

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
