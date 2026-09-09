use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::schema::alarms;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlarmKind {
    TemperatureLow,
    TemperatureHigh,
    HumidityLow,
    HumidityHigh,
}

impl AlarmKind {
    pub const ALL: [AlarmKind; 4] = [
        Self::TemperatureLow,
        Self::TemperatureHigh,
        Self::HumidityLow,
        Self::HumidityHigh,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TemperatureLow => "temperature_low",
            Self::TemperatureHigh => "temperature_high",
            Self::HumidityLow => "humidity_low",
            Self::HumidityHigh => "humidity_high",
        }
    }
}

impl std::str::FromStr for AlarmKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "temperature_low" => Ok(Self::TemperatureLow),
            "temperature_high" => Ok(Self::TemperatureHigh),
            "humidity_low" => Ok(Self::HumidityLow),
            "humidity_high" => Ok(Self::HumidityHigh),
            other => Err(format!("unknown alarm kind '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = alarms)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Alarm {
    pub id: Uuid,
    pub room_id: Uuid,
    pub measurement_id: i64,
    pub kind: String,
    pub measured_value: BigDecimal,
    pub threshold_value: BigDecimal,
    pub raised_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl Alarm {
    pub fn kind(&self) -> AlarmKind {
        self.kind
            .parse()
            .expect("kind is constrained by the database")
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = alarms)]
pub struct NewAlarm {
    pub room_id: Uuid,
    pub measurement_id: i64,
    pub kind: String,
    pub measured_value: BigDecimal,
    pub threshold_value: BigDecimal,
}
