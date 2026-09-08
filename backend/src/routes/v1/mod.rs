//! Versioned API. Feature routers get nested here as they are built:
//!   auth          – K3, K11
//!   rooms         – K5, K7
//!   measurements  – K4, K6
//!   service_calls – K2
//!   devices       – K8, K9, K10

mod audit;
mod auth;
mod rooms;
mod users;

use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::state::AppState;

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(index))
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/audit-log", audit::router())
        .nest("/rooms", rooms::router())
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
