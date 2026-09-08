use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::schema::measurements;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = measurements)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Measurement {
    pub id: i64,
    pub room_id: Uuid,
    pub device_id: Uuid,
    pub temperature_c: BigDecimal,
    pub humidity_pct: BigDecimal,
    /// Device clock.
    pub measured_at: DateTime<Utc>,
    /// Server clock. The gap to `measured_at` shows drift or backlog.
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = measurements)]
pub struct NewMeasurement {
    pub room_id: Uuid,
    pub device_id: Uuid,
    pub temperature_c: BigDecimal,
    pub humidity_pct: BigDecimal,
    /// Device clock, or the server time when the device sent none.
    pub measured_at: DateTime<Utc>,
    /// Server time. Equal to `measured_at` exactly when the server supplied it.
    pub received_at: DateTime<Utc>,
}
