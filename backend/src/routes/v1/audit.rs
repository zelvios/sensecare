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
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::{audit::AuditEntry, role::Permission},
    repos,
    routes::extractors::CurrentUser,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/{entity_type}/{entity_id}", get(for_entity))
}

#[derive(Deserialize)]
pub struct AuditQuery {
    pub entity_type: Option<String>,
    pub actor_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

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

/// Full history of one record, oldest first, e.g. everything ever done to a user.
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
