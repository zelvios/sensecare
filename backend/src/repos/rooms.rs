use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{DbConn, schema::rooms},
    models::room::{NewRoom, Room, RoomUpdate},
};

pub struct RoomFilter<'a> {
    /// Matches room number or name, case-insensitive.
    pub q: Option<&'a str>,
    pub active: Option<bool>,
    pub floor: Option<i16>,
    pub limit: i64,
    pub offset: i64,
}

/// Looks up a room by id, returns `None` if no such room exists.
pub async fn find_by_id(conn: &mut DbConn, id: Uuid) -> QueryResult<Option<Room>> {
    rooms::table
        .find(id)
        .select(Room::as_select())
        .first(conn)
        .await
        .optional()
}

/// Lists rooms matching the filter, ordered by room number, paged.
pub async fn list(conn: &mut DbConn, f: &RoomFilter<'_>) -> QueryResult<Vec<Room>> {
    let mut query = rooms::table
        .select(Room::as_select())
        .order(rooms::room_number.asc())
        .limit(f.limit)
        .offset(f.offset)
        .into_boxed();

    if let Some(q) = f.q.map(str::trim).filter(|s| !s.is_empty()) {
        let pattern = format!("%{}%", super::escape_like(q));
        query = query.filter(
            rooms::room_number
                .ilike(pattern.clone())
                .or(rooms::name.ilike(pattern)),
        );
    }
    if let Some(active) = f.active {
        query = query.filter(rooms::is_active.eq(active));
    }
    if let Some(floor) = f.floor {
        query = query.filter(rooms::floor.eq(floor));
    }
    query.load(conn).await
}

/// Creates a room and returns the stored row.
pub async fn insert(conn: &mut DbConn, new: &NewRoom<'_>) -> QueryResult<Room> {
    diesel::insert_into(rooms::table)
        .values(new)
        .returning(Room::as_returning())
        .get_result(conn)
        .await
}

/// Applies a partial update, `None` fields in `changes` are left untouched.
pub async fn update(conn: &mut DbConn, id: Uuid, changes: &RoomUpdate<'_>) -> QueryResult<Room> {
    diesel::update(rooms::table.find(id))
        .set(changes)
        .returning(Room::as_returning())
        .get_result(conn)
        .await
}

/// Soft delete / restore: sets `is_active`.
pub async fn set_active(conn: &mut DbConn, id: Uuid, active: bool) -> QueryResult<()> {
    diesel::update(rooms::table.find(id))
        .set(rooms::is_active.eq(active))
        .execute(conn)
        .await
        .map(|_| ())
}

/// Hard delete. Fails with a foreign-key violation if anything still references the room.
pub async fn delete(conn: &mut DbConn, id: Uuid) -> QueryResult<()> {
    diesel::delete(rooms::table.find(id))
        .execute(conn)
        .await
        .map(|_| ())
}
