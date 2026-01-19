use chrono::{DateTime, Utc};
use kernel::model::{id::UserId, user::User};
use shared::error::AppError;

pub struct UserRow {
    pub id: UserId,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = AppError;

    fn try_from(value: UserRow) -> Result<Self, Self::Error> {
        Ok(User {
            id: value.id,
            name: value.name,
            email: value.email,
        })
    }
}
