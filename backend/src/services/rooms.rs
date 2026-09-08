//! Rules:
//!   - reading needs `ViewAllRooms` (staff and admin)
//!   - creating, editing, deactivating needs `ManageRooms` (admin)
//!   - rooms are never hard-deleted and measurements, stays and calls reference them
//!   - every mutation writes one audit entry.
//!   - hard delete needs `DeleteRooms` (admin) and is refused by the database if the room is
//!     referenced by devices, stays, measurements or calls

use serde_json::json;
use uuid::Uuid;

use crate::{
    db::DbConn,
    error::ApiError,
    models::{
        audit::AuditAction,
        role::Permission,
        room::{NewRoom, Room, RoomUpdate},
    },
    repos::{self, rooms::RoomFilter},
    services::{audit, auth::AuthenticatedUser},
};

pub struct ListParams<'a> {
    pub q: Option<&'a str>,
    pub active: Option<bool>,
    pub floor: Option<i16>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    p: ListParams<'_>,
) -> Result<Vec<Room>, ApiError> {
    actor.require(Permission::ViewAllRooms)?;
    Ok(repos::rooms::list(
        conn,
        &RoomFilter {
            q: p.q,
            active: p.active,
            floor: p.floor,
            limit: p.limit.clamp(1, 200),
            offset: p.offset.max(0),
        },
    )
    .await?)
}

pub async fn get(conn: &mut DbConn, actor: &AuthenticatedUser, id: Uuid) -> Result<Room, ApiError> {
    actor.require(Permission::ViewAllRooms)?;
    repos::rooms::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)
}

pub async fn create(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    room_number: &str,
    name: Option<&str>,
    floor: Option<i16>,
) -> Result<Room, ApiError> {
    actor.require(Permission::ManageRooms)?;
    let room_number = room_number.trim();
    let name = name.map(str::trim).filter(|n| !n.is_empty());
    validate_room_number(room_number)?;
    validate_name(name)?;
    validate_floor(floor)?;

    let room = repos::rooms::insert(
        conn,
        &NewRoom {
            room_number,
            name,
            floor,
        },
    )
    .await?;

    audit::record(
        conn,
        Some(actor),
        AuditAction::RoomCreated,
        &room.id,
        Some(json!({ "room_number": room.room_number, "name": room.name, "floor": room.floor })),
    )
    .await?;
    Ok(room)
}

pub async fn update(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
    room_number: Option<&str>,
    name: Option<&str>,
    floor: Option<i16>,
) -> Result<Room, ApiError> {
    actor.require(Permission::ManageRooms)?;
    let current = repos::rooms::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let room_number = room_number.map(str::trim);
    let name = name.map(str::trim);
    if let Some(n) = room_number {
        validate_room_number(n)?;
    }
    validate_name(name)?;
    validate_floor(floor)?;

    let mut changes = serde_json::Map::new();
    if let Some(n) = room_number.filter(|n| *n != current.room_number) {
        changes.insert(
            "room_number".into(),
            json!({ "from": current.room_number, "to": n }),
        );
    }
    if let Some(n) = name.filter(|n| Some(*n) != current.name.as_deref()) {
        changes.insert("name".into(), json!({ "from": current.name, "to": n }));
    }
    if let Some(f) = floor.filter(|f| Some(*f) != current.floor) {
        changes.insert("floor".into(), json!({ "from": current.floor, "to": f }));
    }
    if changes.is_empty() {
        return Ok(current);
    }

    let room = repos::rooms::update(
        conn,
        id,
        &RoomUpdate {
            room_number,
            name,
            floor,
        },
    )
    .await?;
    audit::record(
        conn,
        Some(actor),
        AuditAction::RoomUpdated,
        &id,
        Some(changes.into()),
    )
    .await?;
    Ok(room)
}

pub async fn set_active(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
    active: bool,
) -> Result<(), ApiError> {
    actor.require(Permission::ManageRooms)?;
    let room = repos::rooms::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if room.is_active == active {
        return Ok(());
    }

    repos::rooms::set_active(conn, id, active).await?;
    let action = if active {
        AuditAction::RoomActivated
    } else {
        AuditAction::RoomDeactivated
    };
    audit::record(conn, Some(actor), action, &id, None).await?;
    Ok(())
}

pub async fn delete(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<(), ApiError> {
    actor.require(Permission::DeleteRooms)?;
    let room = repos::rooms::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    // Anything referencing the room makes this fail with a foreign-key violation,
    // which error.rs turns into 400 invalid_reference.
    repos::rooms::delete(conn, id).await?;

    // The row is gone, so the audit entry carries a snapshot of what it was.
    audit::record(
        conn,
        Some(actor),
        AuditAction::RoomDeleted,
        &id,
        Some(json!({ "room_number": room.room_number, "name": room.name, "floor": room.floor })),
    )
    .await?;
    Ok(())
}

fn validate_room_number(n: &str) -> Result<(), ApiError> {
    let ok = (1..=16).contains(&n.chars().count())
        && n.chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '.' | '-'));
    if ok {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "room_number must be 1-16 characters: letters, digits, '.', '-'".into(),
        ))
    }
}

fn validate_name(name: Option<&str>) -> Result<(), ApiError> {
    match name {
        Some(n) if n.chars().count() > 64 => Err(ApiError::BadRequest(
            "name must be at most 64 characters".into(),
        )),
        _ => Ok(()),
    }
}

fn validate_floor(floor: Option<i16>) -> Result<(), ApiError> {
    match floor {
        Some(f) if !(-5..=100).contains(&f) => Err(ApiError::BadRequest(
            "floor must be between -5 and 100".into(),
        )),
        _ => Ok(()),
    }
}
