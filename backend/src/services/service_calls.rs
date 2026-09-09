//! Lifecycle: open -> in_progress (acknowledged) -> closed. A call may be closed
//! straight from open.
//!
//! Rules:
//!   - a device must be assigned to a room to raise a call
//!   - a room has at most one unclosed call: a repeated press returns the existing one
//!   - reading needs `ViewAllRooms`, handling needs `HandleServiceCalls` (staff and admin)
//!   - audit: created (actor none), acknowledged, closed, note updated.

use chrono::{DateTime, Duration, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::DbConn,
    error::ApiError,
    models::{
        audit::AuditAction,
        device::Device,
        role::Permission,
        service_call::{NewServiceCall, ServiceCall, ServiceCallStatus},
    },
    repos::{
        self,
        service_calls::{CallFilter, CallWithRoom},
    },
    services::{audit, auth::AuthenticatedUser},
};

/// Outcome of a button press.
pub struct Raised {
    pub call: ServiceCall,
    /// False when the room already had an unclosed call and that one is returned.
    pub created: bool,
}

/// A button press from an authenticated device.
pub async fn raise(
    conn: &mut DbConn,
    device: &Device,
    pressed_at: Option<DateTime<Utc>>,
) -> Result<Raised, ApiError> {
    let room_id = device
        .room_id
        .ok_or_else(|| ApiError::Conflict("device is not assigned to a room".into()))?;

    if let Some(t) = pressed_at
        && t > Utc::now() + Duration::minutes(5)
    {
        return Err(ApiError::BadRequest("pressed_at is in the future".into()));
    }

    if let Some(existing) = repos::service_calls::open_for_room(conn, room_id).await? {
        return Ok(Raised {
            call: existing,
            created: false,
        });
    }

    let call = repos::service_calls::insert(
        conn,
        &NewServiceCall {
            room_id,
            device_id: device.id,
            pressed_at,
        },
    )
    .await?;
    audit::record(
        conn,
        None,
        AuditAction::ServiceCallCreated,
        &call.id,
        Some(json!({ "room_id": room_id, "device_id": device.id })),
    )
    .await?;
    Ok(Raised {
        call,
        created: true,
    })
}

pub struct ListParams {
    pub room_id: Option<Uuid>,
    pub status: Option<ServiceCallStatus>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    p: ListParams,
) -> Result<Vec<CallWithRoom>, ApiError> {
    actor.require(Permission::ViewAllRooms)?;
    Ok(repos::service_calls::list(
        conn,
        &CallFilter {
            room_id: p.room_id,
            status: p.status,
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
) -> Result<CallWithRoom, ApiError> {
    actor.require(Permission::ViewAllRooms)?;
    repos::service_calls::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)
}

pub async fn acknowledge(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<CallWithRoom, ApiError> {
    actor.require(Permission::HandleServiceCalls)?;
    let (current, room_number) = repos::service_calls::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if current.status() != ServiceCallStatus::Open {
        return Err(ApiError::Conflict(format!("call is {}", current.status)));
    }

    let call = repos::service_calls::acknowledge(conn, id, actor.id)
        .await?
        .ok_or_else(|| ApiError::Conflict("call is no longer open".into()))?;

    audit::record(
        conn,
        Some(actor),
        AuditAction::ServiceCallAcknowledged,
        &id,
        Some(json!({ "status": { "from": "open", "to": "in_progress" } })),
    )
    .await?;
    Ok((call, room_number))
}

pub async fn close(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
    note: Option<&str>,
) -> Result<CallWithRoom, ApiError> {
    actor.require(Permission::HandleServiceCalls)?;
    let (current, room_number) = repos::service_calls::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if current.status() == ServiceCallStatus::Closed {
        return Err(ApiError::Conflict("call is already closed".into()));
    }
    let note = note.map(str::trim).filter(|n| !n.is_empty());
    validate_note(note)?;
    // Keep an earlier note if the close request brings none.
    let final_note = note.or(current.note.as_deref());

    let call = repos::service_calls::close(conn, id, actor.id, final_note)
        .await?
        .ok_or_else(|| ApiError::Conflict("call is already closed".into()))?;

    audit::record(
        conn,
        Some(actor),
        AuditAction::ServiceCallClosed,
        &id,
        Some(json!({ "status": { "from": current.status, "to": "closed" }, "note": final_note })),
    )
    .await?;
    Ok((call, room_number))
}

pub async fn update_note(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
    note: Option<&str>,
) -> Result<CallWithRoom, ApiError> {
    actor.require(Permission::HandleServiceCalls)?;
    let (current, room_number) = repos::service_calls::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let note = note.map(str::trim).filter(|n| !n.is_empty());
    validate_note(note)?;
    if note == current.note.as_deref() {
        return Ok((current, room_number));
    }

    let call = repos::service_calls::set_note(conn, id, note).await?;
    audit::record(
        conn,
        Some(actor),
        AuditAction::ServiceCallUpdated,
        &id,
        Some(json!({ "note": { "from": current.note, "to": note } })),
    )
    .await?;
    Ok((call, room_number))
}

fn validate_note(note: Option<&str>) -> Result<(), ApiError> {
    match note {
        Some(n) if n.chars().count() > 500 => Err(ApiError::BadRequest(
            "note must be at most 500 characters".into(),
        )),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_round_trips() {
        for s in [
            ServiceCallStatus::Open,
            ServiceCallStatus::InProgress,
            ServiceCallStatus::Closed,
        ] {
            assert_eq!(s.as_str().parse::<ServiceCallStatus>().unwrap(), s);
        }
        assert!("done".parse::<ServiceCallStatus>().is_err());
    }

    #[test]
    fn note_rules() {
        assert!(validate_note(None).is_ok());
        assert!(validate_note(Some("Bragt vand")).is_ok());
        assert!(validate_note(Some(&"x".repeat(501))).is_err());
    }
}
