//! Rules:
//!   - reading needs `ViewDevices` (staff and admin)
//!   - registering, editing, assigning, rotating keys and deactivating needs `ManageDevices` (admin)
//!   - hard delete needs `DeleteDevices` (admin) and is refused by the database if
//!     the device has measurements or service calls
//!   - a room can have at most one active device (database unique index)
//!   - the device key is random, stored as a SHA-256 hash, and shown exactly once
//!   - every mutation writes one audit entry.

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde_json::json;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::{
    db::DbConn,
    error::ApiError,
    models::{
        audit::AuditAction,
        device::{Device, DeviceUpdate, NewDevice},
        role::Permission,
    },
    repos::{self, devices::DeviceFilter},
    services::{audit, auth::AuthenticatedUser},
};

/// A freshly registered or re-keyed device together with its plaintext key.
pub struct DeviceWithKey {
    pub device: Device,
    /// Shown once. Only the hash is stored.
    pub key: String,
}

pub struct ListParams<'a> {
    pub q: Option<&'a str>,
    pub room_id: Option<Uuid>,
    pub active: Option<bool>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    p: ListParams<'_>,
) -> Result<Vec<Device>, ApiError> {
    actor.require(Permission::ViewDevices)?;
    Ok(repos::devices::list(
        conn,
        &DeviceFilter {
            q: p.q,
            room_id: p.room_id,
            active: p.active,
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
) -> Result<Device, ApiError> {
    actor.require(Permission::ViewDevices)?;
    repos::devices::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)
}

pub async fn register(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    label: Option<&str>,
    firmware_version: Option<&str>,
    room_id: Option<Uuid>,
) -> Result<DeviceWithKey, ApiError> {
    actor.require(Permission::ManageDevices)?;
    let label = label.map(str::trim).filter(|s| !s.is_empty());
    validate_label(label)?;
    if let Some(room_id) = room_id {
        ensure_room_assignable(conn, room_id).await?;
    }

    let key = generate_key();
    let hash = hash_key(&key);
    let device = repos::devices::insert(
        conn,
        &NewDevice {
            room_id,
            key_hash: &hash,
            label,
            firmware_version,
        },
    )
    .await
    .map_err(map_unique_room)?;

    audit::record(
        conn,
        Some(actor),
        AuditAction::DeviceRegistered,
        &device.id,
        Some(json!({ "label": device.label, "room_id": device.room_id })),
    )
    .await?;
    Ok(DeviceWithKey { device, key })
}

pub async fn update(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
    label: Option<&str>,
    firmware_version: Option<&str>,
) -> Result<Device, ApiError> {
    actor.require(Permission::ManageDevices)?;
    let current = repos::devices::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let label = label.map(str::trim);
    validate_label(label)?;

    let mut changes = serde_json::Map::new();
    if let Some(l) = label.filter(|l| Some(*l) != current.label.as_deref()) {
        changes.insert("label".into(), json!({ "from": current.label, "to": l }));
    }
    if let Some(f) = firmware_version.filter(|f| Some(*f) != current.firmware_version.as_deref()) {
        changes.insert(
            "firmware_version".into(),
            json!({ "from": current.firmware_version, "to": f }),
        );
    }
    if changes.is_empty() {
        return Ok(current);
    }

    let device = repos::devices::update(
        conn,
        id,
        &DeviceUpdate {
            label,
            firmware_version,
        },
    )
    .await?;
    audit::record(
        conn,
        Some(actor),
        AuditAction::DeviceUpdated,
        &id,
        Some(changes.into()),
    )
    .await?;
    Ok(device)
}

/// Moves a device to a room, or unassigns it. A room can hold one active device.
pub async fn assign(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
    room_id: Option<Uuid>,
) -> Result<Device, ApiError> {
    actor.require(Permission::ManageDevices)?;
    let current = repos::devices::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if current.room_id == room_id {
        return Ok(current);
    }
    if let Some(room_id) = room_id {
        ensure_room_assignable(conn, room_id).await?;
    }

    let device = repos::devices::set_room(conn, id, room_id)
        .await
        .map_err(map_unique_room)?;
    audit::record(
        conn,
        Some(actor),
        AuditAction::DeviceAssigned,
        &id,
        Some(json!({ "room_id": { "from": current.room_id, "to": room_id } })),
    )
    .await?;
    Ok(device)
}

/// Generates a new key. The old one stops working immediately.
pub async fn rotate_key(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<DeviceWithKey, ApiError> {
    actor.require(Permission::ManageDevices)?;
    repos::devices::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    let key = generate_key();
    repos::devices::set_key_hash(conn, id, &hash_key(&key)).await?;
    audit::record(conn, Some(actor), AuditAction::DeviceKeyRotated, &id, None).await?;

    let device = repos::devices::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(DeviceWithKey { device, key })
}

pub async fn set_active(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
    active: bool,
) -> Result<(), ApiError> {
    actor.require(Permission::ManageDevices)?;
    let device = repos::devices::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if device.is_active == active {
        return Ok(());
    }
    // Reactivating into a room that got another active device meanwhile hits the unique index.
    repos::devices::set_active(conn, id, active)
        .await
        .map_err(map_unique_room)?;
    let action = if active {
        AuditAction::DeviceActivated
    } else {
        AuditAction::DeviceDeactivated
    };
    audit::record(conn, Some(actor), action, &id, None).await?;
    Ok(())
}

pub async fn delete(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<(), ApiError> {
    actor.require(Permission::DeleteDevices)?;
    let device = repos::devices::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;

    repos::devices::delete(conn, id).await?;
    audit::record(
        conn,
        Some(actor),
        AuditAction::DeviceDeleted,
        &id,
        Some(json!({ "label": device.label, "room_id": device.room_id })),
    )
    .await?;
    Ok(())
}

/// Resolves device credentials. Used by the `DeviceAuth` extractor on every device request.
pub async fn authenticate_device(
    conn: &mut DbConn,
    id: Uuid,
    key: &str,
) -> Result<Device, ApiError> {
    let Some(device) = repos::devices::find_by_id(conn, id).await? else {
        return Err(ApiError::Unauthorized);
    };
    let presented = hash_key(key);
    let ok: bool = presented
        .as_bytes()
        .ct_eq(device.key_hash.as_bytes())
        .into();
    if !ok || !device.is_active {
        return Err(ApiError::Unauthorized);
    }
    let device = repos::devices::touch_last_seen(conn, device.id).await?;
    Ok(device)
}

// --- helpers ----

/// 32 random bytes as 64 hex characters.
fn generate_key() -> String {
    hex::encode(rand::random::<[u8; 32]>())
}

fn hash_key(key: &str) -> String {
    hex::encode(Sha256::digest(key.as_bytes()))
}

async fn ensure_room_assignable(conn: &mut DbConn, room_id: Uuid) -> Result<(), ApiError> {
    let room = repos::rooms::find_by_id(conn, room_id)
        .await?
        .ok_or_else(|| ApiError::BadRequest("room does not exist".into()))?;
    if !room.is_active {
        return Err(ApiError::BadRequest("room is deactivated".into()));
    }
    Ok(())
}

/// The one-active-device-per-room index is the only unique constraint on devices,
/// so a unique violation always means exactly that.
fn map_unique_room(e: DieselError) -> ApiError {
    match e {
        DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
            ApiError::Conflict("room already has an active device".into())
        }
        other => other.into(),
    }
}

fn validate_label(label: Option<&str>) -> Result<(), ApiError> {
    match label {
        Some(l) if l.chars().count() > 64 => Err(ApiError::BadRequest(
            "label must be at most 64 characters".into(),
        )),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_random_and_hex() {
        let a = generate_key();
        let b = generate_key();
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_is_stable_and_not_the_key() {
        let k = generate_key();
        assert_eq!(hash_key(&k), hash_key(&k));
        assert_ne!(hash_key(&k), k);
    }
}
