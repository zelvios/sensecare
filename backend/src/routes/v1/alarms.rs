//! Alarms: raised by the system when readings break a room's limits.
//! Rules live in services::alarms.
//!
//! GET   /api/v1/alarms?room_id=&kind=&open=&limit=&offset=    ViewAllRooms
//! GET   /api/v1/alarms/{id}                                   ViewAllRooms
//! POST  /api/v1/alarms/{id}/acknowledge                       HandleAlarms
//! POST  /api/v1/alarms/{id}/resolve      manual resolve       HandleAlarms

use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    error::{ApiError, ErrorResponse},
    models::alarm::AlarmKind,
    repos::alarms::AlarmWithRoom,
    routes::extractors::CurrentUser,
    services::alarms::{self, ListParams, to_f64},
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(get_one))
        .routes(routes!(acknowledge))
        .routes(routes!(resolve))
}

#[derive(Serialize, ToSchema)]
pub struct AlarmResponse {
    pub id: Uuid,
    pub room_id: Uuid,
    pub room_number: String,
    pub measurement_id: i64,
    pub kind: AlarmKind,
    pub measured_value: f64,
    pub threshold_value: f64,
    pub raised_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    /// `null` while the alarm is open.
    pub resolved_at: Option<DateTime<Utc>>,
}

impl From<AlarmWithRoom> for AlarmResponse {
    fn from((a, room_number): AlarmWithRoom) -> Self {
        Self {
            id: a.id,
            room_id: a.room_id,
            room_number,
            measurement_id: a.measurement_id,
            kind: a.kind(),
            measured_value: to_f64(&a.measured_value),
            threshold_value: to_f64(&a.threshold_value),
            raised_at: a.raised_at,
            acknowledged_at: a.acknowledged_at,
            acknowledged_by: a.acknowledged_by,
            resolved_at: a.resolved_at,
        }
    }
}

#[derive(Deserialize, IntoParams)]
pub struct ListQuery {
    pub room_id: Option<Uuid>,
    pub kind: Option<AlarmKind>,
    /// true = unresolved only, false = resolved only
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

/// List alarms, newest first.
#[utoipa::path(
    get, path = "/", tag = "alarms", operation_id = "list_alarms",
    params(ListQuery),
    responses((status = 200, body = Vec<AlarmResponse>), (status = 403, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn list(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<AlarmResponse>>, ApiError> {
    let mut conn = state.pool.get().await?;
    let params = ListParams {
        room_id: q.room_id,
        kind: q.kind,
        open: q.open,
        limit: q.limit,
        offset: q.offset,
    };
    let alarms = alarms::list(&mut conn, &actor, params).await?;
    Ok(Json(alarms.into_iter().map(Into::into).collect()))
}

/// Get one alarm.
#[utoipa::path(
    get, path = "/{id}", tag = "alarms", operation_id = "get_alarm",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = AlarmResponse), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn get_one(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AlarmResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(alarms::get(&mut conn, &actor, id).await?.into()))
}

/// Mark the alarm as seen. It stays open until readings return to range.
#[utoipa::path(
    post, path = "/{id}/acknowledge", tag = "alarms", operation_id = "acknowledge_alarm",
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, body = AlarmResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, description = "Already acknowledged or resolved", body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn acknowledge(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AlarmResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(
        alarms::acknowledge(&mut conn, &actor, id).await?.into(),
    ))
}

/// Resolve by hand, e.g. after fixing a faulty sensor. Normally alarms resolve themselves.
#[utoipa::path(
    post, path = "/{id}/resolve", tag = "alarms", operation_id = "resolve_alarm",
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, body = AlarmResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, description = "Already resolved", body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn resolve(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<AlarmResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(alarms::resolve(&mut conn, &actor, id).await?.into()))
}
