use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{
        DbConn,
        schema::{roles, users},
    },
    models::user::{NewUser, User, UserUpdate},
};

/// A user together with the name of their role.
pub type UserWithRole = (User, String);

/// Filters for `list`. `q` matches username or display name, case-insensitive.
pub struct UserFilter<'a> {
    pub q: Option<&'a str>,
    pub role: Option<&'a str>,
    pub active: Option<bool>,
    pub limit: i64,
    pub offset: i64,
}

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

/// Lists users matching the filter, ordered by username, paged.
pub async fn list(conn: &mut DbConn, f: &UserFilter<'_>) -> QueryResult<Vec<UserWithRole>> {
    let mut query = users::table
        .inner_join(roles::table)
        .select((User::as_select(), roles::name))
        .order(users::username.asc())
        .limit(f.limit)
        .offset(f.offset)
        .into_boxed();

    if let Some(q) = f.q.map(str::trim).filter(|s| !s.is_empty()) {
        let pattern = format!("%{}%", escape_like(q));
        query = query.filter(
            users::username
                .ilike(pattern.clone())
                .or(users::display_name.ilike(pattern)),
        );
    }
    if let Some(role) = f.role {
        query = query.filter(roles::name.eq(role.to_owned()));
    }
    if let Some(active) = f.active {
        query = query.filter(users::is_active.eq(active));
    }
    query.load(conn).await
}

/// Creates a user and returns the stored row (with generated id and timestamps).
pub async fn insert(conn: &mut DbConn, new: &NewUser<'_>) -> QueryResult<User> {
    diesel::insert_into(users::table)
        .values(new)
        .returning(User::as_returning())
        .get_result(conn)
        .await
}

/// Applies a partial update; `None` fields in `changes` are left untouched.
pub async fn update(conn: &mut DbConn, id: Uuid, changes: &UserUpdate<'_>) -> QueryResult<User> {
    diesel::update(users::table.find(id))
        .set(changes)
        .returning(User::as_returning())
        .get_result(conn)
        .await
}

/// Replaces the stored password hash.
pub async fn set_password_hash(conn: &mut DbConn, id: Uuid, hash: &str) -> QueryResult<()> {
    diesel::update(users::table.find(id))
        .set(users::password_hash.eq(hash))
        .execute(conn)
        .await
        .map(|_| ())
}

/// Soft delete / restore: sets `is_active`.
pub async fn set_active(conn: &mut DbConn, id: Uuid, active: bool) -> QueryResult<()> {
    diesel::update(users::table.find(id))
        .set(users::is_active.eq(active))
        .execute(conn)
        .await
        .map(|_| ())
}

/// Hard delete. Fails with a foreign-key violation if anything still references the user.
pub async fn delete(conn: &mut DbConn, id: Uuid) -> QueryResult<()> {
    diesel::delete(users::table.find(id))
        .execute(conn)
        .await
        .map(|_| ())
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

/// Prepares user input for the SQL `ILIKE` pattern used in `list`.
///
/// In SQL pattern matching, `%` means "any characters" and `_` means "any single
/// character". If a user searches for "50%" or "j_hansen", those characters must be
/// treated literally, so they are escaped with a backslash before being wrapped in
/// `%…%` for the contains-match.
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
