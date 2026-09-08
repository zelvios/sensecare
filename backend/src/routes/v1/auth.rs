//! POST /api/v1/auth/login   {username, password} -> user and session token
//! POST /api/v1/auth/logout  (authenticated)        -> deletes the session
//! GET  /api/v1/auth/me      (authenticated)        -> the current user
//!
//! The API is bearer-only: the token from login is sent as `Authorization: Bearer <token>` on
//! every request. The SvelteKit server keeps it in an HttpOnly cookie for the browser and
//! the API itself sets no cookies.

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header::USER_AGENT},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    error::{ApiError, ErrorResponse},
    routes::extractors::CurrentUser,
    services::{self, auth::AuthenticatedUser},
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(login))
        .routes(routes!(logout))
        .routes(routes!(me))
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "admin")]
    pub username: String,
    #[schema(example = "admin-change-me")]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub user: AuthenticatedUser,
    /// Send as `Authorization: Bearer <session_token>` on every request.
    /// The SvelteKit server stores it in an HttpOnly cookie and never expose it to browser JavaScript.
    pub session_token: String,
}

/// Log in. Returns the user and a session token.
#[utoipa::path(
    post, path = "/login",tag = "auth", request_body = LoginRequest, operation_id = "login",
    responses(
        (status = 200, body = LoginResponse),
        (status = 401, description = "Wrong username or password", body = ErrorResponse),
        (status = 403, description = "Account deactivated", body = ErrorResponse),
    )
)]
async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
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

    Ok(Json(LoginResponse {
        user,
        session_token: token.to_string(),
    }))
}

/// Log out: deletes the session so the token stops working.
#[utoipa::path(
    post, path = "/logout", tag = "auth", operation_id = "logout",
    responses((status = 204), (status = 401, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn logout(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    services::auth::logout(&mut conn, &user).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// The currently authenticated user.
#[utoipa::path(
    get, path = "/me", tag = "auth", operation_id = "me",
    responses((status = 200, body = AuthenticatedUser), (status = 401, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn me(CurrentUser(user): CurrentUser) -> Json<AuthenticatedUser> {
    Json(user)
}
