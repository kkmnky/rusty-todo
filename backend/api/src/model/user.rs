use derive_new::new;
use garde::Validate;
use kernel::{
    model::{id::UserId, user::User},
    usecase::user::register::RegisterUserInput,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: UserId,
    pub name: String,
    pub email: String,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        let User { id, name, email } = value;
        Self { id, name, email }
    }
}

#[derive(Debug, Serialize)]
pub struct UsersResponse {
    pub items: Vec<UserResponse>,
}

#[derive(Deserialize, Validate, new)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    #[garde(length(min = 1))]
    pub current_password: String,
    #[garde(length(min = 1))]
    pub new_password: String,
}

#[derive(Deserialize, Validate, new)]
#[serde(rename_all = "camelCase")]
pub struct RegisterUserRequest {
    #[garde(length(min = 1))]
    name: String,
    #[garde(email)]
    email: String,
    #[garde(length(min = 1))]
    password: String,
}

impl From<RegisterUserRequest> for RegisterUserInput {
    fn from(value: RegisterUserRequest) -> Self {
        let RegisterUserRequest {
            name,
            email,
            password,
        } = value;
        Self {
            name,
            email,
            password,
        }
    }
}
