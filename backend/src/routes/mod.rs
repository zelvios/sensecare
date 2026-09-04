//! HTTP layer. Assembles the top-level router and cross-cutting middleware.
//!
//! Layout:
//!   /health        liveness + DB check (used by Docker's healthcheck)
//!   /api/v1/...    the versioned API. Feature routers nest in `v1`

pub mod health;
pub mod v1;

use axum::Router;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(health::router())
        .nest("/api/v1", v1::router())
        // One log line per request with method, path, status and latency.
        .layer(TraceLayer::new_for_http())
        // Used for development. Before production restrict to WEB_ORIGIN or drop CORS entirely 
        // once web and api sit behind the same reverse proxy.
        .layer(CorsLayer::permissive())
        .with_state(state)
}