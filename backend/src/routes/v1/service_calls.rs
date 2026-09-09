//! Rules and audit live in services::service_calls. The device side is
//! POST /devices/service-calls in devices.rs.
//!
//! GET    /api/v1/service-calls?room_id=&status=&limit=&offset=     ViewAllRooms
//! GET    /api/v1/service-calls/{id}                                ViewAllRooms
//! POST   /api/v1/service-calls/{id}/acknowledge                    HandleServiceCalls
//! POST   /api/v1/service-calls/{id}/close      {note?}             HandleServiceCalls
//! PATCH  /api/v1/service-calls/{id}            {note}              HandleServiceCalls

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
    models::service_call::{ServiceCall, ServiceCallStatus},
    repos::service_calls::CallWithRoom,
    routes::extractors::CurrentUser,
    services::service_calls::{self, ListParams},
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(get_one, update_note))
        .routes(routes!(acknowledge))
        .routes(routes!(close))
}

// --- DTOs ----

#[derive(Serialize, ToSchema)]
pub struct ServiceCallResponse {
    pub id: Uuid,
    pub room_id: Uuid,
    /// Empty string when the caller is a device (no join in that path).
    pub room_number: String,
    pub device_id: Uuid,
    pub status: ServiceCallStatus,
    pub created_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_by: Option<Uuid>,
    pub note: Option<String>,
}

impl ServiceCallResponse {
    pub fn from_call(c: ServiceCall, room_number: String) -> Self {
        Self {
            id: c.id,
            room_id: c.room_id,
            room_number,
            device_id: c.device_id,
            status: c.status(),
            created_at: c.created_at,
            acknowledged_at: c.acknowledged_at,
            acknowledged_by: c.acknowledged_by,
            closed_at: c.closed_at,
            closed_by: c.closed_by,
            note: c.note,
        }
    }
}

impl From<CallWithRoom> for ServiceCallResponse {
    fn from((c, room_number): CallWithRoom) -> Self {
        Self::from_call(c, room_number)
    }
}

#[derive(Deserialize, IntoParams)]
pub struct ListQuery {
    pub room_id: Option<Uuid>,
    /// open | in_progress | closed
    pub status: Option<ServiceCallStatus>,
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
pub struct NoteRequest {
    /// Up to 500 characters. `null` clears the note.
    #[schema(example = "Manglede piller")]
    pub note: Option<String>,
}

// --- handlers ----

/// List calls, newest first. Filter by room and status.
#[utoipa::path(
    get, path = "/", tag = "service_calls", operation_id = "list_service_calls",
    params(ListQuery),
    responses((status = 200, body = Vec<ServiceCallResponse>), (status = 403, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn list(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<ServiceCallResponse>>, ApiError> {
    let mut conn = state.pool.get().await?;
    let params = ListParams {
        room_id: q.room_id,
        status: q.status,
        limit: q.limit,
        offset: q.offset,
    };
    let calls = service_calls::list(&mut conn, &actor, params).await?;
    Ok(Json(calls.into_iter().map(Into::into).collect()))
}

/// Get one call.
#[utoipa::path(
    get, path = "/{id}", tag = "service_calls", operation_id = "get_service_call",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = ServiceCallResponse), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn get_one(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ServiceCallResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(
        service_calls::get(&mut conn, &actor, id).await?.into(),
    ))
}

/// Mark the call as being handled: open -> in_progress.
#[utoipa::path(
    post, path = "/{id}/acknowledge", tag = "service_calls", operation_id = "acknowledge_service_call",
    params(("id" = Uuid, Path)),
    responses(
        (status = 200, body = ServiceCallResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, description = "Call is not open", body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn acknowledge(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ServiceCallResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(
        service_calls::acknowledge(&mut conn, &actor, id)
            .await?
            .into(),
    ))
}

/// Close the call, optionally with a note. Allowed from open or in_progress.
#[utoipa::path(
    post, path = "/{id}/close", tag = "service_calls", operation_id = "close_service_call",
    params(("id" = Uuid, Path)),
    request_body = NoteRequest,
    responses(
        (status = 200, body = ServiceCallResponse),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, description = "Already closed", body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn close(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<NoteRequest>,
) -> Result<Json<ServiceCallResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(
        service_calls::close(&mut conn, &actor, id, body.note.as_deref())
            .await?
            .into(),
    ))
}

/// Change the note on a call in any state.
#[utoipa::path(
    patch, path = "/{id}", tag = "service_calls", operation_id = "update_service_call_note",
    params(("id" = Uuid, Path)),
    request_body = NoteRequest,
    responses(
        (status = 200, body = ServiceCallResponse),
        (status = 400, body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn update_note(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<NoteRequest>,
) -> Result<Json<ServiceCallResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(
        service_calls::update_note(&mut conn, &actor, id, body.note.as_deref())
            .await?
            .into(),
    ))
}
