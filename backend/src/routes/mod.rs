//! GET  /health         liveness + DB round trip (used by Docker's healthcheck)
//! GET  /api/v1         index
//! *    /api/v1/auth    login, logout, me                       -> routes::v1::auth
//!
//! Middleware applied to everything:
//!   TraceLayer  one log line per request (method, path, status, latency)
//!   CorsLayer   permissive in development, locked to WEB_ORIGIN in production

pub mod extractors;
pub mod health;
pub mod v1;

use axum::{
    Router,
    http::{HeaderValue, Method, header},
};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::{config::Environment, state::AppState};

pub fn router(state: AppState) -> Router {
    let cors = cors_layer(&state);

    Router::new()
        .merge(health::router())
        .nest("/api/v1", v1::router())
        // One log line per request with method, path, status and latency.
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// Development: anything goes.
/// Production: only the configured web origin, with credentials allowed for the cookie.
fn cors_layer(state: &AppState) -> CorsLayer {
    match state.config.environment {
        Environment::Development => CorsLayer::permissive(),
        Environment::Production => {
            let origin = state
                .config
                .web_origin
                .as_deref()
                .expect("validated in Config::from_env")
                .parse::<HeaderValue>()
                .expect("WEB_ORIGIN is not a valid header value");
            CorsLayer::new()
                .allow_origin(AllowOrigin::exact(origin))
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
                .allow_credentials(true)
        }
    }
}
