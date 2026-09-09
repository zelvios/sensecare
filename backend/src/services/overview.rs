//! The staff dashboard in one call (K5): every room with its latest reading,
//! occupant, device status and open call/alarm counts.

use std::collections::HashMap;

use bigdecimal::ToPrimitive;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    db::DbConn,
    error::ApiError,
    models::{alarm::AlarmKind, role::Permission, room::Room, service_call::ServiceCallStatus},
    repos::{self, rooms::RoomFilter},
    services::auth::AuthenticatedUser,
};

pub struct LatestReading {
    pub temperature_c: f64,
    pub humidity_pct: f64,
    pub measured_at: DateTime<Utc>,
}

pub struct Occupant {
    pub user_id: Uuid,
    pub display_name: String,
    pub checked_in_at: DateTime<Utc>,
}

pub struct DeviceStatus {
    pub id: Uuid,
    pub is_active: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
}

pub struct OpenCall {
    pub id: Uuid,
    pub status: ServiceCallStatus,
    pub created_at: DateTime<Utc>,
}

pub struct OpenAlarm {
    pub id: Uuid,
    pub kind: AlarmKind,
    pub raised_at: DateTime<Utc>,
    pub acknowledged: bool,
}

pub struct RoomOverview {
    pub room: Room,
    pub latest: Option<LatestReading>,
    pub occupant: Option<Occupant>,
    pub device: Option<DeviceStatus>,
    pub open_calls: Vec<OpenCall>,
    pub open_alarms: Vec<OpenAlarm>,
}

pub async fn rooms(
    conn: &mut DbConn,
    actor: &AuthenticatedUser,
    include_inactive: bool,
) -> Result<Vec<RoomOverview>, ApiError> {
    actor.require(Permission::ViewAllRooms)?;

    let rooms = repos::rooms::list(
        conn,
        &RoomFilter {
            q: None,
            active: (!include_inactive).then_some(true),
            floor: None,
            limit: 1000,
            offset: 0,
        },
    )
    .await?;

    let latest: HashMap<Uuid, LatestReading> = repos::overview::latest_per_room(conn)
        .await?
        .into_iter()
        .map(|(room_id, t, h, measured_at)| {
            (
                room_id,
                LatestReading {
                    temperature_c: t.to_f64().unwrap_or(f64::NAN),
                    humidity_pct: h.to_f64().unwrap_or(f64::NAN),
                    measured_at,
                },
            )
        })
        .collect();

    let occupants: HashMap<Uuid, Occupant> = repos::overview::occupants(conn)
        .await?
        .into_iter()
        .map(|(room_id, user_id, display_name, checked_in_at)| {
            (
                room_id,
                Occupant {
                    user_id,
                    display_name,
                    checked_in_at,
                },
            )
        })
        .collect();

    let devices: HashMap<Uuid, DeviceStatus> = repos::overview::assigned_devices(conn)
        .await?
        .into_iter()
        .map(|(room_id, id, is_active, last_seen_at)| {
            (
                room_id,
                DeviceStatus {
                    id,
                    is_active,
                    last_seen_at,
                },
            )
        })
        .collect();

    let mut calls: HashMap<Uuid, Vec<OpenCall>> = HashMap::new();
    for (room_id, id, status, created_at) in repos::overview::open_calls(conn).await? {
        let status = status.parse().map_err(|e: String| anyhow::anyhow!(e))?;
        calls.entry(room_id).or_default().push(OpenCall {
            id,
            status,
            created_at,
        });
    }

    let mut alarms: HashMap<Uuid, Vec<OpenAlarm>> = HashMap::new();
    for (room_id, id, kind, raised_at, acknowledged) in repos::overview::open_alarms(conn).await? {
        let kind = kind.parse().map_err(|e: String| anyhow::anyhow!(e))?;
        alarms.entry(room_id).or_default().push(OpenAlarm {
            id,
            kind,
            raised_at,
            acknowledged,
        });
    }

    let mut latest = latest;
    let mut occupants = occupants;
    let mut devices = devices;

    Ok(rooms
        .into_iter()
        .map(|room| RoomOverview {
            latest: latest.remove(&room.id),
            occupant: occupants.remove(&room.id),
            device: devices.remove(&room.id),
            open_calls: calls.remove(&room.id).unwrap_or_default(),
            open_alarms: alarms.remove(&room.id).unwrap_or_default(),
            room,
        })
        .collect())
}
