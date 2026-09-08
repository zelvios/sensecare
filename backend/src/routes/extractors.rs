//! The token is read from `Authorization: Bearer <uuid>`. The SvelteKit server
//! forwards it on every request and the browser itself never talks to the API.

use axum::{extract::FromRequestParts, http::request::Parts};
use uuid::Uuid;

use crate::{error::ApiError, services::auth::AuthenticatedUser, state::AppState};

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
