use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
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

    // --- accounts ---
    /// create, edit, deactivate accounts with the `client` role
    ManageClients,
    /// create, edit, deactivate accounts with the `staff` or `admin` role
    ManageStaff,

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
