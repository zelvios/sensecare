//! Rules and audit live in services::users.
//!
//! GET    /api/v1/users                    list / search
//! POST   /api/v1/users                    create
//! GET    /api/v1/users/{id}
//! PATCH  /api/v1/users/{id}               display_name and/or role
//! POST   /api/v1/users/{id}/password      set a new password
//! POST   /api/v1/users/{id}/deactivate    soft delete
//! POST   /api/v1/users/{id}/activate
//! DELETE /api/v1/users/{id}               hard delete (admin only)
//!
//! Query parameters for the list, all optional:
//!   q        matches username or display name, case-insensitive
//!   role     client | staff | admin
//!   active   true | false
//!   limit    1-200, default 50
//!   offset   default 0
//!
//! Examples:
//!   /api/v1/users?q=jens
//!   /api/v1/users?role=staff&active=true
//!   /api/v1/users?offset=50
//!
//! Staff manage clients and admins manage everyone.

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    error::{ApiError, ErrorResponse},
    models::role::Role,
    routes::extractors::CurrentUser,
    services::users::{self, ListParams, UserRecord},
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(get_one, update, delete))
        .routes(routes!(set_password))
        .routes(routes!(deactivate))
        .routes(routes!(activate))
}

// --- DTOs ----

/// What the API returns for a user. Never includes the password hash.
#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRecord> for UserResponse {
    fn from(r: UserRecord) -> Self {
        Self {
            id: r.user.id,
            username: r.user.username,
            display_name: r.user.display_name,
            role: r.role,
            is_active: r.user.is_active,
            last_login_at: r.user.last_login_at,
            created_at: r.user.created_at,
            updated_at: r.user.updated_at,
        }
    }
}

#[derive(Deserialize, IntoParams)]
pub struct ListQuery {
    /// Matches username or display name, case-insensitive.
    pub q: Option<String>,
    pub role: Option<Role>,
    pub active: Option<bool>,
    /// 1-200
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    #[schema(example = "hansen")]
    pub username: String,
    #[schema(example = "Sygeplejerske Hansen")]
    pub display_name: String,
    #[schema(example = "hansen-pass-1")]
    pub password: String,
    pub role: Role,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub role: Option<Role>,
}

#[derive(Deserialize, ToSchema)]
pub struct SetPasswordRequest {
    #[schema(example = "new-pass-12345")]
    pub password: String,
}

// --- handlers ----

/// List or search accounts. Staff see clients only.
#[utoipa::path(
    get,
    path = "/",
    tag = "users",
    params(ListQuery),
    responses((status = 200, body = Vec<UserResponse>), (status = 403, body = ErrorResponse)),
    security(("session_cookie" = []), ("bearer" = []))
)]
async fn list(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<UserResponse>>, ApiError> {
    let mut conn = state.pool.get().await?;
    let params = ListParams {
        q: q.q.as_deref(),
        role: q.role,
        active: q.active,
        limit: q.limit,
        offset: q.offset,
    };
    let users = users::list(&mut conn, &actor, params).await?;
    Ok(Json(users.into_iter().map(Into::into).collect()))
}

/// Create an account. The requested role decides which permission is needed.
#[utoipa::path(
    post,
    path = "/",
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, body = UserResponse),
        (status = 400, description = "Validation failed", body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 409, description = "Username already exists", body = ErrorResponse),
    ),
    security(("session_cookie" = []), ("bearer" = []))
)]
async fn create(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Json(body): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    let mut conn = state.pool.get().await?;
    let created = users::create(
        &mut conn,
        &actor,
        &body.username,
        &body.display_name,
        &body.password,
        body.role,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(created.into())))
}

/// One account.
#[utoipa::path(
    get,
    path = "/{id}",
    tag = "users",
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, body = UserResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
    ),
    security(("session_cookie" = []), ("bearer" = []))
)]
async fn get_one(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(users::get(&mut conn, &actor, id).await?.into()))
}

/// Change display name and/or role. A role change needs permission for both the old and new role.
#[utoipa::path(
    patch,
    path = "/{id}",
    tag = "users",
    params(("id" = Uuid, Path)),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, body = UserResponse),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
    ),
    security(("session_cookie" = []), ("bearer" = []))
)]
async fn update(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    let updated = users::update(
        &mut conn,
        &actor,
        id,
        body.display_name.as_deref(),
        body.role,
    )
    .await?;
    Ok(Json(updated.into()))
}

/// Set a new password. Logs the account out everywhere.
#[utoipa::path(
    post,
    path = "/{id}/password",
    tag = "users",
    params(("id" = Uuid, Path)),
    request_body = SetPasswordRequest,
    responses((status = 204), (status = 400, body = ErrorResponse), (status = 403, body = ErrorResponse)),
    security(("session_cookie" = []), ("bearer" = []))
)]
async fn set_password(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<SetPasswordRequest>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    users::set_password(&mut conn, &actor, id, &body.password).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Soft delete: the account can no longer log in and its sessions are removed.
#[utoipa::path(
    post,
    path = "/{id}/deactivate",
    tag = "users",
    params(("id" = Uuid, Path)),
    responses(
        (status = 204),
        (status = 400, description = "Cannot deactivate your own account", body = ErrorResponse),
        (status = 403, body = ErrorResponse),
    ),
    security(("session_cookie" = []), ("bearer" = []))
)]
async fn deactivate(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    users::set_active(&mut conn, &actor, id, false).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Reactivate a deactivated account.
#[utoipa::path(
    post,
    path = "/{id}/activate",
    tag = "users",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 403, body = ErrorResponse)),
    security(("session_cookie" = []), ("bearer" = []))
)]
async fn activate(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    users::set_active(&mut conn, &actor, id, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Permanently delete an account (admin only). Refused if anything still references it.
#[utoipa::path(
    delete,
    path = "/{id}",
    tag = "users",
    params(("id" = Uuid, Path)),
    responses(
        (status = 204),
        (status = 400, description = "Still referenced by other records", body = ErrorResponse),
        (status = 403, body = ErrorResponse),
    ),
    security(("session_cookie" = []), ("bearer" = []))
)]
async fn delete(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    users::delete(&mut conn, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
