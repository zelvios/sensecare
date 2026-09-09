//! Rules and audit live in services::rooms.
//!
//! GET    /api/v1/rooms                             list / search                  ViewAllRooms
//! POST   /api/v1/rooms                             create                         ManageRooms
//! GET    /api/v1/rooms/{id}                                                       ViewAllRooms
//! PATCH  /api/v1/rooms/{id}                        room_number/name/floor         ManageRooms
//! POST   /api/v1/rooms/{id}/deactivate                                            ManageRooms
//! POST   /api/v1/rooms/{id}/activate                                              ManageRooms
//! DELETE /api/v1/rooms/{id}                        hard delete                    DeleteRooms (admin)
//! GET    /api/v1/rooms/{id}/measurements           history, default last 24 h     ViewAllRooms or own room
//! GET    /api/v1/rooms/{id}/measurements/latest    most recent reading            ViewAllRooms or own room
//! GET    /api/v1/rooms/{id}/thresholds             effective limits and source    ViewAllRooms or own room
//! PUT    /api/v1/rooms/{id}/thresholds             create or replace override     ManageThresholds
//! DELETE /api/v1/rooms/{id}/thresholds             remove override                ManageThresholds
//!
//! Query parameters for the list, all optional:
//!   q        matches room number or name, case-insensitive
//!   active   true | false
//!   floor    integer
//!   limit    1-200, default 50
//!   offset   default 0

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    error::{ApiError, ErrorResponse},
    models::room::Room,
    routes::{
        extractors::CurrentUser,
        v1::{
            measurements::{HistoryQuery, MeasurementResponse},
            thresholds::{ThresholdRequest, ThresholdResponse},
        },
    },
    services::{
        measurements::{self, HistoryParams},
        rooms::{self, ListParams},
        thresholds::{self, ThresholdSource},
    },
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(get_one, update, delete))
        .routes(routes!(deactivate))
        .routes(routes!(activate))
        .routes(routes!(measurement_history))
        .routes(routes!(latest_measurement))
        .routes(routes!(get_thresholds, set_thresholds, remove_thresholds))
}

#[derive(Deserialize, IntoParams)]
pub struct ListQuery {
    /// Matches room number or name, case-insensitive.
    pub q: Option<String>,
    pub active: Option<bool>,
    pub floor: Option<i16>,
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
pub struct CreateRoomRequest {
    #[schema(example = "12")]
    pub room_number: String,
    #[schema(example = "Stue 12, Kardiologi")]
    pub name: Option<String>,
    #[schema(example = 1)]
    pub floor: Option<i16>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateRoomRequest {
    #[schema(example = "7")]
    pub room_number: Option<String>,
    #[schema(example = "Kardiologi")]
    pub name: Option<String>,
    #[schema(example = "2")]
    pub floor: Option<i16>,
}

/// List or search rooms.
#[utoipa::path(
    get, path = "/", tag = "rooms", operation_id = "list_rooms",
    params(ListQuery),
    responses((status = 200, body = Vec<Room>), (status = 403, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn list(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Room>>, ApiError> {
    let mut conn = state.pool.get().await?;
    let params = ListParams {
        q: q.q.as_deref(),
        active: q.active,
        floor: q.floor,
        limit: q.limit,
        offset: q.offset,
    };
    Ok(Json(rooms::list(&mut conn, &actor, params).await?))
}

/// Create a room.
#[utoipa::path(
    post, path = "/", tag = "rooms", operation_id = "create_room",
    request_body = CreateRoomRequest,
    responses(
        (status = 201, body = Room),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 409, description = "Room number already exists", body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn create(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Json(body): Json<CreateRoomRequest>,
) -> Result<(StatusCode, Json<Room>), ApiError> {
    let mut conn = state.pool.get().await?;
    let room = rooms::create(
        &mut conn,
        &actor,
        &body.room_number,
        body.name.as_deref(),
        body.floor,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(room)))
}

/// One room.
#[utoipa::path(
    get, path = "/{id}", tag = "rooms", operation_id = "get_room",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = Room), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn get_one(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Room>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(rooms::get(&mut conn, &actor, id).await?))
}

/// Change room number, name and/or floor.
#[utoipa::path(
    patch, path = "/{id}", tag = "rooms", operation_id = "update_room",
    params(("id" = Uuid, Path)),
    request_body = UpdateRoomRequest,
    responses(
        (status = 200, body = Room),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn update(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateRoomRequest>,
) -> Result<Json<Room>, ApiError> {
    let mut conn = state.pool.get().await?;
    let room = rooms::update(
        &mut conn,
        &actor,
        id,
        body.room_number.as_deref(),
        body.name.as_deref(),
        body.floor,
    )
    .await?;
    Ok(Json(room))
}

/// Deactivate a room. It stays in history but is hidden from active overviews.
#[utoipa::path(
    post, path = "/{id}/deactivate", tag = "rooms", operation_id = "deactivate_room",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn deactivate(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    rooms::set_active(&mut conn, &actor, id, false).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Reactivate a room.
#[utoipa::path(
    post, path = "/{id}/activate", tag = "rooms", operation_id = "activate_room",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn activate(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    rooms::set_active(&mut conn, &actor, id, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Permanently delete a room (admin only). Refused if anything still references it.
#[utoipa::path(
    delete, path = "/{id}", tag = "rooms", operation_id = "delete_room",
    params(("id" = Uuid, Path)),
    responses(
        (status = 204),
        (status = 400, description = "Still referenced by devices, stays, measurements or calls", body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn delete(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    rooms::delete(&mut conn, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Readings for a room, newest first. Defaults to the last 24 hours.
/// Staff and admins may read any room and a client can only read the room they are checked into.
#[utoipa::path(
    get, path = "/{id}/measurements", tag = "measurements", operation_id = "room_measurements",
    params(("id" = Uuid, Path), HistoryQuery),
    responses(
        (status = 200, body = Vec<MeasurementResponse>),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn measurement_history(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<Vec<MeasurementResponse>>, ApiError> {
    let mut conn = state.pool.get().await?;
    let params = HistoryParams {
        from: q.from,
        to: q.to,
        limit: q.limit,
        offset: q.offset,
    };
    let rows = measurements::history(&mut conn, &actor, id, params).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

/// The most recent reading for a room.
#[utoipa::path(
    get, path = "/{id}/measurements/latest", tag = "measurements", operation_id = "room_latest_measurement",
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, body = MeasurementResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, description = "Room not found or no readings yet", body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn latest_measurement(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<MeasurementResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(
        measurements::latest(&mut conn, &actor, id).await?.into(),
    ))
}

/// The limits in force for this room: its override, or the global default.
#[utoipa::path(
    get, path = "/{id}/thresholds", tag = "thresholds", operation_id = "room_thresholds",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = ThresholdResponse), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn get_thresholds(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ThresholdResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    let (t, source) = thresholds::read_for_room(&mut conn, &actor, id).await?;
    Ok(Json(ThresholdResponse::from_threshold(t, source)))
}

/// Create or replace this room's override (admin).
#[utoipa::path(
    put, path = "/{id}/thresholds", tag = "thresholds", operation_id = "set_room_thresholds",
    params(("id" = Uuid, Path)),
    request_body = ThresholdRequest,
    responses((status = 200, body = ThresholdResponse), (status = 400, body = ErrorResponse), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn set_thresholds(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<ThresholdRequest>,
) -> Result<Json<ThresholdResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    let t = thresholds::set_for_room(&mut conn, &actor, id, body.into()).await?;
    Ok(Json(ThresholdResponse::from_threshold(
        t,
        ThresholdSource::Room,
    )))
}

/// Remove this rooms override, so the global default applies again (admin).
#[utoipa::path(
    delete, path = "/{id}/thresholds", tag = "thresholds", operation_id = "remove_room_thresholds",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 403, body = ErrorResponse), (status = 404, description = "Room has no override", body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn remove_thresholds(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    thresholds::remove_for_room(&mut conn, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
