//! POST /api/v1/auth/login   {username, password} -> user and session cookie
//! POST /api/v1/auth/logout  (authenticated)        -> deletes the session
//! GET  /api/v1/auth/me      (authenticated)        -> the current user

use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header::USER_AGENT},
    routing::{get, post},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    routes::extractors::{CurrentUser, SESSION_COOKIE},
    services::{self, auth::AuthenticatedUser},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user: AuthenticatedUser,
    /// The same token as the cookie, for the SvelteKit server to forward as
    /// `Authorization: Bearer`. Never store this in browser JavaScript.
    pub session_token: String,
}

async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), ApiError> {
    let user_agent = headers.get(USER_AGENT).and_then(|v| v.to_str().ok());
    let mut conn = state.pool.get().await?;

    let (token, user) = services::auth::login(
        &mut conn,
        &body.username,
        &body.password,
        user_agent,
        state.config.session_ttl_hours,
    )
    .await?;

    let cookie = Cookie::build((SESSION_COOKIE, token.to_string()))
        .http_only(true)
        .secure(state.config.cookie_secure)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::hours(state.config.session_ttl_hours))
        .build();

    Ok((
        jar.add(cookie),
        Json(LoginResponse {
            user,
            session_token: token.to_string(),
        }),
    ))
}

async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    CurrentUser(user): CurrentUser,
) -> Result<(CookieJar, StatusCode), ApiError> {
    let mut conn = state.pool.get().await?;
    services::auth::logout(&mut conn, user.session_id).await?;
    Ok((jar.remove(SESSION_COOKIE), StatusCode::NO_CONTENT))
}

async fn me(CurrentUser(user): CurrentUser) -> Json<AuthenticatedUser> {
    Json(user)
}
