use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Client,
    Staff,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    // --- rooms & climate ---
    /// a client's own room, stay and measurements
    ViewOwnRoom,
    /// overview of all rooms and their history
    ViewAllRooms,
    /// create, edit, deactivate rooms
    ManageRooms,
    /// global and per-room climate limits
    ManageThresholds,
    /// permanently delete a room that was created by mistake (`admin` only)
    DeleteRooms,

    // --- operations ---
    /// acknowledge and close service calls
    HandleServiceCalls,
    /// acknowledge and resolve climate alarms
    HandleAlarms,
    /// check clients in to and out of rooms
    ManageStays,

    // --- devices ---
    /// see device status (last seen, firmware) per room
    ViewDevices,
    /// register devices, assign to rooms, rotate keys
    ManageDevices,
    /// permanently delete a device that was registered by mistake (`admin` only)
    DeleteDevices,

    // --- accounts ---
    /// create, edit, deactivate accounts with the `client` role
    ManageClients,
    /// create, edit, deactivate accounts with the `staff` or `admin` role
    ManageStaff,
    /// permanently delete an account that was created by mistake (`admin` only)
    DeleteUsers,

    // --- governance ---
    /// read the audit log
    ViewAuditLog,
}

impl Role {
    pub fn has(self, permission: Permission) -> bool {
        use Permission::*;
        match self {
            Role::Client => matches!(permission, ViewOwnRoom),
            Role::Staff => matches!(
                permission,
                ViewOwnRoom
                    | ViewAllRooms
                    | HandleServiceCalls
                    | HandleAlarms
                    | ManageStays
                    | ViewDevices
                    | ManageClients
            ),
            Role::Admin => true,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Role::Client => "client",
            Role::Staff => "staff",
            Role::Admin => "admin",
        }
    }
}

impl Permission {
    /// The permission needed to create, edit or deactivate an account with `target` role.
    pub fn for_managing(target: Role) -> Permission {
        match target {
            Role::Client => Permission::ManageClients,
            Role::Staff | Role::Admin => Permission::ManageStaff,
        }
    }
}

impl std::str::FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "client" => Ok(Role::Client),
            "staff" => Ok(Role::Staff),
            "admin" => Ok(Role::Admin),
            other => Err(format!("unknown role '{other}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_only_sees_own_room() {
        assert!(Role::Client.has(Permission::ViewOwnRoom));
        assert!(!Role::Client.has(Permission::ViewAllRooms));
        assert!(!Role::Client.has(Permission::HandleServiceCalls));
    }

    #[test]
    fn clients_cannot_manage_any_account() {
        assert!(!Role::Client.has(Permission::ManageClients));
        assert!(!Role::Client.has(Permission::ManageStaff));
        assert!(!Role::Client.has(Permission::DeleteUsers));
    }

    #[test]
    fn staff_manages_clients_but_not_staff() {
        assert!(Role::Staff.has(Permission::ManageClients));
        assert!(!Role::Staff.has(Permission::ManageStaff));
    }

    #[test]
    fn staff_operates_but_does_not_administer() {
        assert!(Role::Staff.has(Permission::ViewAllRooms));
        assert!(Role::Staff.has(Permission::HandleServiceCalls));
        assert!(Role::Staff.has(Permission::HandleAlarms));
        assert!(Role::Staff.has(Permission::ManageStays));
        assert!(Role::Staff.has(Permission::ViewDevices));
        assert!(!Role::Staff.has(Permission::ManageRooms));
        assert!(!Role::Staff.has(Permission::ManageDevices));
        assert!(!Role::Staff.has(Permission::ManageThresholds));
        assert!(!Role::Staff.has(Permission::ViewAuditLog));
    }

    #[test]
    fn only_admin_hard_deletes() {
        assert!(!Role::Client.has(Permission::DeleteUsers));
        assert!(!Role::Client.has(Permission::DeleteRooms));
        assert!(!Role::Staff.has(Permission::DeleteUsers));
        assert!(!Role::Staff.has(Permission::DeleteRooms));
        assert!(!Role::Staff.has(Permission::DeleteDevices));
        assert!(Role::Admin.has(Permission::DeleteUsers));
        assert!(Role::Admin.has(Permission::DeleteRooms));
        assert!(Role::Admin.has(Permission::DeleteDevices));
    }

    #[test]
    fn admin_has_everything() {
        for p in [
            Permission::ViewOwnRoom,
            Permission::ManageStaff,
            Permission::ManageThresholds,
            Permission::ViewAuditLog,
            Permission::DeleteRooms,
        ] {
            assert!(Role::Admin.has(p), "admin should have {p:?}");
        }
    }

    #[test]
    fn managing_an_account_maps_to_the_target_role() {
        assert_eq!(
            Permission::for_managing(Role::Client),
            Permission::ManageClients
        );
        assert_eq!(
            Permission::for_managing(Role::Staff),
            Permission::ManageStaff
        );
        assert_eq!(
            Permission::for_managing(Role::Admin),
            Permission::ManageStaff
        );
    }

    #[test]
    fn role_names_round_trip() {
        for r in [Role::Client, Role::Staff, Role::Admin] {
            assert_eq!(r.as_str().parse::<Role>().unwrap(), r);
        }
        assert!("nurse".parse::<Role>().is_err());
    }
}
