#[cfg(feature = "axum")]
extern crate axum;

#[cfg(feature = "axum")]
extern crate serde;

use axum::Json;
use thiserror::Error;

pub type Result<T> = anyhow::Result<T, BlogiError>;
pub type Success = Result<()>;

#[derive(Debug, Error)]
pub enum BlogiError {
    #[error("not found")]
    NotFound,

    #[error("internal server error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("database error: {0}")]
    DbErr(#[from] sqlx::Error)
}

#[cfg(feature = "axum")]
#[cfg_attr(feature = "axum", derive(serde::Serialize))]
pub struct XrpcErrorResponse {
    pub error: String,
    pub message: Option<String>,
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for BlogiError {
    fn into_response(self) -> axum::response::Response {
        let error = match self {
            BlogiError::NotFound => "NotFound",
            _ => "InternalServerError",
        }.to_string();

        let message = match self {
            BlogiError::NotFound => Some("The requested resource was not found.".to_string()),
            _ => None,
        };

        let code = match self {
            BlogiError::NotFound => axum::http::StatusCode::NOT_FOUND,
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        ((code, Json(XrpcErrorResponse { error, message }))).into_response()
    }
}
