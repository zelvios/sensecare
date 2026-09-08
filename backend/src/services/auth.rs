use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use serde::Serialize;
use uuid::Uuid;
use utoipa::ToSchema;

use crate::{
    db::DbConn,
    error::ApiError,
    models::{
        audit::AuditAction,
        role::{Permission, Role},
        session::NewSession,
        user::NewUser,
    },
    repos,
    services::audit,
};

/// What every authenticated request knows about the caller.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    #[serde(skip)]
    pub session_id: Uuid,
}

impl AuthenticatedUser {
    /// Returns Forbidden unless the caller's role grants the permission.
    /// First line of every protected handler: `user.require(Permission::X)?;`
    pub fn require(&self, permission: Permission) -> Result<(), ApiError> {
        if self.role.has(permission) {
            Ok(())
        } else {
            Err(ApiError::Forbidden)
        }
    }
}

/// Argon2 is deliberately slow (~50–100 ms), so hashing runs on a blocking thread.
pub async fn hash_password(password: String) -> Result<String, ApiError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| anyhow::anyhow!("password hashing failed: {e}"))
    })
    .await
    .map_err(|e| anyhow::anyhow!(e))?
    .map_err(ApiError::from)
}

pub async fn verify_password(password: String, hash: String) -> Result<bool, ApiError> {
    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&hash).map_err(|e| anyhow::anyhow!("bad hash: {e}"))?;
        Ok::<bool, anyhow::Error>(
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok(),
        )
    })
    .await
    .map_err(|e| anyhow::anyhow!(e))?
    .map_err(ApiError::from)
}

/// Verifies credentials and opens a session. Returns Unauthorized for a wrong
/// username, wrong password or deactivated account.
pub async fn login(
    conn: &mut DbConn,
    username: &str,
    password: &str,
    user_agent: Option<&str>,
    ttl_hours: i64,
) -> Result<(Uuid, AuthenticatedUser), ApiError> {
    let Some((user, role_name)) = repos::users::find_by_username(conn, username).await? else {
        // Burn the same time as a real verification so timing doesn't leak existence.
        let _ = verify_password(password.to_owned(), DUMMY_HASH.to_owned()).await;
        return Err(ApiError::Unauthorized);
    };

    if !verify_password(password.to_owned(), user.password_hash.clone()).await? {
        return Err(ApiError::Unauthorized);
    }
    // Only reached with a correct password, so this reveals nothing to a guesser
    // and tells a real user why they can't get in.
    if !user.is_active {
        return Err(ApiError::AccountDeactivated);
    }

    let role: Role = role_name.parse().map_err(|e: String| anyhow::anyhow!(e))?;

    let session = repos::sessions::insert(
        conn,
        &NewSession {
            user_id: user.id,
            user_agent,
            expires_at: Utc::now() + Duration::hours(ttl_hours),
        },
    )
    .await?;
    repos::users::touch_last_login(conn, user.id).await?;

    Ok((
        session.id,
        AuthenticatedUser {
            id: user.id,
            username: user.username,
            display_name: user.display_name,
            role,
            session_id: session.id,
        },
    ))
}

pub async fn logout(conn: &mut DbConn, session_id: Uuid) -> Result<(), ApiError> {
    repos::sessions::delete(conn, session_id).await?;
    Ok(())
}

/// Resolves a session token to a user. Called by the `CurrentUser` extractor
/// on every authenticated request.
pub async fn authenticate(conn: &mut DbConn, token: Uuid) -> Result<AuthenticatedUser, ApiError> {
    let Some((session, user, role_name)) =
        repos::sessions::find_valid_with_user(conn, token).await?
    else {
        return Err(ApiError::Unauthorized);
    };

    if !user.is_active {
        // Deactivated after logging in: drop every session so the token can't be retried.
        repos::sessions::delete_all_for_user(conn, user.id).await?;
        return Err(ApiError::AccountDeactivated);
    }

    repos::sessions::touch(conn, session.id).await?;

    let role: Role = role_name.parse().map_err(|e: String| anyhow::anyhow!(e))?;
    Ok(AuthenticatedUser {
        id: user.id,
        username: user.username,
        display_name: user.display_name,
        role,
        session_id: session.id,
    })
}

/// Creates the first admin if the users table is empty. Lets a fresh install log in without
/// hand-inserting a password hash.
pub async fn bootstrap_admin(conn: &mut DbConn, password: &str) -> Result<bool, ApiError> {
    if repos::users::count(conn).await? > 0 {
        return Ok(false);
    }
    let hash = hash_password(password.to_owned()).await?;
    let role_id = repos::users::role_id_by_name(conn, Role::Admin.as_str()).await?;

    let user = repos::users::insert(
        conn,
        &NewUser {
            username: "admin",
            display_name: "Administrator",
            password_hash: &hash,
            role_id,
        },
    )
    .await?;

    audit::record(
        conn,
        None,
        AuditAction::UserCreated,
        &user.id,
        Some(serde_json::json!({ "bootstrap": true })),
    )
    .await?;
    Ok(true)
}

/// A valid Argon2id hash of a random string, only used to equalise timing.
const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHRzb21lc2FsdA$x9zZ3Q5u0Y0m6hK4dKq7tXhQZ0yS8v8pQ3g2tWq6kTQ";
