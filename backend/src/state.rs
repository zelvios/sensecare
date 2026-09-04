//! State shared by all handlers. axum clones this per request, so everything
//! inside must be cheap to clone: the pool is an Arc, the config is wrapped in one.

use std::sync::Arc;

use crate::{config::Config, db::DbPool};

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(pool: DbPool, config: Config) -> Self {
        Self {
            pool,
            config: Arc::new(config),
        }
    }
}
