//!   auth          login, logout, me                               K3, K11
//!   users         account management                              K7
//!   rooms         room management + measurement history           K5, K6, K7
//!   devices       registration, key auth, measurement recording   K8, K9, K10
//!   stays         check-in / check-out, own room                  K4, K11
//!   audit-log     audit log
//!   measurements  DTOs only, handlers live in rooms and devices
//! Planned: service_calls (K2), alarms

mod audit;
mod auth;
mod devices;
mod measurements;
mod rooms;
mod service_calls;
mod stays;
mod users;

use crate::state::AppState;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(index))
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/audit-log", audit::router())
        .nest("/rooms", rooms::router())
        .nest("/devices", devices::router())
        .nest("/stays", stays::router())
        .nest("/service-calls", service_calls::router())
}

#[derive(Serialize, ToSchema)]
pub struct ApiIndex {
    name: &'static str,
    version: &'static str,
}

/// API index.
#[utoipa::path(get, path = "/", tag = "health", operation_id = "api_index",
    responses((status = 200, body = ApiIndex)))]
async fn index() -> Json<ApiIndex> {
    Json(ApiIndex {
        name: "sensecare-api",
        version: env!("CARGO_PKG_VERSION"),
    })
}
