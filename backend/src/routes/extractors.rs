use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::{error::ApiError, services::auth::AuthenticatedUser, state::AppState};

pub const SESSION_COOKIE: &str = "sensecare_session";

pub struct CurrentUser(pub AuthenticatedUser);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let token = bearer_token(parts)
            .or_else(|| cookie_token(parts))
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

fn cookie_token(parts: &Parts) -> Option<String> {
    let jar = CookieJar::from_headers(&parts.headers);
    jar.get(SESSION_COOKIE).map(|c| c.value().to_owned())
}
