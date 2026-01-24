use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    ValidationError(#[from] garde::Report),
    #[error("{0}")]
    ConvertToUuidError(#[from] uuid::Error),
    #[error("{0}")]
    HashPasswordError(#[from] bcrypt::BcryptError),
    #[error("{0}")]
    EntityNotFoundError(String),
    #[error("SQL execution failed.")]
    SqlExecuteError(#[source] sqlx::Error),
    #[error("No rows affected: {0}")]
    NoRowsAffectedError(String),
    #[error("{0}")]
    KeyValueStoreError(#[from] redis::RedisError),
    #[error("{0}")]
    ConversionEntityError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self {
            AppError::EntityNotFoundError(_) => StatusCode::NOT_FOUND,
            AppError::ConvertToUuidError(_) => StatusCode::BAD_REQUEST,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::HashPasswordError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SqlExecuteError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NoRowsAffectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::KeyValueStoreError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ConversionEntityError(_) => StatusCode::BAD_REQUEST,
        };
        status_code.into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
