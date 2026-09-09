//! Rules and audit live in services::thresholds.
//! Room endpoints are in rooms.rs: GET/PUT/DELETE /rooms/{id}/thresholds.
//!
//! GET  /api/v1/thresholds     global default          ViewAllRooms
//! PUT  /api/v1/thresholds     replace global default  ManageThresholds

use axum::{Json, extract::State};
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    error::{ApiError, ErrorResponse},
    models::threshold::ClimateThreshold,
    routes::extractors::CurrentUser,
    services::thresholds::{self, Limits, ThresholdSource},
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_global, set_global))
}

#[derive(Serialize, ToSchema)]
pub struct ThresholdResponse {
    pub id: Uuid,
    /// `null` for the global default.
    pub room_id: Option<Uuid>,
    /// Whether these limits come from a room override or the global default.
    pub source: ThresholdSource,
    pub temperature_min: f64,
    pub temperature_max: f64,
    pub humidity_min: f64,
    pub humidity_max: f64,
    pub updated_at: DateTime<Utc>,
}

impl ThresholdResponse {
    pub fn from_threshold(t: ClimateThreshold, source: ThresholdSource) -> Self {
        let f = |v: &bigdecimal::BigDecimal| v.to_f64().unwrap_or(f64::NAN);
        Self {
            id: t.id,
            room_id: t.room_id,
            source,
            temperature_min: f(&t.temperature_min),
            temperature_max: f(&t.temperature_max),
            humidity_min: f(&t.humidity_min),
            humidity_max: f(&t.humidity_max),
            updated_at: t.updated_at,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct ThresholdRequest {
    #[schema(example = 19.0)]
    pub temperature_min: f64,
    #[schema(example = 26.0)]
    pub temperature_max: f64,
    #[schema(example = 30.0)]
    pub humidity_min: f64,
    #[schema(example = 60.0)]
    pub humidity_max: f64,
}

impl From<ThresholdRequest> for Limits {
    fn from(r: ThresholdRequest) -> Self {
        Self {
            temperature_min: r.temperature_min,
            temperature_max: r.temperature_max,
            humidity_min: r.humidity_min,
            humidity_max: r.humidity_max,
        }
    }
}

/// The global default limits.
#[utoipa::path(
    get, path = "/", tag = "thresholds", operation_id = "get_global_thresholds",
    responses((status = 200, body = ThresholdResponse), (status = 403, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn get_global(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
) -> Result<Json<ThresholdResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    let t = thresholds::read_global(&mut conn, &actor).await?;
    Ok(Json(ThresholdResponse::from_threshold(
        t,
        ThresholdSource::Global,
    )))
}

/// Replace the global default limits (admin).
#[utoipa::path(
    put, path = "/", tag = "thresholds", operation_id = "set_global_thresholds",
    request_body = ThresholdRequest,
    responses((status = 200, body = ThresholdResponse), (status = 400, body = ErrorResponse), (status = 403, body = ErrorResponse)),
    security(("bearer" = []))
)]
async fn set_global(
    State(state): State<AppState>,
    CurrentUser(actor): CurrentUser,
    Json(body): Json<ThresholdRequest>,
) -> Result<Json<ThresholdResponse>, ApiError> {
    let mut conn = state.pool.get().await?;
    let t = thresholds::set_global(&mut conn, &actor, body.into()).await?;
    Ok(Json(ThresholdResponse::from_threshold(
        t,
        ThresholdSource::Global,
    )))
}
