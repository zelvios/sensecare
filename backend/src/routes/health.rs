//! GET /health
//!
//! Proves three things at once: the process is up, it can borrow a pooled connection, and
//! Postgres answers. Docker's healthcheck polls this: `web` won't start until it passes.

use axum::{Json, extract::State};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{error::ApiError, state::AppState};

#[derive(Serialize, ToSchema)]
pub struct Health {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    database: &'static str,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(health))
}

/// Liveness check with a database round-trip.
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses((status = 200, body = Health), (status = 500, body = crate::error::ErrorResponse))
)]
async fn health(State(state): State<AppState>) -> Result<Json<Health>, ApiError> {
    let mut conn = state.pool.get().await?;
    diesel::sql_query("SELECT 1").execute(&mut conn).await?;

    Ok(Json(Health {
        status: "ok",
        service: "sensecare-api",
        version: env!("CARGO_PKG_VERSION"),
        database: "connected",
    }))
}
