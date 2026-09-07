//! One call per action, made from the service that performs it,
//! never from routes. So an action can't be logged without actually happening,
//! and can't happen without being logged.
//!
//!   audit::record(conn, Some(&actor), AuditAction::UserDeactivated, &id, None).await?;
//!   audit::record(conn, None, AuditAction::AlarmResolved, &alarm_id, Some(json!({...}))).await?;

use serde_json::Value;

use crate::{
    db::DbConn,
    error::ApiError,
    models::audit::{AuditAction, NewAuditEntry},
    repos,
    services::auth::AuthenticatedUser,
};

/// Appends an audit entry. `actor` is `None` for actions the system takes on its own.
pub async fn record(
    conn: &mut DbConn,
    actor: Option<&AuthenticatedUser>,
    action: AuditAction,
    entity_id: &impl ToString,
    details: Option<Value>,
) -> Result<(), ApiError> {
    let entity_id = entity_id.to_string();

    repos::audit::insert(
        conn,
        &NewAuditEntry {
            actor_id: actor.map(|a| a.id),
            action: action.as_str(),
            entity_type: action.entity_type(),
            entity_id: &entity_id,
            details: details.clone(),
        },
    )
    .await?;

    tracing::info!(
        target: "audit",
        actor = actor.map(|a| a.username.as_str()).unwrap_or("system"),
        action = action.as_str(),
        entity = %entity_id,
        details = %details.map(|d| d.to_string()).unwrap_or_default(),
        "audit"
    );
    Ok(())
}
