//! Versioned API. Feature routers get nested here as they are built:
//!   auth          – K3, K11
//!   rooms         – K5, K7
//!   measurements  – K4, K6
//!   service_calls – K2
//!   devices       – K8, K9, K10

mod auth;
mod audit;

use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .nest("/auth", auth::router())
        .nest("/audit-log", audit::router())
}

async fn index() -> Json<Value> {
    Json(json!({
        "name": "sensecare-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
