use axum::response::{IntoResponse, Response};
use http::StatusCode;
use thiserror::Error;


#[derive(Debug, Error, Eq, PartialEq)]
pub enum TokenError {
    #[error("Invalid token")]
    InvalidToken,

    #[error("Missing Bearer token")]
    MissingToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Token error: {0}")]
    GenericTokenError(String),
}

impl IntoResponse for TokenError {
    fn into_response(self) -> Response {
        let status: StatusCode = match &self {
            TokenError::MissingToken => StatusCode::UNAUTHORIZED,
            TokenError::InvalidToken => StatusCode::UNAUTHORIZED,
            TokenError::TokenExpired => StatusCode::UNAUTHORIZED,
            TokenError::GenericTokenError(_) => StatusCode::UNAUTHORIZED,
        };

        tracing::error!("Token error: {:#}", self);

        (status, self.to_string()).into_response()
    }
}
