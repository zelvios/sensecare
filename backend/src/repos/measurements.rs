use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{DbConn, schema::measurements},
    models::measurement::{Measurement, NewMeasurement},
};

pub struct MeasurementRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub limit: i64,
    pub offset: i64,
}

/// Stores a reading and returns the row.
pub async fn insert(conn: &mut DbConn, new: &NewMeasurement) -> QueryResult<Measurement> {
    diesel::insert_into(measurements::table)
        .values(new)
        .returning(Measurement::as_returning())
        .get_result(conn)
        .await
}

/// Readings for a room within `[from, to]`, newest first, paged.
pub async fn for_room(
    conn: &mut DbConn,
    room_id: Uuid,
    r: &MeasurementRange,
) -> QueryResult<Vec<Measurement>> {
    measurements::table
        .filter(measurements::room_id.eq(room_id))
        .filter(measurements::measured_at.ge(r.from))
        .filter(measurements::measured_at.le(r.to))
        .select(Measurement::as_select())
        .order(measurements::measured_at.desc())
        .limit(r.limit)
        .offset(r.offset)
        .load(conn)
        .await
}

/// The most recent reading for a room, if any.
pub async fn latest_for_room(conn: &mut DbConn, room_id: Uuid) -> QueryResult<Option<Measurement>> {
    measurements::table
        .filter(measurements::room_id.eq(room_id))
        .select(Measurement::as_select())
        .order(measurements::measured_at.desc())
        .first(conn)
        .await
        .optional()
}
