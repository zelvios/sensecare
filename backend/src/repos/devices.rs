use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{DbConn, schema::devices},
    models::device::{Device, DeviceUpdate, NewDevice},
    repos::escape_like,
};

pub struct DeviceFilter<'a> {
    /// Matches label, case-insensitive.
    pub q: Option<&'a str>,
    pub room_id: Option<Uuid>,
    pub active: Option<bool>,
    pub limit: i64,
    pub offset: i64,
}

/// Looks up a device by id, returns `None` if no such device exists.
pub async fn find_by_id(conn: &mut DbConn, id: Uuid) -> QueryResult<Option<Device>> {
    devices::table
        .find(id)
        .select(Device::as_select())
        .first(conn)
        .await
        .optional()
}

/// Lists devices matching the filter, newest first, paged.
pub async fn list(conn: &mut DbConn, f: &DeviceFilter<'_>) -> QueryResult<Vec<Device>> {
    let mut query = devices::table
        .select(Device::as_select())
        .order(devices::created_at.desc())
        .limit(f.limit)
        .offset(f.offset)
        .into_boxed();

    if let Some(q) = f.q.map(str::trim).filter(|s| !s.is_empty()) {
        query = query.filter(devices::label.ilike(format!("%{}%", escape_like(q))));
    }
    if let Some(room_id) = f.room_id {
        query = query.filter(devices::room_id.eq(room_id));
    }
    if let Some(active) = f.active {
        query = query.filter(devices::is_active.eq(active));
    }
    query.load(conn).await
}

/// Registers a device and returns the stored row.
pub async fn insert(conn: &mut DbConn, new: &NewDevice<'_>) -> QueryResult<Device> {
    diesel::insert_into(devices::table)
        .values(new)
        .returning(Device::as_returning())
        .get_result(conn)
        .await
}

/// Applies a partial update, `None` fields in `changes` are left untouched.
pub async fn update(
    conn: &mut DbConn,
    id: Uuid,
    changes: &DeviceUpdate<'_>,
) -> QueryResult<Device> {
    diesel::update(devices::table.find(id))
        .set(changes)
        .returning(Device::as_returning())
        .get_result(conn)
        .await
}

/// Moves the device to a room, or unassigns it with `None`.
/// Fails with a unique violation if the room already has an active device.
pub async fn set_room(conn: &mut DbConn, id: Uuid, room_id: Option<Uuid>) -> QueryResult<Device> {
    diesel::update(devices::table.find(id))
        .set(devices::room_id.eq(room_id))
        .returning(Device::as_returning())
        .get_result(conn)
        .await
}

/// Replaces the stored key hash.
pub async fn set_key_hash(conn: &mut DbConn, id: Uuid, hash: &str) -> QueryResult<()> {
    diesel::update(devices::table.find(id))
        .set(devices::key_hash.eq(hash))
        .execute(conn)
        .await
        .map(|_| ())
}

/// Soft delete / restore: sets `is_active`.
pub async fn set_active(conn: &mut DbConn, id: Uuid, active: bool) -> QueryResult<()> {
    diesel::update(devices::table.find(id))
        .set(devices::is_active.eq(active))
        .execute(conn)
        .await
        .map(|_| ())
}

/// Sets `last_seen_at` to now and returns the updated device. Called on every accepted device request.
pub async fn touch_last_seen(conn: &mut DbConn, id: Uuid) -> QueryResult<Device> {
    diesel::update(devices::table.find(id))
        .set(devices::last_seen_at.eq(diesel::dsl::now))
        .returning(Device::as_returning())
        .get_result(conn)
        .await
}

/// Hard delete. Fails with a foreign-key violation if measurements or calls reference the device.
pub async fn delete(conn: &mut DbConn, id: Uuid) -> QueryResult<()> {
    diesel::delete(devices::table.find(id))
        .execute(conn)
        .await
        .map(|_| ())
}
