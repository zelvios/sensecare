//! Rules:
//!   - a device must be assigned to a room before it can report
//!   - values are rounded to one decimal and range-checked before insert
//!   - `measured_at` is optional (server time if missing) and may not be in the future
//!   - staff and admins read any room; a client reads only the room they are checked into
//!   - no audit entries: the measurements table is the record.
//!   - every stored reading is evaluated against the room's thresholds (see services::alarms)

use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::{
    db::DbConn,
    error::ApiError,
    models::{
        device::Device,
        measurement::{Measurement, NewMeasurement},
        role::Permission,
    },
    repos::{self, measurements::MeasurementRange},
    services::{alarms, auth::AuthenticatedUser},
};

/// Readings older than this cannot be requested in one page.
const MAX_LIMIT: i64 = 2000;
const DEFAULT_LIMIT: i64 = 500;
const FUTURE_TOLERANCE_MINUTES: i64 = 5;

pub struct Reading {
    pub temperature_c: f64,
    pub humidity_pct: f64,
    pub measured_at: Option<DateTime<Utc>>,
}

/// Stores a reading from an authenticated device.
pub async fn ingest(
    conn: &mut DbConn,
    device: &Device,
    reading: Reading,
) -> Result<Measurement, ApiError> {
    let room_id = device
        .room_id
        .ok_or_else(|| ApiError::Conflict("device is not assigned to a room".into()))?;

    let temperature_c = to_decimal(reading.temperature_c, -40.0, 85.0, "temperature_c")?;
    let humidity_pct = to_decimal(reading.humidity_pct, 0.0, 100.0, "humidity_pct")?;

    let now = Utc::now();
    let measured_at = reading.measured_at.unwrap_or(now);
    if measured_at > now + Duration::minutes(FUTURE_TOLERANCE_MINUTES) {
        return Err(ApiError::BadRequest("measured_at is in the future".into()));
    }

    let measurement = repos::measurements::insert(
        conn,
        &NewMeasurement {
            room_id,
            device_id: device.id,
            temperature_c,
            humidity_pct,
            measured_at,
            received_at: now,
        },
    )
    .await?;

    alarms::evaluate(conn, &measurement).await?;

    Ok(measurement)
}

pub struct HistoryParams {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// History for a room. Defaults to the last 24 hours.
pub async fn history(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    room_id: Uuid,
    p: HistoryParams,
) -> Result<Vec<Measurement>, ApiError> {
    ensure_can_view_room(conn, actor, room_id).await?;

    let to = p.to.unwrap_or_else(Utc::now);
    let from = p.from.unwrap_or(to - Duration::hours(24));
    if from > to {
        return Err(ApiError::BadRequest("from must be before to".into()));
    }

    Ok(repos::measurements::for_room(
        conn,
        room_id,
        &MeasurementRange {
            from,
            to,
            limit: p.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT),
            offset: p.offset.unwrap_or(0).max(0),
        },
    )
    .await?)
}

/// Most recent reading for a room. 404 when the room has none yet.
pub async fn latest(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    room_id: Uuid,
) -> Result<Measurement, ApiError> {
    ensure_can_view_room(conn, actor, room_id).await?;
    repos::measurements::latest_for_room(conn, room_id)
        .await?
        .ok_or(ApiError::NotFound)
}

/// Staff and admins see every room, a client only the room of their open stay.
pub async fn ensure_can_view_room(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    room_id: Uuid,
) -> Result<(), ApiError> {
    if actor.role.has(Permission::ViewAllRooms) {
        if repos::rooms::find_by_id(conn, room_id).await?.is_none() {
            return Err(ApiError::NotFound);
        }
        return Ok(());
    }
    actor.require(Permission::ViewOwnRoom)?;
    if repos::stays::user_occupies_room(conn, actor.id, room_id).await? {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

/// Rounds to one decimal and checks the range the database also enforces.
fn to_decimal(value: f64, min: f64, max: f64, field: &str) -> Result<BigDecimal, ApiError> {
    if !value.is_finite() || value < min || value > max {
        return Err(ApiError::BadRequest(format!(
            "{field} must be between {min} and {max}"
        )));
    }
    BigDecimal::from_f64(value)
        .map(|d| d.round(1))
        .ok_or_else(|| ApiError::BadRequest(format!("{field} is not a valid number")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_rounded_to_one_decimal() {
        assert_eq!(
            to_decimal(21.37, -40.0, 85.0, "t").unwrap().to_string(),
            "21.4"
        );
        assert_eq!(
            to_decimal(21.0, -40.0, 85.0, "t").unwrap().to_string(),
            "21.0"
        );
    }

    #[test]
    fn out_of_range_and_nan_are_rejected() {
        assert!(to_decimal(-40.1, -40.0, 85.0, "t").is_err());
        assert!(to_decimal(100.1, 0.0, 100.0, "h").is_err());
        assert!(to_decimal(f64::NAN, 0.0, 100.0, "h").is_err());
    }
}
