use axum::response::{IntoResponse, Response};
use http::StatusCode;
use thiserror::Error;

use super::token_error::TokenError;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum AppError {
    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error(transparent)]
    TokenError(#[from] TokenError),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response(),
            AppError::TokenError(error) => error.into_response(),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()).into_response(),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.to_string()).into_response(),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.to_string()).into_response(),
            AppError::ConfigurationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()).into_response(),
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()).into_response(),
        }
    }
}

impl From<oauth2::ConfigurationError> for AppError {
    fn from(err: oauth2::ConfigurationError) -> Self {
        Self::ConfigurationError(format!("{:?}", err))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError(format!("{:?}", err))
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::InternalServerError(err.to_string())
    }
}
