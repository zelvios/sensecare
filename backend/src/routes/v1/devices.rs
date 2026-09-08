//! Rules and audit live in services::devices.
//!
//! GET    /api/v1/devices                    list / search            ViewDevices
//! POST   /api/v1/devices                    register, key shown once ManageDevices
//! GET    /api/v1/devices/self               the calling device       X-Device-Id + X-Device-Key
//! GET    /api/v1/devices/{id}                                        ViewDevices
//! PATCH  /api/v1/devices/{id}               label, firmware_version  ManageDevices
//! POST   /api/v1/devices/{id}/assign        {room_id} or null        ManageDevices
//! POST   /api/v1/devices/{id}/rotate-key    new key shown once       ManageDevices
//! POST   /api/v1/devices/{id}/deactivate                             ManageDevices
//! POST   /api/v1/devices/{id}/activate                               ManageDevices
//! DELETE /api/v1/devices/{id}               hard delete              DeleteDevices (admin)

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
    models::device::Device,
    routes::extractors::{CurrentUser, DeviceAuth},
    services::devices::{self, DeviceWithKey, ListParams},
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list, register))
        .routes(routes!(self_info))
        .routes(routes!(get_one, update, delete))
        .routes(routes!(assign))
        .routes(routes!(rotate_key))
        .routes(routes!(deactivate))
        .routes(routes!(activate))
}

// --- DTOs ----

/// A device as seen by staff and admins. Never includes the key or its hash.
#[derive(Serialize, ToSchema)]
pub struct DeviceResponse {
    pub id: Uuid,
    pub room_id: Option<Uuid>,
    pub label: Option<String>,
    pub firmware_version: Option<String>,
    pub is_active: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Device> for DeviceResponse {
    fn from(d: Device) -> Self {
        Self {
            id: d.id,
            room_id: d.room_id,
            label: d.label,
            firmware_version: d.firmware_version,
            is_active: d.is_active,
            last_seen_at: d.last_seen_at,
            created_at: d.created_at,
            updated_at: d.updated_at,
        }
    }
}

/// Returned by register and rotate key. The key is shown here and never again.
#[derive(Serialize, ToSchema)]
pub struct DeviceWithKeyResponse {
    #[serde(flatten)]
    pub device: DeviceResponse,
    /// Put this in the firmware's config.h as DEVICE_KEY. It cannot be retrieved later.
    pub key: String,
}

impl From<DeviceWithKey> for DeviceWithKeyResponse {
    fn from(d: DeviceWithKey) -> Self {
        Self {
            device: d.device.into(),
            key: d.key,
        }
    }
}

#[derive(Deserialize, IntoParams)]
pub struct ListQuery {
    /// Matches label, case-insensitive.
    pub q: Option<String>,
    pub room_id: Option<Uuid>,
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
pub struct RegisterDeviceRequest {
    #[schema(example = "Stue 12 node")]
    pub label: Option<String>,
    #[schema(example = "0.1.0")]
    pub firmware_version: Option<String>,
    /// Optional. Assign to a room right away.
    pub room_id: Option<Uuid>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateDeviceRequest {
    pub label: Option<String>,
    pub firmware_version: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct AssignDeviceRequest {
    /// `null` unassigns the device.
    pub room_id: Option<Uuid>,
}

// --- handlers ----

/// List or search devices.
#[utoipa::path(
    get, path = "/", tag = "devices", operation_id = "list_devices",
    params(ListQuery),
    responses((status = 200, body = Vec<DeviceResponse>), (status = 403, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn list(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<DeviceResponse>>, ApiError> {
    let mut conn = state.pool.get().await?;
    let params = ListParams {
        q: q.q.as_deref(),
        room_id: q.room_id,
        active: q.active,
        limit: q.limit,
        offset: q.offset,
    };
    let devices = devices::list(&mut conn, &actor, params).await?;
    Ok(Json(devices.into_iter().map(Into::into).collect()))
}

/// Register a device. The response contains the key exactly once.
#[utoipa::path(
    post, path = "/", tag = "devices", operation_id = "register_device",
    request_body = RegisterDeviceRequest,
    responses(
        (status = 201, body = DeviceWithKeyResponse),
        (status = 400, description = "Validation failed or room unusable", body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 409, description = "Room already has an active device", body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn register(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Json(body): Json<RegisterDeviceRequest>,
) -> Result<(StatusCode, Json<DeviceWithKeyResponse>), ApiError> {
    let mut conn = state.pool.get().await?;
    let created = devices::register(
        &mut conn,
        &actor,
        body.label.as_deref(),
        body.firmware_version.as_deref(),
        body.room_id,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(created.into())))
}

/// The calling device, authenticated with X-Device-Id and X-Device-Key.
/// Lets firmware verify its credentials during setup.
#[utoipa::path(
    get, path = "/self", tag = "devices", operation_id = "device_self",
    responses((status = 200, body = DeviceResponse), (status = 401, body = ErrorResponse)),
    security(("device_id" = [], "device_key" = []))
)]
async fn self_info(DeviceAuth(device): DeviceAuth) -> Json<DeviceResponse> {
    Json(device.into())
}

/// One device.
#[utoipa::path(
    get, path = "/{id}", tag = "devices", operation_id = "get_device",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = DeviceResponse), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn get_one(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(devices::get(&mut conn, &actor, id).await?.into()))
}

/// Change label and/or firmware version.
#[utoipa::path(
    patch, path = "/{id}", tag = "devices", operation_id = "update_device",
    params(("id" = Uuid, Path)),
    request_body = UpdateDeviceRequest,
    responses((status = 200, body = DeviceResponse), (status = 400, body = ErrorResponse), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn update(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateDeviceRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    let device = devices::update(
        &mut conn,
        &actor,
        id,
        body.label.as_deref(),
        body.firmware_version.as_deref(),
    )
    .await?;
    Ok(Json(device.into()))
}

/// Assign the device to a room, or unassign it with `room_id: null`.
#[utoipa::path(
    post, path = "/{id}/assign", tag = "devices", operation_id = "assign_device",
    params(("id" = Uuid, Path)),
    request_body = AssignDeviceRequest,
    responses(
        (status = 200, body = DeviceResponse),
        (status = 400, description = "Room does not exist or is deactivated", body = ErrorResponse),
        (status = 403, body = ErrorResponse),
        (status = 404, body = ErrorResponse),
        (status = 409, description = "Room already has an active device", body = ErrorResponse),
    ),
    security(("bearer" = []))
)]
async fn assign(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<AssignDeviceRequest>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(
        devices::assign(&mut conn, &actor, id, body.room_id)
            .await?
            .into(),
    ))
}

/// Generate a new key. The old key stops working immediately. The new key is shown once.
#[utoipa::path(
    post, path = "/{id}/rotate-key", tag = "devices", operation_id = "rotate_device_key",
    params(("id" = Uuid, Path)),
    responses((status = 200, body = DeviceWithKeyResponse), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn rotate_key(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<DeviceWithKeyResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    Ok(Json(
        devices::rotate_key(&mut conn, &actor, id).await?.into(),
    ))
}

/// Deactivate a device. It's key stops working until reactivated.
#[utoipa::path(
    post, path = "/{id}/deactivate", tag = "devices", operation_id = "deactivate_device",
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
    devices::set_active(&mut conn, &actor, id, false).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Reactivate a device.
#[utoipa::path(
    post, path = "/{id}/activate", tag = "devices", operation_id = "activate_device",
    params(("id" = Uuid, Path)),
    responses((status = 204), (status = 403, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn activate(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let mut conn = state.pool.get().await?;
    devices::set_active(&mut conn, &actor, id, true).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Permanently delete a device (admin only). Refused if it has measurements or calls.
#[utoipa::path(
    delete, path = "/{id}", tag = "devices", operation_id = "delete_device",
    params(("id" = Uuid, Path)),
    responses(
        (status = 204),
        (status = 400, description = "Still referenced by measurements or calls", body = ErrorResponse),
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
    devices::delete(&mut conn, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
