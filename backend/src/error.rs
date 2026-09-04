//! The single error type returned by handlers.
//!
//! Every handler returns `Result<T, ApiError>`. axum calls `into_response` on
//! the error, so status codes and JSON bodies are decided here, in one place.
//!
//! Anything that could leak internals (SQL errors, pool failures) is logged
//! with `tracing::error!` and turned into a generic 500 for the client.
//! `?` works on diesel, pool and anyhow errors thanks to the `#[from]` impls.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Database(#[from] diesel::result::Error),
    #[error(transparent)]
    Pool(#[from] diesel_async::pooled_connection::deadpool::PoolError),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let internal = (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal error".to_string(),
        );

        let (status, message) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),

            // Diesel returns this when `.first()` / `.get_result()` finds no row.
            // Mapping it here means repos can use those methods directly and
            // handlers get a 404 for free.
            ApiError::Database(diesel::result::Error::NotFound) => {
                (StatusCode::NOT_FOUND, "not found".to_string())
            }
            ApiError::Database(e) => {
                tracing::error!(error = %e, "database error");
                internal
            }
            ApiError::Pool(e) => {
                tracing::error!(error = %e, "connection pool error");
                internal
            }
            ApiError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                internal
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
