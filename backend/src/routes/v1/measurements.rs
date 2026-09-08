//! Measurement request and response types. Handlers live in `devices.rs`
//! (POST /devices/measurements) and `rooms.rs` (GET /rooms/{id}/measurements).

use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::models::measurement::Measurement;

#[derive(Deserialize, ToSchema)]
pub struct MeasurementRequest {
    /// -40 to 85, rounded to one decimal.
    #[schema(example = 21.4)]
    pub temperature_c: f64,
    /// 0 to 100, rounded to one decimal.
    #[schema(example = 43.0)]
    pub humidity_pct: f64,
    /// Device clock in RFC 3339. Omit if the device has no reliable clock.
    pub measured_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct MeasurementResponse {
    pub id: i64,
    pub room_id: Uuid,
    pub device_id: Uuid,
    #[schema(example = 21.4)]
    pub temperature_c: f64,
    #[schema(example = 43.0)]
    pub humidity_pct: f64,
    pub measured_at: DateTime<Utc>,
    pub received_at: DateTime<Utc>,
}

impl From<Measurement> for MeasurementResponse {
    fn from(m: Measurement) -> Self {
        Self {
            id: m.id,
            room_id: m.room_id,
            device_id: m.device_id,
            temperature_c: m.temperature_c.to_f64().unwrap_or(f64::NAN),
            humidity_pct: m.humidity_pct.to_f64().unwrap_or(f64::NAN),
            measured_at: m.measured_at,
            received_at: m.received_at,
        }
    }
}

#[derive(Deserialize, IntoParams)]
pub struct HistoryQuery {
    /// RFC 3339. Default: 24 hours before `to`.
    pub from: Option<DateTime<Utc>>,
    /// RFC 3339. Default: now.
    pub to: Option<DateTime<Utc>>,
    /// 1-2000, default 500
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
