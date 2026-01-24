use crate::model::id::UserId;

pub struct StoreToken {
    pub user_id: UserId,
    pub access_token: String,
}
