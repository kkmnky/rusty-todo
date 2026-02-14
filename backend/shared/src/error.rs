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
    Unauthorized(String),
    #[error("{0}")]
    EntityNotFoundError(String),
    #[error("SQL execution failed.")]
    SqlExecuteError(#[source] sqlx::Error),
    #[error("Transaction failed.")]
    TransactionError(#[source] sqlx::Error),
    #[error("No rows affected: {0}")]
    NoRowsAffectedError(String),
    #[error("{0}")]
    KeyValueStoreError(#[from] redis::RedisError),
    #[error("{0}")]
    ConversionEntityError(String),
    #[error("{0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::EntityNotFoundError(_) => StatusCode::NOT_FOUND,
            AppError::ConvertToUuidError(_) => StatusCode::BAD_REQUEST,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::HashPasswordError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::SqlExecuteError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::TransactionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NoRowsAffectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::KeyValueStoreError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JwtError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ConversionEntityError(_) => StatusCode::BAD_REQUEST,
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            AppError::Unauthorized(_) => "Unauthorized",
            AppError::EntityNotFoundError(_) => "EntityNotFoundError",
            AppError::ConvertToUuidError(_) => "ConvertToUuidError",
            AppError::ValidationError(_) => "ValidationError",
            AppError::HashPasswordError(_) => "HashPasswordError",
            AppError::SqlExecuteError(_) => "SqlExecuteError",
            AppError::TransactionError(_) => "TransactionError",
            AppError::NoRowsAffectedError(_) => "NoRowsAffectedError",
            AppError::KeyValueStoreError(_) => "KeyValueStoreError",
            AppError::JwtError(_) => "JwtError",
            AppError::ConversionEntityError(_) => "ConversionEntityError",
        }
    }

    pub fn safe_message(&self) -> String {
        self.to_string()
    }

    pub fn cause_chain(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut cur = std::error::Error::source(self);
        while let Some(e) = cur {
            out.push(e.to_string());
            cur = e.source();
        }
        out
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let kind = self.kind();
        let msg = self.safe_message();
        let causes = self.cause_chain();

        if status.is_server_error() {
            tracing::error!(error.kind = kind, error.message = %msg, error.cause_chain = ?causes, status = status.as_u16(), "request failed");
        } else if status.is_client_error() {
            tracing::warn!(error.kind = kind, error.message = %msg, error.cause_chain = ?causes, status = status.as_u16(), "request failed");
        }
        status.into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
