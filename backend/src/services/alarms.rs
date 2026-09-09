//! Alarms: raised automatically when a reading breaks the room's limits,
//! resolved automatically when a reading is back inside them.
//!
//! One open alarm per (room, kind) at a time. Raise and auto-resolve are
//! system actions and are not audited: the alarms table is the record.
//! Staff acknowledge (audited) and may resolve by hand (audited), e.g. a faulty sensor.

use bigdecimal::ToPrimitive;
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::DbConn,
    error::ApiError,
    models::{
        alarm::{Alarm, AlarmKind, NewAlarm},
        audit::AuditAction,
        measurement::Measurement,
        role::Permission,
        threshold::ClimateThreshold,
    },
    repos::{
        self,
        alarms::{AlarmFilter, AlarmWithRoom},
    },
    services::{audit, auth::AuthenticatedUser, thresholds},
};

/// Called after every stored measurement. Returns the alarms raised by it.
pub async fn evaluate(conn: &mut DbConn, m: &Measurement) -> Result<Vec<Alarm>, ApiError> {
    let (limits, _) = thresholds::effective(conn, m.room_id).await?;
    let open = repos::alarms::open_for_room(conn, m.room_id).await?;
    let mut raised = Vec::new();

    for kind in AlarmKind::ALL {
        let breach = breach_value(m, &limits, kind);
        let existing = open.iter().find(|a| a.kind() == kind);

        match (breach, existing) {
            (Some(threshold_value), None) => {
                let alarm = repos::alarms::insert(
                    conn,
                    &NewAlarm {
                        room_id: m.room_id,
                        measurement_id: m.id,
                        kind: kind.as_str().to_owned(),
                        measured_value: measured(m, kind),
                        threshold_value,
                    },
                )
                .await?;
                tracing::warn!(
                    target: "alarms",
                    room_id = %m.room_id, kind = kind.as_str(),
                    value = %alarm.measured_value, limit = %alarm.threshold_value,
                    "alarm raised"
                );
                raised.push(alarm);
            }
            (None, Some(a)) => {
                repos::alarms::resolve(conn, a.id).await?;
                tracing::info!(target: "alarms", room_id = %m.room_id, kind = kind.as_str(), "alarm resolved");
            }
            _ => {} // still breached with an open alarm, or fine with none
        }
    }
    Ok(raised)
}

/// The limit that was crossed, if the reading is outside it for this kind.
fn breach_value(
    m: &Measurement,
    t: &ClimateThreshold,
    kind: AlarmKind,
) -> Option<bigdecimal::BigDecimal> {
    match kind {
        AlarmKind::TemperatureLow if m.temperature_c < t.temperature_min => {
            Some(t.temperature_min.clone())
        }
        AlarmKind::TemperatureHigh if m.temperature_c > t.temperature_max => {
            Some(t.temperature_max.clone())
        }
        AlarmKind::HumidityLow if m.humidity_pct < t.humidity_min => Some(t.humidity_min.clone()),
        AlarmKind::HumidityHigh if m.humidity_pct > t.humidity_max => Some(t.humidity_max.clone()),
        _ => None,
    }
}

fn measured(m: &Measurement, kind: AlarmKind) -> bigdecimal::BigDecimal {
    match kind {
        AlarmKind::TemperatureLow | AlarmKind::TemperatureHigh => m.temperature_c.clone(),
        AlarmKind::HumidityLow | AlarmKind::HumidityHigh => m.humidity_pct.clone(),
    }
}

pub struct ListParams {
    pub room_id: Option<Uuid>,
    pub kind: Option<AlarmKind>,
    pub open: Option<bool>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    p: ListParams,
) -> Result<Vec<AlarmWithRoom>, ApiError> {
    actor.require(Permission::ViewAllRooms)?;
    Ok(repos::alarms::list(
        conn,
        &AlarmFilter {
            room_id: p.room_id,
            kind: p.kind,
            open: p.open,
            limit: p.limit.clamp(1, 200),
            offset: p.offset.max(0),
        },
    )
    .await?)
}

pub async fn get(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<AlarmWithRoom, ApiError> {
    actor.require(Permission::ViewAllRooms)?;
    repos::alarms::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)
}

pub async fn acknowledge(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<AlarmWithRoom, ApiError> {
    actor.require(Permission::HandleAlarms)?;
    let (current, room_number) = repos::alarms::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if current.resolved_at.is_some() {
        return Err(ApiError::Conflict("alarm is already resolved".into()));
    }
    if current.acknowledged_at.is_some() {
        return Err(ApiError::Conflict("alarm is already acknowledged".into()));
    }

    let alarm = repos::alarms::acknowledge(conn, id, actor.id)
        .await?
        .ok_or_else(|| ApiError::Conflict("alarm is no longer open".into()))?;
    audit::record(
        conn,
        Some(actor),
        AuditAction::AlarmAcknowledged,
        &id,
        Some(json!({ "kind": alarm.kind })),
    )
    .await?;
    Ok((alarm, room_number))
}

/// Manual resolve. Normally alarms resolve themselves when readings return to range.
pub async fn resolve(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<AlarmWithRoom, ApiError> {
    actor.require(Permission::HandleAlarms)?;
    let (current, room_number) = repos::alarms::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if current.resolved_at.is_some() {
        return Err(ApiError::Conflict("alarm is already resolved".into()));
    }

    let alarm = repos::alarms::resolve(conn, id)
        .await?
        .ok_or_else(|| ApiError::Conflict("alarm is already resolved".into()))?;
    audit::record(
        conn,
        Some(actor),
        AuditAction::AlarmResolved,
        &id,
        Some(json!({ "kind": alarm.kind, "manual": true })),
    )
    .await?;
    Ok((alarm, room_number))
}

/// For responses: BigDecimal to f64.
pub fn to_f64(v: &bigdecimal::BigDecimal) -> f64 {
    v.to_f64().unwrap_or(f64::NAN)
}
