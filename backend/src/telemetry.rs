//! Structured logging via `tracing`.
//!
//! Level is controlled by the RUST_LOG environment variable, e.g.
//!     RUST_LOG=info                       everything at info and above
//!     RUST_LOG=debug,tower_http=trace     see every request/response
//! Defaults to `info` when RUST_LOG is unset.

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
