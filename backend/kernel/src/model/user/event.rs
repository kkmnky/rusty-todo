use crate::model::id::UserId;

pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub password: String,
}

pub struct DeleteUser {
    pub id: UserId,
}
