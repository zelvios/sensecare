//! Every handler returns `Result<T, ApiError>`. axum calls `into_response` on
//! the error, so status codes and JSON bodies are decided here, in one place.
//!
//! Anything that could leak internals (SQL errors, pool failures) is logged
//! with `tracing::error!` and turned into a generic 500 for the client.
//! `?` works on diesel, pool and anyhow errors because of `#[from]` impls.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("account deactivated")]
    AccountDeactivated,
    /// Authenticated, but the role lacks the required permission.
    #[error("forbidden")]
    Forbidden,
    #[error("bad request: {0}")]
    BadRequest(String),
    /// Business level conflict, e.g. "room already has an active device".
    #[error("conflict: {0}")]
    Conflict(String),
    #[error(transparent)]
    Database(#[from] DieselError),
    #[error(transparent)]
    Pool(#[from] diesel_async::pooled_connection::deadpool::PoolError),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        const INTERNAL: (StatusCode, &str, &str) = (
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            "internal error",
        );

        let (status, code, message): (StatusCode, &str, String) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not_found", self.to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", self.to_string()),
            ApiError::AccountDeactivated => (
                StatusCode::FORBIDDEN,
                "account_deactivated",
                self.to_string(),
            ),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "forbidden", self.to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg.clone()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),

            ApiError::Database(DieselError::NotFound) => {
                (StatusCode::NOT_FOUND, "not_found", "not found".to_string())
            }
            ApiError::Database(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                info,
            )) => (
                StatusCode::CONFLICT,
                "already_exists",
                info.message().to_string(),
            ),
            ApiError::Database(DieselError::DatabaseError(
                DatabaseErrorKind::ForeignKeyViolation,
                info,
            )) => (
                StatusCode::BAD_REQUEST,
                "invalid_reference",
                info.message().to_string(),
            ),
            ApiError::Database(e) => {
                tracing::error!(error = %e, "database error");
                (INTERNAL.0, INTERNAL.1, INTERNAL.2.to_string())
            }
            ApiError::Pool(e) => {
                tracing::error!(error = %e, "connection pool error");
                (INTERNAL.0, INTERNAL.1, INTERNAL.2.to_string())
            }
            ApiError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (INTERNAL.0, INTERNAL.1, INTERNAL.2.to_string())
            }
        };

        (status, Json(json!({ "error": code, "message": message }))).into_response()
    }
}
