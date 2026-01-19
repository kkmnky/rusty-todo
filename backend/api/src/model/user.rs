use derive_new::new;
use garde::Validate;
use kernel::model::{
    id::UserId,
    user::{User, event::CreateUser},
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
pub struct CreateUserRequest {
    #[garde(length(min = 1))]
    name: String,
    #[garde(email)]
    email: String,
    #[garde(length(min = 1))]
    password: String,
}

impl From<CreateUserRequest> for CreateUser {
    fn from(value: CreateUserRequest) -> Self {
        let CreateUserRequest {
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
