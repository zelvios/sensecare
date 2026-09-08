use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{
        DbConn,
        schema::{rooms, stays, users},
    },
    models::stay::{NewStay, Stay},
};

/// A stay together with the room number and the client's display name.
pub type StayWithNames = (Stay, String, String);

pub struct StayFilter {
    pub room_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    /// `Some(true)` open stays only, `Some(false)` closed only, `None` both.
    pub open: Option<bool>,
    pub limit: i64,
    pub offset: i64,
}

/// Looks up a stay by id, returns `None` if no such stay exists.
pub async fn find_by_id(conn: &mut DbConn, id: Uuid) -> QueryResult<Option<StayWithNames>> {
    stays::table
        .inner_join(rooms::table)
        .inner_join(users::table)
        .filter(stays::id.eq(id))
        .select((Stay::as_select(), rooms::room_number, users::display_name))
        .first(conn)
        .await
        .optional()
}

/// The users open stay, if any.
pub async fn open_for_user(conn: &mut DbConn, user_id: Uuid) -> QueryResult<Option<StayWithNames>> {
    stays::table
        .inner_join(rooms::table)
        .inner_join(users::table)
        .filter(stays::user_id.eq(user_id))
        .filter(stays::checked_out_at.is_null())
        .select((Stay::as_select(), rooms::room_number, users::display_name))
        .first(conn)
        .await
        .optional()
}

/// True if the user currently has an open stay in the room.
pub async fn user_occupies_room(
    conn: &mut DbConn,
    user_id: Uuid,
    room_id: Uuid,
) -> QueryResult<bool> {
    let n: i64 = stays::table
        .filter(stays::user_id.eq(user_id))
        .filter(stays::room_id.eq(room_id))
        .filter(stays::checked_out_at.is_null())
        .count()
        .get_result(conn)
        .await?;
    Ok(n > 0)
}

/// Lists stays matching the filter, newest check-in first, paged.
pub async fn list(conn: &mut DbConn, f: &StayFilter) -> QueryResult<Vec<StayWithNames>> {
    let mut query = stays::table
        .inner_join(rooms::table)
        .inner_join(users::table)
        .select((Stay::as_select(), rooms::room_number, users::display_name))
        .order(stays::checked_in_at.desc())
        .limit(f.limit)
        .offset(f.offset)
        .into_boxed();

    if let Some(room_id) = f.room_id {
        query = query.filter(stays::room_id.eq(room_id));
    }
    if let Some(user_id) = f.user_id {
        query = query.filter(stays::user_id.eq(user_id));
    }
    match f.open {
        Some(true) => query = query.filter(stays::checked_out_at.is_null()),
        Some(false) => query = query.filter(stays::checked_out_at.is_not_null()),
        None => {}
    }
    query.load(conn).await
}

/// Opens a stay. Fails with a unique violation if the room or the user already has an open one.
pub async fn insert(conn: &mut DbConn, new: &NewStay) -> QueryResult<Stay> {
    diesel::insert_into(stays::table)
        .values(new)
        .returning(Stay::as_returning())
        .get_result(conn)
        .await
}

/// Closes an open stay. Returns `None` if it was already closed (or does not exist).
pub async fn check_out(conn: &mut DbConn, id: Uuid) -> QueryResult<Option<Stay>> {
    diesel::update(
        stays::table
            .find(id)
            .filter(stays::checked_out_at.is_null()),
    )
    .set(stays::checked_out_at.eq(diesel::dsl::now))
    .returning(Stay::as_returning())
    .get_result(conn)
    .await
    .optional()
}
