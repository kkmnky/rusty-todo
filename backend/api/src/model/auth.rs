use derive_new::new;
use garde::Validate;
use kernel::model::{auth::AccessToken, id::UserId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Validate, new)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    #[garde(email)]
    pub email: String,
    #[garde(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct AccessTokenResponse {
    pub access_token: AccessToken,
    pub expires_in: u64,
    pub user_id: UserId,
}
