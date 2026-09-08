use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::schema::devices;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Device {
    pub id: Uuid,
    pub room_id: Option<Uuid>,
    pub key_hash: String,
    pub label: Option<String>,
    pub firmware_version: Option<String>,
    pub is_active: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = devices)]
pub struct NewDevice<'a> {
    pub room_id: Option<Uuid>,
    pub key_hash: &'a str,
    pub label: Option<&'a str>,
    pub firmware_version: Option<&'a str>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = devices)]
pub struct DeviceUpdate<'a> {
    pub label: Option<&'a str>,
    pub firmware_version: Option<&'a str>,
}
