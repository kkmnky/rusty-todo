use std::sync::Arc;

use shared::error::AppResult;

use crate::{
    model::user::{User, event::CreateUser},
    repository::user::UserRepository,
};

pub struct RegisterUserInput {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl From<RegisterUserInput> for CreateUser {
    fn from(value: RegisterUserInput) -> Self {
        Self {
            name: value.name,
            email: value.email,
            password: value.password,
        }
    }
}

pub struct RegisterUserUsecase {
    user_repository: Arc<dyn UserRepository>,
}

impl RegisterUserUsecase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, input: RegisterUserInput) -> AppResult<User> {
        self.user_repository.create(input.into()).await
    }
}
