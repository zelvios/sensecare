//! One global default row (seeded - `room_id` is `null`) plus optional per-room overrides.
//! Rules:
//!   - reading needs `ViewAllRooms`, or `ViewOwnRoom` for the client's own room
//!   - changing needs `ManageThresholds` (admin): thresholds are policy, not day-to-day handling
//!   - min < max, humidity within 0-100 and temperature within the sensor's -40..85 (database constraint)
//!   - every change writes one audit entry.

use bigdecimal::{BigDecimal, FromPrimitive};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::DbConn,
    error::ApiError,
    models::{
        audit::AuditAction,
        role::Permission,
        threshold::{ClimateThreshold, NewThreshold, ThresholdValues},
    },
    repos,
    services::{audit, auth::AuthenticatedUser, measurements::ensure_can_view_room},
};

/// Where a room's effective limits come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdSource {
    Room,
    Global,
}

pub struct Limits {
    pub temperature_min: f64,
    pub temperature_max: f64,
    pub humidity_min: f64,
    pub humidity_max: f64,
}

/// Global default. Exists by migration, missing means a broken database.
pub async fn global(conn: &mut DbConn) -> Result<ClimateThreshold, ApiError> {
    repos::thresholds::global(conn)
        .await?
        .ok_or_else(|| anyhow::anyhow!("global climate threshold row is missing").into())
}

/// The limits in force for a room: its override, else the global default.
pub async fn effective(
    conn: &mut DbConn,
    room_id: Uuid,
) -> Result<(ClimateThreshold, ThresholdSource), ApiError> {
    match repos::thresholds::for_room(conn, room_id).await? {
        Some(t) => Ok((t, ThresholdSource::Room)),
        None => Ok((global(conn).await?, ThresholdSource::Global)),
    }
}

pub async fn read_global(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
) -> Result<ClimateThreshold, ApiError> {
    actor.require(Permission::ViewAllRooms)?;
    global(conn).await
}

pub async fn read_for_room(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    room_id: Uuid,
) -> Result<(ClimateThreshold, ThresholdSource), ApiError> {
    ensure_can_view_room(conn, actor, room_id).await?;
    effective(conn, room_id).await
}

pub async fn set_global(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    limits: Limits,
) -> Result<ClimateThreshold, ApiError> {
    actor.require(Permission::ManageThresholds)?;
    let values = validate(limits)?;
    let current = global(conn).await?;

    let updated = repos::thresholds::update(conn, current.id, &values).await?;
    audit::record(
        conn,
        Some(actor),
        AuditAction::ThresholdUpdated,
        &updated.id,
        Some(json!({ "room_id": null, "from": snapshot(&current), "to": snapshot(&updated) })),
    )
    .await?;
    Ok(updated)
}

/// Creates or replaces a room's override.
pub async fn set_for_room(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    room_id: Uuid,
    limits: Limits,
) -> Result<ClimateThreshold, ApiError> {
    actor.require(Permission::ManageThresholds)?;
    let values = validate(limits)?;
    repos::rooms::find_by_id(conn, room_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let (updated, from) = match repos::thresholds::for_room(conn, room_id).await? {
        Some(existing) => (
            repos::thresholds::update(conn, existing.id, &values).await?,
            snapshot(&existing),
        ),
        None => (
            repos::thresholds::insert(
                conn,
                &NewThreshold {
                    room_id: Some(room_id),
                    values,
                },
            )
            .await?,
            serde_json::Value::Null,
        ),
    };

    audit::record(
        conn,
        Some(actor),
        AuditAction::ThresholdUpdated,
        &updated.id,
        Some(json!({ "room_id": room_id, "from": from, "to": snapshot(&updated) })),
    )
    .await?;
    Ok(updated)
}

/// Removes a room's override so the global default applies again.
pub async fn remove_for_room(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    room_id: Uuid,
) -> Result<(), ApiError> {
    actor.require(Permission::ManageThresholds)?;
    let existing = repos::thresholds::for_room(conn, room_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    repos::thresholds::delete(conn, existing.id).await?;
    audit::record(
        conn,
        Some(actor),
        AuditAction::ThresholdRemoved,
        &existing.id,
        Some(json!({ "room_id": room_id, "from": snapshot(&existing) })),
    )
    .await?;
    Ok(())
}

fn validate(l: Limits) -> Result<ThresholdValues, ApiError> {
    let dec = |v: f64, name: &str| -> Result<BigDecimal, ApiError> {
        if !v.is_finite() {
            return Err(ApiError::BadRequest(format!(
                "{name} is not a valid number"
            )));
        }
        BigDecimal::from_f64(v)
            .map(|d| d.round(1))
            .ok_or_else(|| ApiError::BadRequest(format!("{name} is not a valid number")))
    };

    if l.temperature_min >= l.temperature_max {
        return Err(ApiError::BadRequest(
            "temperature_min must be below temperature_max".into(),
        ));
    }
    if l.humidity_min >= l.humidity_max {
        return Err(ApiError::BadRequest(
            "humidity_min must be below humidity_max".into(),
        ));
    }
    if l.temperature_min < -40.0 || l.temperature_max > 85.0 {
        return Err(ApiError::BadRequest(
            "temperature limits must be within -40 and 85".into(),
        ));
    }
    if l.humidity_min < 0.0 || l.humidity_max > 100.0 {
        return Err(ApiError::BadRequest(
            "humidity limits must be within 0 and 100".into(),
        ));
    }

    Ok(ThresholdValues {
        temperature_min: dec(l.temperature_min, "temperature_min")?,
        temperature_max: dec(l.temperature_max, "temperature_max")?,
        humidity_min: dec(l.humidity_min, "humidity_min")?,
        humidity_max: dec(l.humidity_max, "humidity_max")?,
    })
}

pub async fn list_overrides(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
) -> Result<Vec<ClimateThreshold>, ApiError> {
    actor.require(Permission::ViewAllRooms)?;
    Ok(repos::thresholds::list_overrides(conn).await?)
}

fn snapshot(t: &ClimateThreshold) -> serde_json::Value {
    json!({
        "temperature_min": t.temperature_min.to_string(),
        "temperature_max": t.temperature_max.to_string(),
        "humidity_min": t.humidity_min.to_string(),
        "humidity_max": t.humidity_max.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits(tmin: f64, tmax: f64, hmin: f64, hmax: f64) -> Limits {
        Limits {
            temperature_min: tmin,
            temperature_max: tmax,
            humidity_min: hmin,
            humidity_max: hmax,
        }
    }

    #[test]
    fn valid_limits_pass() {
        assert!(validate(limits(19.0, 26.0, 30.0, 60.0)).is_ok());
    }

    #[test]
    fn order_and_range_are_enforced() {
        assert!(
            validate(limits(26.0, 19.0, 30.0, 60.0)).is_err(),
            "temperature order"
        );
        assert!(
            validate(limits(19.0, 26.0, 60.0, 30.0)).is_err(),
            "humidity order"
        );
        assert!(
            validate(limits(-50.0, 26.0, 30.0, 60.0)).is_err(),
            "below sensor range"
        );
        assert!(
            validate(limits(19.0, 26.0, 30.0, 101.0)).is_err(),
            "humidity above 100"
        );
    }
}
