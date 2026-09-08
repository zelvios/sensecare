use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::schema::rooms;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, ToSchema)]
#[diesel(table_name = rooms)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Room {
    pub id: Uuid,
    pub room_number: String,
    pub name: Option<String>,
    pub floor: Option<i16>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = rooms)]
pub struct NewRoom<'a> {
    pub room_number: &'a str,
    pub name: Option<&'a str>,
    pub floor: Option<i16>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = rooms)]
pub struct RoomUpdate<'a> {
    pub room_number: Option<&'a str>,
    pub name: Option<&'a str>,
    pub floor: Option<i16>,
}
