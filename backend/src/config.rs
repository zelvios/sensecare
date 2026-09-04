//! Runtime configuration, read once at startup from environment variables.
//!
//! Locally these come from `backend/.env` (via dotenvy). In Docker they from the `environment:`
//! block in docker-compose.yml. Required variables fail fast with a clear message while
//! optional ones have defaults.

use anyhow::Context;

#[derive(Debug, Clone)]
pub struct Config {
    /// postgres://user:password@host:port/database
    pub database_url: String,
    /// Socket the API listens on. 0.0.0.0 inside Docker so the port mapping works.
    pub bind_addr: String,
    /// Max simultaneous DB connections.
    pub db_pool_size: usize,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL is not set")?,
            bind_addr: std::env::var("API_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            db_pool_size: std::env::var("DB_POOL_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        })
    }
}
