use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::db::schema::climate_thresholds;

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = climate_thresholds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ClimateThreshold {
    pub id: Uuid,
    /// `None` = the single global default.
    pub room_id: Option<Uuid>,
    pub temperature_min: BigDecimal,
    pub temperature_max: BigDecimal,
    pub humidity_min: BigDecimal,
    pub humidity_max: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The four limits, used for both insert and update.
#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = climate_thresholds)]
pub struct ThresholdValues {
    pub temperature_min: BigDecimal,
    pub temperature_max: BigDecimal,
    pub humidity_min: BigDecimal,
    pub humidity_max: BigDecimal,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = climate_thresholds)]
pub struct NewThreshold {
    pub room_id: Option<Uuid>,
    #[diesel(embed)]
    pub values: ThresholdValues,
}
