//! GET /api/v1/audit-log/{entity_type}/{entity_id}   (ViewAuditLog)  -> one record's history, oldest first
//!
//!   /api/v1/audit-log/user/4a5f…                       everything done to that account
//!   /api/v1/audit-log/room/7c1e…                       everything done to that room
//!
//! GET /api/v1/audit-log   (ViewAuditLog)  -> newest entries first, paged
//!
//! Query parameters, all optional and combinable:
//!   entity_type   user | room | device | threshold | stay | alarm | service_call
//!   actor_id      uuid of the user who performed the actions
//!   limit         1-200, default 50
//!   offset        default 0
//!
//! Examples:
//!   /api/v1/audit-log                                  latest 50 entries
//!   /api/v1/audit-log?entity_type=user                 everything done to user accounts
//!   /api/v1/audit-log?actor_id=4a5f…&limit=10          the last 10 things one admin did
//!   /api/v1/audit-log?entity_type=device&offset=50     page 2 of device changes

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    error::{ApiError, ErrorResponse},
    models::{audit::AuditEntry, role::Permission},
    repos,
    routes::extractors::CurrentUser,
    state::AppState,
};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(for_entity))
}

#[derive(Deserialize, IntoParams)]
pub struct AuditQuery {
    /// user | room | device | threshold | stay | alarm | service_call
    pub entity_type: Option<String>,
    /// Only actions performed by this user.
    pub actor_id: Option<Uuid>,
    /// 1-200
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

/// Newest audit entries first, optionally filtered by entity type and actor.
#[utoipa::path(
    get, path = "/", tag = "audit", operation_id = "list_audit_log",
    params(AuditQuery),
    responses((status = 200, body = Vec<AuditEntry>), (status = 403, body = ErrorResponse)),
    security(("session_cookie" = []), ("bearer" = []))
)]
async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(q): Query<AuditQuery>,
) -> Result<Json<Vec<AuditEntry>>, ApiError> {
    user.require(Permission::ViewAuditLog)?;
    let limit = q.limit.clamp(1, 200);
    let mut conn = state.pool.get().await?;
    let entries = repos::audit::list(
        &mut conn,
        q.entity_type.as_deref(),
        q.actor_id,
        limit,
        q.offset.max(0),
    )
    .await?;
    Ok(Json(entries))
}

/// Full history of one record, oldest first.
#[utoipa::path(
    get, path = "/{entity_type}/{entity_id}", tag = "audit", operation_id = "audit_history",
    params(("entity_type" = String, Path), ("entity_id" = String, Path)),
    responses((status = 200, body = Vec<AuditEntry>), (status = 403, body = ErrorResponse)),
    security(("session_cookie" = []), ("bearer" = []))
)]
async fn for_entity(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((entity_type, entity_id)): Path<(String, String)>,
) -> Result<Json<Vec<AuditEntry>>, ApiError> {
    user.require(Permission::ViewAuditLog)?;
    let mut conn = state.pool.get().await?;
    let entries = repos::audit::for_entity(&mut conn, &entity_type, &entity_id).await?;
    Ok(Json(entries))
}
