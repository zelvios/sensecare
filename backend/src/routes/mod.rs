//! HTTP layer. Assembles the top-level router and cross-cutting middleware.
//!
//! GET  /health              liveness + DB round-trip (used by Docker's healthcheck)
//! GET  /swagger-ui          Swagger UI (API_DOCS=false to disable)
//! GET  /api-docs/openapi.json   the OpenAPI spec
//! GET  /api/v1              index
//! *    /api/v1/auth         login, logout, me                  -> routes::v1::auth
//! *    /api/v1/users        account management                 -> routes::v1::users
//! *    /api/v1/audit-log    audit log                          -> routes::v1::audit
//!
//! Middleware applied to everything:
//!   TraceLayer  one log line per request (method, path, status, latency)
//!   CorsLayer   permissive in development, locked to WEB_ORIGIN in production

pub mod docs;
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
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{config::Environment, state::AppState};

pub fn router(state: AppState) -> Router {
    let cors = cors_layer(&state);

    // Every route is registered together with its OpenAPI entry; split into
    // the axum router and the finished document.
    let (router, api) = OpenApiRouter::with_openapi(docs::ApiDoc::openapi())
        .merge(health::router())
        .nest("/api/v1", v1::router())
        .split_for_parts();

    let router = if state.config.api_docs {
        router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api))
    } else {
        router
    };

    router
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
