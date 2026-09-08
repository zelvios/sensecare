//! Authentication extractors.
//!
//!   CurrentUser  a person. Reads `Authorization: Bearer <session token>`.
//!                The SvelteKit server forwards it on every request.
//!   DeviceAuth   a room node. Reads `X-Device-Id` and `X-Device-Key`.
//!
//! Authorisation for people is a separate step inside the handler:
//!   user.require(Permission::ViewAllRooms)?;

use axum::{extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::device::Device,
    services::{auth::AuthenticatedUser, devices},
    state::AppState,
};

// --- people ----

pub struct CurrentUser(pub AuthenticatedUser);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let token = bearer_token(parts)
            .and_then(|t| t.parse::<Uuid>().ok())
            .ok_or(ApiError::Unauthorized)?;

        let mut conn = state.pool.get().await?;
        let user = crate::services::auth::authenticate(&mut conn, token).await?;
        Ok(CurrentUser(user))
    }
}

fn bearer_token(parts: &Parts) -> Option<String> {
    parts
        .headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::to_owned)
}

// --- devices ----

/// The device behind a request from firmware.
pub struct DeviceAuth(pub Device);

impl FromRequestParts<AppState> for DeviceAuth {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let header = |name: &str| parts.headers.get(name).and_then(|v| v.to_str().ok());

        let id = header("x-device-id")
            .and_then(|v| v.parse::<Uuid>().ok())
            .ok_or(ApiError::Unauthorized)?;
        let key = header("x-device-key")
            .ok_or(ApiError::Unauthorized)?
            .to_owned();

        let mut conn = state.pool.get().await?;
        let device = devices::authenticate_device(&mut conn, id, &key).await?;
        Ok(DeviceAuth(device))
    }
}
