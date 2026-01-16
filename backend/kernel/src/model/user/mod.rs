use crate::model::id::UserId;

pub mod event;

#[derive(Debug)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
}
