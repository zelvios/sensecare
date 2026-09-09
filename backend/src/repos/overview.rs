//! Grouped queries for the room overview. Each returns one row per room that has
//! data, keyed by room_id. The service group them onto the room list.

use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::db::{
    DbConn,
    schema::{alarms, devices, measurements, service_calls, stays, users},
};

/// (room_id, temperature_c, humidity_pct, measured_at): the newest reading per room.
pub async fn latest_per_room(
    conn: &mut DbConn,
) -> QueryResult<Vec<(Uuid, BigDecimal, BigDecimal, DateTime<Utc>)>> {
    measurements::table
        .distinct_on(measurements::room_id)
        .order((measurements::room_id, measurements::measured_at.desc()))
        .select((
            measurements::room_id,
            measurements::temperature_c,
            measurements::humidity_pct,
            measurements::measured_at,
        ))
        .load(conn)
        .await
}

/// (room_id, user_id, display_name, checked_in_at) for every open stay.
pub async fn occupants(conn: &mut DbConn) -> QueryResult<Vec<(Uuid, Uuid, String, DateTime<Utc>)>> {
    stays::table
        .inner_join(users::table)
        .filter(stays::checked_out_at.is_null())
        .select((
            stays::room_id,
            stays::user_id,
            users::display_name,
            stays::checked_in_at,
        ))
        .load(conn)
        .await
}

/// (room_id, device_id, is_active, last_seen_at) for every device assigned to a room.
pub async fn assigned_devices(
    conn: &mut DbConn,
) -> QueryResult<Vec<(Uuid, Uuid, bool, Option<DateTime<Utc>>)>> {
    devices::table
        .filter(devices::room_id.is_not_null())
        .select((
            devices::room_id.assume_not_null(),
            devices::id,
            devices::is_active,
            devices::last_seen_at,
        ))
        .load(conn)
        .await
}

/// (room_id, call_id, status, created_at) for every unclosed call.
pub async fn open_calls(
    conn: &mut DbConn,
) -> QueryResult<Vec<(Uuid, Uuid, String, DateTime<Utc>)>> {
    service_calls::table
        .filter(service_calls::status.ne("closed"))
        .order(service_calls::created_at.asc())
        .select((
            service_calls::room_id,
            service_calls::id,
            service_calls::status,
            service_calls::created_at,
        ))
        .load(conn)
        .await
}

/// (room_id, alarm_id, kind, raised_at, acknowledged) for every unresolved alarm.
pub async fn open_alarms(
    conn: &mut DbConn,
) -> QueryResult<Vec<(Uuid, Uuid, String, DateTime<Utc>, bool)>> {
    alarms::table
        .filter(alarms::resolved_at.is_null())
        .order(alarms::raised_at.asc())
        .select((
            alarms::room_id,
            alarms::id,
            alarms::kind,
            alarms::raised_at,
            alarms::acknowledged_at.is_not_null(),
        ))
        .load(conn)
        .await
}
