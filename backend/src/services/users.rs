//! Rules:
//!   - managing an account needs `Permission::for_managing(target role)`
//!     -> staff manage clients and admins manage everyone
//!   - changing a role needs that permission for the old AND the new role
//!   - nobody deactivates, deletes or demotes their own account
//!   - hard delete needs `DeleteUsers` and is refused by the database if the
//!     user is referenced anywhere (stays, alarms, …)
//! Every mutation writes one audit entry.

use serde_json::json;
use uuid::Uuid;

use crate::{
    db::DbConn,
    error::ApiError,
    models::{
        audit::AuditAction,
        role::{Permission, Role},
        user::{NewUser, User, UserUpdate},
    },
    repos::{self, users::UserFilter},
    services::{
        audit,
        auth::{self, AuthenticatedUser},
    },
};

/// A user with its role resolved to the enum — what routes work with.
pub struct UserRecord {
    pub user: User,
    pub role: Role,
}

fn resolve((user, role_name): (User, String)) -> Result<UserRecord, ApiError> {
    let role = role_name.parse().map_err(|e: String| anyhow::anyhow!(e))?;
    Ok(UserRecord { user, role })
}

/// Loads the target and checks the actor may manage it. Used by every per account operation.
async fn load_managed(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<UserRecord, ApiError> {
    let target = resolve(
        repos::users::find_by_id(conn, id)
            .await?
            .ok_or(ApiError::NotFound)?,
    )?;
    actor.require(Permission::for_managing(target.role))?;
    Ok(target)
}

pub struct ListParams<'a> {
    pub q: Option<&'a str>,
    pub role: Option<Role>,
    pub active: Option<bool>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    p: ListParams<'_>,
) -> Result<Vec<UserRecord>, ApiError> {
    // Staff see clients only, the role filter is forced for them.
    let role = if actor.role.has(Permission::ManageStaff) {
        p.role
    } else {
        actor.require(Permission::ManageClients)?;
        match p.role {
            Some(r) if r != Role::Client => return Err(ApiError::Forbidden),
            _ => Some(Role::Client),
        }
    };

    let rows = repos::users::list(
        conn,
        &UserFilter {
            q: p.q,
            role: role.map(Role::as_str),
            active: p.active,
            limit: p.limit.clamp(1, 200),
            offset: p.offset.max(0),
        },
    )
    .await?;
    rows.into_iter().map(resolve).collect()
}

pub async fn get(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<UserRecord, ApiError> {
    load_managed(conn, actor, id).await
}

pub async fn create(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    username: &str,
    display_name: &str,
    password: &str,
    role: Role,
) -> Result<UserRecord, ApiError> {
    actor.require(Permission::for_managing(role))?;
    let username = username.trim();
    let display_name = display_name.trim();
    validate_username(username)?;
    validate_display_name(display_name)?;
    validate_password(password)?;

    let hash = auth::hash_password(password.to_owned()).await?;
    let role_id = repos::users::role_id_by_name(conn, role.as_str()).await?;
    let user = repos::users::insert(
        conn,
        &NewUser {
            username,
            display_name,
            password_hash: &hash,
            role_id,
        },
    )
    .await?;

    audit::record(
        conn,
        Some(actor),
        AuditAction::UserCreated,
        &user.id,
        Some(json!({ "username": user.username, "role": role.as_str() })),
    )
    .await?;

    Ok(UserRecord { user, role })
}

pub async fn update(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
    display_name: Option<&str>,
    new_role: Option<Role>,
) -> Result<UserRecord, ApiError> {
    let target = load_managed(conn, actor, id).await?;
    let display_name = display_name.map(str::trim);
    if let Some(name) = display_name {
        validate_display_name(name)?;
    }

    let mut changes = serde_json::Map::new();

    if let Some(name) = display_name.filter(|n| *n != target.user.display_name) {
        changes.insert(
            "display_name".into(),
            json!({ "from": target.user.display_name, "to": name }),
        );
    }

    let role_id = match new_role {
        Some(r) if r != target.role => {
            // Promoting a client to staff is "creating a staff account".
            actor.require(Permission::for_managing(r))?;
            if id == actor.id {
                return Err(ApiError::BadRequest("cannot change your own role".into()));
            }
            changes.insert(
                "role".into(),
                json!({ "from": target.role.as_str(), "to": r.as_str() }),
            );
            Some(repos::users::role_id_by_name(conn, r.as_str()).await?)
        }
        _ => None,
    };

    if changes.is_empty() {
        return Ok(target); // nothing to do, no audit entry for a no-op
    }

    let user = repos::users::update(
        conn,
        id,
        &UserUpdate {
            display_name,
            role_id,
        },
    )
    .await?;
    audit::record(
        conn,
        Some(actor),
        AuditAction::UserUpdated,
        &id,
        Some(changes.into()),
    )
    .await?;

    Ok(UserRecord {
        user,
        role: new_role.unwrap_or(target.role),
    })
}

pub async fn set_password(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
    password: &str,
) -> Result<(), ApiError> {
    load_managed(conn, actor, id).await?;
    validate_password(password)?;

    let hash = auth::hash_password(password.to_owned()).await?;
    repos::users::set_password_hash(conn, id, &hash).await?;
    // A changed password invalidates every existing login for that account.
    repos::sessions::delete_all_for_user(conn, id).await?;

    audit::record(
        conn,
        Some(actor),
        AuditAction::UserPasswordChanged,
        &id,
        None,
    )
    .await?;
    Ok(())
}

pub async fn set_active(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
    active: bool,
) -> Result<(), ApiError> {
    if !active && id == actor.id {
        return Err(ApiError::BadRequest(
            "cannot deactivate your own account".into(),
        ));
    }
    let target = load_managed(conn, actor, id).await?;
    if target.user.is_active == active {
        return Ok(()); // already in that state
    }

    repos::users::set_active(conn, id, active).await?;
    if !active {
        repos::sessions::delete_all_for_user(conn, id).await?;
    }

    let action = if active {
        AuditAction::UserActivated
    } else {
        AuditAction::UserDeactivated
    };
    audit::record(conn, Some(actor), action, &id, None).await?;
    Ok(())
}

pub async fn delete(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<(), ApiError> {
    actor.require(Permission::DeleteUsers)?;
    if id == actor.id {
        return Err(ApiError::BadRequest(
            "cannot delete your own account".into(),
        ));
    }
    let target = load_managed(conn, actor, id).await?;

    // Sessions cascade, anything else referencing the user makes the delete fail
    // with a foreign-key violation, that error.rs turns into 400 invalid_reference.
    repos::users::delete(conn, id).await?;

    // The row is gone, so the audit entry carries a snapshot of what it was.
    audit::record(
        conn,
        Some(actor),
        AuditAction::UserDeleted,
        &id,
        Some(json!({
            "username": target.user.username,
            "display_name": target.user.display_name,
            "role": target.role.as_str(),
        })),
    )
    .await?;
    Ok(())
}

fn validate_username(u: &str) -> Result<(), ApiError> {
    let ok = (3..=64).contains(&u.chars().count())
        && u.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, 'æ' | 'ø' | 'å' | '.' | '_' | '-'));
    if ok {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "username must be 3-64 characters: a-z, æøå, 0-9, '.', '_', '-'".into(),
        ))
    }
}

fn validate_display_name(n: &str) -> Result<(), ApiError> {
    if (1..=128).contains(&n.len()) {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "display_name must be 1-128 characters".into(),
        ))
    }
}

fn validate_password(p: &str) -> Result<(), ApiError> {
    if p.len() >= 10 {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "password must be at least 10 characters".into(),
        ))
    }
}
