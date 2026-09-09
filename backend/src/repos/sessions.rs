use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{
        DbConn,
        schema::{roles, sessions, users},
    },
    models::{
        session::{NewSession, Session},
        user::User,
    },
};

pub async fn insert(conn: &mut DbConn, new: &NewSession<'_>) -> QueryResult<Session> {
    diesel::insert_into(sessions::table)
        .values(new)
        .returning(Session::as_returning())
        .get_result(conn)
        .await
}

/// Returns the session only if it exists and has not expired.
pub async fn find_valid(conn: &mut DbConn, id: Uuid) -> QueryResult<Option<Session>> {
    sessions::table
        .find(id)
        .filter(sessions::expires_at.gt(diesel::dsl::now))
        .select(Session::as_select())
        .first(conn)
        .await
        .optional()
}

/// Session together with its owner and the owner's role name, in one query.
/// `None` if the session doesn't exist or has expired.
pub async fn find_valid_with_user(
    conn: &mut DbConn,
    id: Uuid,
) -> QueryResult<Option<(Session, User, String)>> {
    sessions::table
        .inner_join(users::table.inner_join(roles::table))
        .filter(sessions::id.eq(id))
        .filter(sessions::expires_at.gt(diesel::dsl::now))
        .select((Session::as_select(), User::as_select(), roles::name))
        .first(conn)
        .await
        .optional()
}

/// Records that the session was just used by setting `last_used_at` to now.
pub async fn touch(conn: &mut DbConn, id: Uuid) -> QueryResult<()> {
    diesel::update(sessions::table.find(id))
        .set(sessions::last_used_at.eq(diesel::dsl::now))
        .execute(conn)
        .await
        .map(|_| ())
}

pub async fn delete(conn: &mut DbConn, id: Uuid) -> QueryResult<()> {
    diesel::delete(sessions::table.find(id))
        .execute(conn)
        .await
        .map(|_| ())
}

pub async fn delete_all_for_user(conn: &mut DbConn, user_id: Uuid) -> QueryResult<usize> {
    diesel::delete(sessions::table.filter(sessions::user_id.eq(user_id)))
        .execute(conn)
        .await
}

/// Logs the user out everywhere except the session they are using right now.
pub async fn delete_all_for_user_except(
    conn: &mut DbConn,
    user_id: Uuid,
    keep: Uuid,
) -> QueryResult<usize> {
    diesel::delete(
        sessions::table
            .filter(sessions::user_id.eq(user_id))
            .filter(sessions::id.ne(keep)),
    )
    .execute(conn)
    .await
}
