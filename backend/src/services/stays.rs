//! Rules:
//!   - checkin and checkout need `ManageStays` (staff and admin)
//!   - only active clients can be checked in, only into active rooms
//!   - one open stay per room and one per user (database unique indexes)
//!   - listing needs `ViewAllRooms`, and `/me` needs `ViewOwnRoom` which returns the callers own stay
//!   - stays are never edited or deleted: they are the history
//!   - every checkin and checkout writes one audit entry.

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::DbConn,
    error::ApiError,
    models::{
        audit::AuditAction,
        role::{Permission, Role},
        stay::NewStay,
    },
    repos::{
        self,
        stays::{StayFilter, StayWithNames},
    },
    services::{audit, auth::AuthenticatedUser},
};

pub struct ListParams {
    pub room_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub open: Option<bool>,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    p: ListParams,
) -> Result<Vec<StayWithNames>, ApiError> {
    actor.require(Permission::ViewAllRooms)?;
    Ok(repos::stays::list(
        conn,
        &StayFilter {
            room_id: p.room_id,
            user_id: p.user_id,
            open: p.open,
            limit: p.limit.clamp(1, 200),
            offset: p.offset.max(0),
        },
    )
    .await?)
}

pub async fn get(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<StayWithNames, ApiError> {
    actor.require(Permission::ViewAllRooms)?;
    repos::stays::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)
}

/// The caller's own open stay. 404 when they are not checked in anywhere.
pub async fn current_for(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
) -> Result<StayWithNames, ApiError> {
    actor.require(Permission::ViewOwnRoom)?;
    repos::stays::open_for_user(conn, actor.id)
        .await?
        .ok_or(ApiError::NotFound)
}

pub async fn check_in(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<StayWithNames, ApiError> {
    actor.require(Permission::ManageStays)?;

    let (guest, role_name) = repos::users::find_by_id(conn, user_id)
        .await?
        .ok_or_else(|| ApiError::BadRequest("user does not exist".into()))?;
    let role: Role = role_name.parse().map_err(|e: String| anyhow::anyhow!(e))?;
    if role != Role::Client {
        return Err(ApiError::BadRequest(
            "only clients can be checked in".into(),
        ));
    }
    if !guest.is_active {
        return Err(ApiError::BadRequest("user is deactivated".into()));
    }

    let room = repos::rooms::find_by_id(conn, room_id)
        .await?
        .ok_or_else(|| ApiError::BadRequest("room does not exist".into()))?;
    if !room.is_active {
        return Err(ApiError::BadRequest("room is deactivated".into()));
    }

    let stay = repos::stays::insert(conn, &NewStay { room_id, user_id })
        .await
        .map_err(map_unique)?;

    audit::record(
        conn,
        Some(actor),
        AuditAction::StayCheckedIn,
        &stay.id,
        Some(json!({ "room_id": room_id, "user_id": user_id })),
    )
    .await?;

    Ok((stay, room.room_number, guest.display_name))
}

pub async fn check_out(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    id: Uuid,
) -> Result<StayWithNames, ApiError> {
    actor.require(Permission::ManageStays)?;

    let (existing, room_number, display_name) = repos::stays::find_by_id(conn, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if existing.checked_out_at.is_some() {
        return Err(ApiError::Conflict("stay is already checked out".into()));
    }

    let stay = repos::stays::check_out(conn, id)
        .await?
        .ok_or_else(|| ApiError::Conflict("stay is already checked out".into()))?;

    audit::record(
        conn,
        Some(actor),
        AuditAction::StayCheckedOut,
        &id,
        Some(json!({ "room_id": stay.room_id, "user_id": stay.user_id })),
    )
    .await?;

    Ok((stay, room_number, display_name))
}

fn map_unique(e: DieselError) -> ApiError {
    match &e {
        DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
            match info.constraint_name() {
                Some("uq_stays_one_open_per_room") => {
                    ApiError::Conflict("room is already occupied".into())
                }
                Some("uq_stays_one_open_per_user") => {
                    ApiError::Conflict("user already has an open stay".into())
                }
                _ => e.into(),
            }
        }
        _ => e.into(),
    }
}
