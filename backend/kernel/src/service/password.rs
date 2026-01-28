use shared::error::{AppError, AppResult};

pub fn hash(password: &str) -> AppResult<String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(AppError::from)
}

pub fn verify(password: &str, password_hash: &str) -> AppResult<bool> {
    bcrypt::verify(password, password_hash).map_err(AppError::from)
}
