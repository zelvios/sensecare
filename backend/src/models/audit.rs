use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

use crate::db::schema::audit_log;

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = audit_log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AuditEntry {
    pub id: i64,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = audit_log)]
pub struct NewAuditEntry<'a> {
    pub actor_id: Option<Uuid>,
    pub action: &'a str,
    pub entity_type: &'a str,
    pub entity_id: &'a str,
    pub details: Option<serde_json::Value>,
}

/// Every auditable action, named `<entity>.<verb>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    UserCreated,
    UserUpdated,
    UserPasswordChanged,
    UserDeactivated,
    UserActivated,
    RoomCreated,
    RoomUpdated,
    RoomDeactivated,
    RoomActivated,
    DeviceRegistered,
    DeviceAssigned,
    DeviceKeyRotated,
    DeviceDeactivated,
    ThresholdUpdated,
    StayCheckedIn,
    StayCheckedOut,
    AlarmAcknowledged,
    AlarmResolved,
    ServiceCallAcknowledged,
    ServiceCallClosed,
}

impl AuditAction {
    pub fn as_str(self) -> &'static str {
        use AuditAction::*;
        match self {
            UserCreated => "user.created",
            UserUpdated => "user.updated",
            UserPasswordChanged => "user.password_changed",
            UserDeactivated => "user.deactivated",
            UserActivated => "user.activated",
            RoomCreated => "room.created",
            RoomUpdated => "room.updated",
            RoomDeactivated => "room.deactivated",
            RoomActivated => "room.activated",
            DeviceRegistered => "device.registered",
            DeviceAssigned => "device.assigned",
            DeviceKeyRotated => "device.key_rotated",
            DeviceDeactivated => "device.deactivated",
            ThresholdUpdated => "threshold.updated",
            StayCheckedIn => "stay.checked_in",
            StayCheckedOut => "stay.checked_out",
            AlarmAcknowledged => "alarm.acknowledged",
            AlarmResolved => "alarm.resolved",
            ServiceCallAcknowledged => "service_call.acknowledged",
            ServiceCallClosed => "service_call.closed",
        }
    }

    /// The part before the dot is stored separately so the log can be filtered per entity.
    pub fn entity_type(self) -> &'static str {
        self.as_str().split('.').next().unwrap_or("unknown")
    }
}
