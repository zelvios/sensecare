use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{DbConn, schema::audit_log},
    models::audit::{AuditEntry, NewAuditEntry},
};

/// Appends one entry. The table is append-only, there is no update or delete.
pub async fn insert(conn: &mut DbConn, new: &NewAuditEntry<'_>) -> QueryResult<()> {
    diesel::insert_into(audit_log::table)
        .values(new)
        .execute(conn)
        .await
        .map(|_| ())
}

/// Newest first, paged. `entity_type` / `actor_id` narrow the result when given.
pub async fn list(
    conn: &mut DbConn,
    entity_type: Option<&str>,
    actor_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> QueryResult<Vec<AuditEntry>> {
    let mut query = audit_log::table
        .select(AuditEntry::as_select())
        .order(audit_log::created_at.desc())
        .limit(limit)
        .offset(offset)
        .into_boxed();
    if let Some(t) = entity_type {
        query = query.filter(audit_log::entity_type.eq(t.to_owned()));
    }
    if let Some(a) = actor_id {
        query = query.filter(audit_log::actor_id.eq(a));
    }
    query.load(conn).await
}

/// Full history of one record, oldest first.
pub async fn for_entity(
    conn: &mut DbConn,
    entity_type: &str,
    entity_id: &str,
) -> QueryResult<Vec<AuditEntry>> {
    audit_log::table
        .filter(audit_log::entity_type.eq(entity_type))
        .filter(audit_log::entity_id.eq(entity_id))
        .select(AuditEntry::as_select())
        .order(audit_log::created_at.asc())
        .load(conn)
        .await
}
