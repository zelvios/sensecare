use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{
        DbConn,
        schema::{rooms, service_calls},
    },
    models::service_call::{NewServiceCall, ServiceCall, ServiceCallStatus},
};

/// A call together with its room number.
pub type CallWithRoom = (ServiceCall, String);

pub struct CallFilter {
    pub room_id: Option<Uuid>,
    pub status: Option<ServiceCallStatus>,
    pub limit: i64,
    pub offset: i64,
}

/// Looks up a call by id, returns `None` if no such call exists.
pub async fn find_by_id(conn: &mut DbConn, id: Uuid) -> QueryResult<Option<CallWithRoom>> {
    service_calls::table
        .inner_join(rooms::table)
        .filter(service_calls::id.eq(id))
        .select((ServiceCall::as_select(), rooms::room_number))
        .first(conn)
        .await
        .optional()
}

/// The room's current unclosed call, if any.
pub async fn open_for_room(conn: &mut DbConn, room_id: Uuid) -> QueryResult<Option<ServiceCall>> {
    service_calls::table
        .filter(service_calls::room_id.eq(room_id))
        .filter(service_calls::status.ne(ServiceCallStatus::Closed.as_str()))
        .select(ServiceCall::as_select())
        .order(service_calls::created_at.desc())
        .first(conn)
        .await
        .optional()
}

/// Lists calls, newest first, paged.
pub async fn list(conn: &mut DbConn, f: &CallFilter) -> QueryResult<Vec<CallWithRoom>> {
    let mut query = service_calls::table
        .inner_join(rooms::table)
        .select((ServiceCall::as_select(), rooms::room_number))
        .order(service_calls::created_at.desc())
        .limit(f.limit)
        .offset(f.offset)
        .into_boxed();

    if let Some(room_id) = f.room_id {
        query = query.filter(service_calls::room_id.eq(room_id));
    }
    if let Some(status) = f.status {
        query = query.filter(service_calls::status.eq(status.as_str()));
    }
    query.load(conn).await
}

pub async fn insert(conn: &mut DbConn, new: &NewServiceCall) -> QueryResult<ServiceCall> {
    diesel::insert_into(service_calls::table)
        .values(new)
        .returning(ServiceCall::as_returning())
        .get_result(conn)
        .await
}

/// open -> in_progress. `None` if the call was not open (already acknowledged, closed or missing).
pub async fn acknowledge(
    conn: &mut DbConn,
    id: Uuid,
    by: Uuid,
) -> QueryResult<Option<ServiceCall>> {
    diesel::update(
        service_calls::table
            .find(id)
            .filter(service_calls::status.eq(ServiceCallStatus::Open.as_str())),
    )
    .set((
        service_calls::status.eq(ServiceCallStatus::InProgress.as_str()),
        service_calls::acknowledged_at.eq(diesel::dsl::now),
        service_calls::acknowledged_by.eq(by),
    ))
    .returning(ServiceCall::as_returning())
    .get_result(conn)
    .await
    .optional()
}

/// open|in_progress -> closed. `None` if already closed or missing.
pub async fn close(
    conn: &mut DbConn,
    id: Uuid,
    by: Uuid,
    note: Option<&str>,
) -> QueryResult<Option<ServiceCall>> {
    diesel::update(
        service_calls::table
            .find(id)
            .filter(service_calls::status.ne(ServiceCallStatus::Closed.as_str())),
    )
    .set((
        service_calls::status.eq(ServiceCallStatus::Closed.as_str()),
        service_calls::closed_at.eq(diesel::dsl::now),
        service_calls::closed_by.eq(by),
        service_calls::note.eq(note),
    ))
    .returning(ServiceCall::as_returning())
    .get_result(conn)
    .await
    .optional()
}

pub async fn set_note(conn: &mut DbConn, id: Uuid, note: Option<&str>) -> QueryResult<ServiceCall> {
    diesel::update(service_calls::table.find(id))
        .set(service_calls::note.eq(note))
        .returning(ServiceCall::as_returning())
        .get_result(conn)
        .await
}
