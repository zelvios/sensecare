use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::schema::stays;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, ToSchema)]
#[diesel(table_name = stays)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Stay {
    pub id: Uuid,
    pub room_id: Uuid,
    pub user_id: Uuid,
    pub checked_in_at: DateTime<Utc>,
    pub checked_out_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = stays)]
pub struct NewStay {
    pub room_id: Uuid,
    pub user_id: Uuid,
}
