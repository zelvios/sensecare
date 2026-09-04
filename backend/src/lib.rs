//! Layering, top to bottom - each layer only calls the one below it:
//!   api       HTTP: routing, extractors, JSON in/out
//!   services  business rules (auth, alarms, audit)
//!   repos     Diesel queries: the only layer that imports `db::schema`
//!   models    row structs
//!   db        pool and migrations
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod repos;
pub mod routes;
pub mod services;
pub mod state;
pub mod telemetry;

use anyhow::Context;

use crate::{config::Config, state::AppState};

/// Builds the axum application without binding a socket.
///
/// Runs migrations first so the schema is guaranteed to exist before the pool hands out
/// connections. Integration tests call this and drive the router in-memory. `run` calls
/// it and serves it.
pub async fn build_app(config: Config) -> anyhow::Result<axum::Router> {
    db::run_migrations(&config.database_url)
        .await
        .context("database migration failed")?;
    tracing::info!("database migrations applied");

    let pool = db::create_pool(&config.database_url, config.db_pool_size)
        .context("could not create DB pool")?;
    let state = AppState::new(pool, config);
    Ok(routes::router(state))
}

/// Builds the app, binds `config.bind_addr` and serves until SIGINT/SIGTERM.
pub async fn run(config: Config) -> anyhow::Result<()> {
    let bind_addr = config.bind_addr.clone();
    let app = build_app(config).await?;

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("could not bind {bind_addr}"))?;
    tracing::info!("SenseCare API listening on {bind_addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// Resolves on Ctrl-C (all platforms) or SIGTERM (Unix - what Docker sends).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl-c");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received");
}
