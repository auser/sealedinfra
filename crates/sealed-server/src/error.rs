use axum::{http::StatusCode, Json};
use sealed_common::error::SealedError;
use serde_json::Value;
use thiserror::Error;

pub type SealedServerResult<T = (), E = SealedServerError> = Result<T, E>;

#[derive(Debug, Error)]
pub enum SealedServerError {
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Database error: {0}")]
    DatabaseError(sealed_database::error::SealedDatabaseError),
}

impl From<SealedServerError> for SealedError {
    fn from(error: SealedServerError) -> Self {
        SealedError::ServerError(error.to_string())
    }
}

impl From<axum::http::StatusCode> for SealedServerError {
    fn from(status: axum::http::StatusCode) -> Self {
        SealedServerError::ServerError(status.to_string())
    }
}

impl From<(StatusCode, Json<Value>)> for SealedServerError {
    fn from(status: (StatusCode, Json<Value>)) -> Self {
        SealedServerError::ServerError(status.1.to_string())
    }
}

impl From<sealed_database::error::SealedDatabaseError> for SealedServerError {
    fn from(err: sealed_database::error::SealedDatabaseError) -> Self {
        SealedServerError::DatabaseError(err)
    }
}
