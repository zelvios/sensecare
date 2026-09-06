use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{
        DbConn,
        schema::{roles, users},
    },
    models::user::{NewUser, User},
};

/// A user together with the name of their role.
pub type UserWithRole = (User, String);

/// Looks up a user by username, returns `None` if no such user exists.
pub async fn find_by_username(
    conn: &mut DbConn,
    username: &str,
) -> QueryResult<Option<UserWithRole>> {
    users::table
        .inner_join(roles::table)
        .filter(users::username.eq(username))
        .select((User::as_select(), roles::name))
        .first(conn)
        .await
        .optional()
}

/// Looks up a user by id, returns `None` if no such user exists.
pub async fn find_by_id(conn: &mut DbConn, id: Uuid) -> QueryResult<Option<UserWithRole>> {
    users::table
        .inner_join(roles::table)
        .filter(users::id.eq(id))
        .select((User::as_select(), roles::name))
        .first(conn)
        .await
        .optional()
}

/// Creates a user and returns the stored row (with generated id and timestamps).
pub async fn insert(conn: &mut DbConn, new: &NewUser<'_>) -> QueryResult<User> {
    diesel::insert_into(users::table)
        .values(new)
        .returning(User::as_returning())
        .get_result(conn)
        .await
}

/// Sets `last_login_at` to now, called after a successful login.
pub async fn touch_last_login(conn: &mut DbConn, id: Uuid) -> QueryResult<()> {
    diesel::update(users::table.find(id))
        .set(users::last_login_at.eq(diesel::dsl::now))
        .execute(conn)
        .await
        .map(|_| ())
}

/// Total number of users, used to decide whether to bootstrap the first admin.
pub async fn count(conn: &mut DbConn) -> QueryResult<i64> {
    users::table.count().get_result(conn).await
}

/// Resolves a role name (`client`, `staff`, `admin`) to its id in the `roles` table.
pub async fn role_id_by_name(conn: &mut DbConn, name: &str) -> QueryResult<i16> {
    roles::table
        .filter(roles::name.eq(name))
        .select(roles::id)
        .first(conn)
        .await
}
