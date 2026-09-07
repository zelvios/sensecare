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
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::role::Role,
    routes::extractors::CurrentUser,
    services::users::{self, ListParams, UserRecord},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", get(get_one).patch(update).delete(delete))
        .route("/{id}/password", post(set_password))
        .route("/{id}/deactivate", post(deactivate))
        .route("/{id}/activate", post(activate))
}

// --- DTOs ---

/// What the API returns for a user. Never includes the password hash.
#[derive(Serialize)]
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

#[derive(Deserialize)]
pub struct ListQuery {
    pub q: Option<String>,
    pub role: Option<Role>,
    pub active: Option<bool>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub display_name: String,
    pub password: String,
    pub role: Role,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub role: Option<Role>,
}

#[derive(Deserialize)]
pub struct SetPasswordRequest {
    pub password: String,
}

// --- handlers ----

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

async fn get_one(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(users::get(&mut conn, &actor, id).await?.into()))
}

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

async fn deactivate(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    users::set_active(&mut conn, &actor, id, false).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn activate(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    users::set_active(&mut conn, &actor, id, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    users::delete(&mut conn, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
