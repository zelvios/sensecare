//! Rules and audit live in services::stays.
//!
//! POST   /api/v1/stays                    {room_id, user_id}   ManageStays
//! POST   /api/v1/stays/{id}/check-out                          ManageStays
//! GET    /api/v1/stays                    list                 ViewAllRooms
//! GET    /api/v1/stays/{id}                                    ViewAllRooms
//! GET    /api/v1/stays/me                 caller's open stay   ViewOwnRoom (any role)
//!
//! Query parameters for the list, all optional:
//!   room_id, user_id   uuid
//!   open               true | false
//!   limit              1-200, default 50
//!   offset             default 0

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
    repos::stays::StayWithNames,
    routes::extractors::CurrentUser,
    services::stays::{self, ListParams},
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, check_in))
        .routes(routes!(me))
        .routes(routes!(get_one))
        .routes(routes!(check_out))
}

// --- DTOs ----

#[derive(Serialize, ToSchema)]
pub struct StayResponse {
    pub id: Uuid,
    pub room_id: Uuid,
    pub room_number: String,
    pub user_id: Uuid,
    pub user_display_name: String,
    pub checked_in_at: DateTime<Utc>,
    /// `null` while the stay is open.
    pub checked_out_at: Option<DateTime<Utc>>,
}

impl From<StayWithNames> for StayResponse {
    fn from((s, room_number, user_display_name): StayWithNames) -> Self {
        Self {
            id: s.id,
            room_id: s.room_id,
            room_number,
            user_id: s.user_id,
            user_display_name,
            checked_in_at: s.checked_in_at,
            checked_out_at: s.checked_out_at,
        }
    }
}

#[derive(Deserialize, IntoParams)]
pub struct ListQuery {
    pub room_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    /// true = open stays only, false = closed only
    pub open: Option<bool>,
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
pub struct CheckInRequest {
    pub room_id: Uuid,
    /// Must be an active user with the client role.
    pub user_id: Uuid,
}

// --- handlers ----

/// List stays, newest checkin first.
#[utoipa::path(
    get, path = "/", tag = "stays", operation_id = "list_stays",
    params(ListQuery),
    responses((status = 200, body = Vec<StayResponse>), (status = 403, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn list(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<StayResponse>>, ApiError> {
    let mut conn = state.pool.get().await?;
    let params = ListParams {
        room_id: q.room_id,
        user_id: q.user_id,
        open: q.open,
        limit: q.limit,
        offset: q.offset,
    };
    let stays = stays::list(&mut conn, &actor, params).await?;
    Ok(Json(stays.into_iter().map(Into::into).collect()))
}

/// Check a client into a room.
#[utoipa::path(
    post, path = "/", tag = "stays", operation_id = "check_in",
    request_body = CheckInRequest,
    responses(
        (status = 201, body = StayResponse),
        (status = 400, description = "User or room missing, inactive, or user is not a client", body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 409, description = "Room is occupied or user already has an open stay", body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn check_in(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Json(body): Json<CheckInRequest>,
) -> Result<(StatusCode, Json<StayResponse>), ApiError> {
    let mut conn = state.pool.get().await?;
    let stay = stays::check_in(&mut conn, &actor, body.room_id, body.user_id).await?;
    Ok((StatusCode::CREATED, Json(stay.into())))
}

/// The caller's current open stay. 404 when not checked in anywhere.
#[utoipa::path(
    get, path = "/me", tag = "stays", operation_id = "my_stay",
    responses((status = 200, body = StayResponse), (status = 404, description = "Not checked in", body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn me(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
) -> Result<Json<StayResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(stays::current_for(&mut conn, &actor).await?.into()))
}

/// Get one stay.
#[utoipa::path(
    get, path = "/{id}", tag = "stays", operation_id = "get_stay",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = StayResponse), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn get_one(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<StayResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(stays::get(&mut conn, &actor, id).await?.into()))
}

/// Check the client out. The stay stays in history with its check-out time.
#[utoipa::path(
    post, path = "/{id}/check-out", tag = "stays", operation_id = "check_out",
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, body = StayResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, description = "Already checked out", body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn check_out(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<StayResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(stays::check_out(&mut conn, &actor, id).await?.into()))
}
