//! Runtime configuration, read once at startup from environment variables.
//!
//! Locally these come from `backend/.env` (via dotenvy). In Docker they come from the
//! `environment:` block in docker-compose.yml. Required variables fail with a clear message
//! while optional ones have defaults.
//!
//! `APP_ENV` selects the mode. In production, settings that would be unsafe to leave at
//! their development defaults are required, so a misconfigured deploy refuses to start
//! instead of running open.

use anyhow::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Production,
}

impl std::str::FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "development" | "dev" => Ok(Environment::Development),
            "production" | "prod" => Ok(Environment::Production),
            other => Err(format!(
                "APP_ENV must be 'development' or 'production', got '{other}'"
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    /// development or production. Drives CORS, cookie flags and validation.
    pub environment: Environment,
    /// postgres://user:password@host:port/database
    pub database_url: String,
    /// Socket the API listens on. 0.0.0.0 inside Docker so the port mapping works.
    pub bind_addr: String,
    /// Max simultaneous DB connections.
    pub db_pool_size: usize,
    /// Exact origin of the web frontend. Used for CORS. Required in production.
    pub web_origin: Option<String>,
    /// How long a session stays valid without activity.
    pub session_ttl_hours: i64,
    /// Secure flag on the session cookie. Always true in production.
    pub cookie_secure: bool,
    /// If set and the users table is empty, an admin account is created on startup.
    pub bootstrap_admin_password: Option<String>,
    /// Serve Swagger UI at /swagger-ui. Default on. Set API_DOCS=false to hide it.
    pub api_docs: bool,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let environment: Environment = std::env::var("APP_ENV")
            .unwrap_or_else(|_| "development".into())
            .parse()
            .map_err(|e: String| anyhow::anyhow!(e))?;

        let web_origin = std::env::var("WEB_ORIGIN").ok().filter(|s| !s.is_empty());
        let bootstrap_admin_password = std::env::var("BOOTSTRAP_ADMIN_PASSWORD")
            .ok()
            .filter(|s| !s.is_empty());

        if environment == Environment::Production {
            anyhow::ensure!(
                web_origin.is_some(),
                "WEB_ORIGIN is required when APP_ENV=production"
            );
            if let Some(p) = &bootstrap_admin_password {
                anyhow::ensure!(
                    p.len() >= 8,
                    "BOOTSTRAP_ADMIN_PASSWORD must be at least 8 characters in production"
                );
            }
        }

        Ok(Self {
            environment,
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL is not set")?,
            bind_addr: std::env::var("API_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            db_pool_size: env_or("DB_POOL_SIZE", 10),
            web_origin,
            session_ttl_hours: env_or("SESSION_TTL_HOURS", 12),
            // Production always sets Secure. Development can also for testing behind a local proxy.
            cookie_secure: environment == Environment::Production || env_or("COOKIE_SECURE", false),
            bootstrap_admin_password,
            api_docs: env_or("API_DOCS", true),
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
}

/// Reads and parses an environment variable, falling back to `default` when unset or invalid.
fn env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
