use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::schema::service_calls;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ServiceCallStatus {
    Open,
    InProgress,
    Closed,
}

impl ServiceCallStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::Closed => "closed",
        }
    }
}

impl std::str::FromStr for ServiceCallStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "in_progress" => Ok(Self::InProgress),
            "closed" => Ok(Self::Closed),
            other => Err(format!("unknown service call status '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = service_calls)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ServiceCall {
    pub id: Uuid,
    pub room_id: Uuid,
    pub device_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_by: Option<Uuid>,
    pub note: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl ServiceCall {
    pub fn status(&self) -> ServiceCallStatus {
        self.status
            .parse()
            .expect("status is constrained by the database")
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = service_calls)]
pub struct NewServiceCall {
    pub room_id: Uuid,
    pub device_id: Uuid,
}
