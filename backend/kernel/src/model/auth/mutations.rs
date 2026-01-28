use crate::model::{auth::AccessToken, id::UserId};

pub struct StoreToken {
    pub user_id: UserId,
    pub access_token: AccessToken,
}
