use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::schema::audit_log;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, ToSchema)]
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
    UserDeleted,
    UserLoggedIn,
    UserLoggedOut,
    UserLoginFailed,
    UserLoginRefused,
    RoomCreated,
    RoomUpdated,
    RoomDeactivated,
    RoomActivated,
    RoomDeleted,
    DeviceRegistered,
    DeviceAssigned,
    DeviceKeyRotated,
    DeviceUpdated,
    DeviceActivated,
    DeviceDeactivated,
    DeviceDeleted,
    ThresholdUpdated,
    StayCheckedIn,
    StayCheckedOut,
    AlarmAcknowledged,
    AlarmResolved,
    ServiceCallAcknowledged,
    ServiceCallClosed,
    ServiceCallCreated,
    ServiceCallUpdated,
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
            UserDeleted => "user.deleted",
            UserLoggedIn => "user.logged_in",
            UserLoggedOut => "user.logged_out",
            UserLoginFailed => "user.login_failed",
            UserLoginRefused => "user.login_refused",
            RoomCreated => "room.created",
            RoomUpdated => "room.updated",
            RoomDeactivated => "room.deactivated",
            RoomActivated => "room.activated",
            RoomDeleted => "room.deleted",
            DeviceRegistered => "device.registered",
            DeviceAssigned => "device.assigned",
            DeviceKeyRotated => "device.key_rotated",
            DeviceUpdated => "device.updated",
            DeviceActivated => "device.activated",
            DeviceDeactivated => "device.deactivated",
            DeviceDeleted => "device.deleted",
            ThresholdUpdated => "threshold.updated",
            StayCheckedIn => "stay.checked_in",
            StayCheckedOut => "stay.checked_out",
            AlarmAcknowledged => "alarm.acknowledged",
            AlarmResolved => "alarm.resolved",
            ServiceCallAcknowledged => "service_call.acknowledged",
            ServiceCallClosed => "service_call.closed",
            ServiceCallCreated => "service_call.created",
            ServiceCallUpdated => "service_call.updated",
        }
    }

    /// The part before the dot is stored separately so the log can be filtered per entity.
    pub fn entity_type(self) -> &'static str {
        self.as_str().split('.').next().unwrap_or("unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_strings_follow_entity_dot_verb() {
        assert_eq!(AuditAction::UserDeactivated.as_str(), "user.deactivated");
        assert_eq!(
            AuditAction::ServiceCallClosed.as_str(),
            "service_call.closed"
        );
    }

    #[test]
    fn entity_type_is_the_part_before_the_dot() {
        assert_eq!(AuditAction::UserCreated.entity_type(), "user");
        assert_eq!(AuditAction::RoomDeleted.entity_type(), "room");
        assert_eq!(
            AuditAction::ServiceCallAcknowledged.entity_type(),
            "service_call"
        );
    }
}
