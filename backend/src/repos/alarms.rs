use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{
        DbConn,
        schema::{alarms, rooms},
    },
    models::alarm::{Alarm, AlarmKind, NewAlarm},
};

pub type AlarmWithRoom = (Alarm, String);

pub struct AlarmFilter {
    pub room_id: Option<Uuid>,
    pub kind: Option<AlarmKind>,
    /// `Some(true)` unresolved only, `Some(false)` resolved only.
    pub open: Option<bool>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn find_by_id(conn: &mut DbConn, id: Uuid) -> QueryResult<Option<AlarmWithRoom>> {
    alarms::table
        .inner_join(rooms::table)
        .filter(alarms::id.eq(id))
        .select((Alarm::as_select(), rooms::room_number))
        .first(conn)
        .await
        .optional()
}

/// All unresolved alarms for a room. At most one per kind by construction.
pub async fn open_for_room(conn: &mut DbConn, room_id: Uuid) -> QueryResult<Vec<Alarm>> {
    alarms::table
        .filter(alarms::room_id.eq(room_id))
        .filter(alarms::resolved_at.is_null())
        .select(Alarm::as_select())
        .load(conn)
        .await
}

pub async fn list(conn: &mut DbConn, f: &AlarmFilter) -> QueryResult<Vec<AlarmWithRoom>> {
    let mut query = alarms::table
        .inner_join(rooms::table)
        .select((Alarm::as_select(), rooms::room_number))
        .order(alarms::raised_at.desc())
        .limit(f.limit)
        .offset(f.offset)
        .into_boxed();

    if let Some(room_id) = f.room_id {
        query = query.filter(alarms::room_id.eq(room_id));
    }
    if let Some(kind) = f.kind {
        query = query.filter(alarms::kind.eq(kind.as_str()));
    }
    match f.open {
        Some(true) => query = query.filter(alarms::resolved_at.is_null()),
        Some(false) => query = query.filter(alarms::resolved_at.is_not_null()),
        None => {}
    }
    query.load(conn).await
}

pub async fn insert(conn: &mut DbConn, new: &NewAlarm) -> QueryResult<Alarm> {
    diesel::insert_into(alarms::table)
        .values(new)
        .returning(Alarm::as_returning())
        .get_result(conn)
        .await
}

/// `None` if already acknowledged or resolved.
pub async fn acknowledge(conn: &mut DbConn, id: Uuid, by: Uuid) -> QueryResult<Option<Alarm>> {
    diesel::update(
        alarms::table
            .find(id)
            .filter(alarms::acknowledged_at.is_null())
            .filter(alarms::resolved_at.is_null()),
    )
    .set((
        alarms::acknowledged_at.eq(diesel::dsl::now),
        alarms::acknowledged_by.eq(by),
    ))
    .returning(Alarm::as_returning())
    .get_result(conn)
    .await
    .optional()
}

/// `None` if already resolved.
pub async fn resolve(conn: &mut DbConn, id: Uuid) -> QueryResult<Option<Alarm>> {
    diesel::update(alarms::table.find(id).filter(alarms::resolved_at.is_null()))
        .set(alarms::resolved_at.eq(diesel::dsl::now))
        .returning(Alarm::as_returning())
        .get_result(conn)
        .await
        .optional()
}
