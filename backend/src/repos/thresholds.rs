use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    db::{DbConn, schema::climate_thresholds},
    models::threshold::{ClimateThreshold, NewThreshold, ThresholdValues},
};

/// The global default row (room_id IS NULL). Seeded by the migration, so `None` means a broken database.
pub async fn global(conn: &mut DbConn) -> QueryResult<Option<ClimateThreshold>> {
    climate_thresholds::table
        .filter(climate_thresholds::room_id.is_null())
        .select(ClimateThreshold::as_select())
        .first(conn)
        .await
        .optional()
}

/// The room's own override, if it has one.
pub async fn for_room(conn: &mut DbConn, room_id: Uuid) -> QueryResult<Option<ClimateThreshold>> {
    climate_thresholds::table
        .filter(climate_thresholds::room_id.eq(room_id))
        .select(ClimateThreshold::as_select())
        .first(conn)
        .await
        .optional()
}

pub async fn insert(conn: &mut DbConn, new: &NewThreshold) -> QueryResult<ClimateThreshold> {
    diesel::insert_into(climate_thresholds::table)
        .values(new)
        .returning(ClimateThreshold::as_returning())
        .get_result(conn)
        .await
}

pub async fn update(
    conn: &mut DbConn,
    id: Uuid,
    values: &ThresholdValues,
) -> QueryResult<ClimateThreshold> {
    diesel::update(climate_thresholds::table.find(id))
        .set(values)
        .returning(ClimateThreshold::as_returning())
        .get_result(conn)
        .await
}

/// Deletes a room override. Never used on the global row.
pub async fn delete(conn: &mut DbConn, id: Uuid) -> QueryResult<()> {
    diesel::delete(climate_thresholds::table.find(id))
        .execute(conn)
        .await
        .map(|_| ())
}
