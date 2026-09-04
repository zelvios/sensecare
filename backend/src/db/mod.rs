//! Database connectivity: the connection pool and embedded migrations.
//!
//! Uses diesel-async (tokio-postgres underneath), so no libpq is linked and queries are awaited
//! instead of blocking a thread.

pub mod schema;

use diesel::Connection;
use diesel_async::{
    AsyncPgConnection,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

/// The shared pool stored in `AppState`. Cheap to clone (it's an Arc inside).
pub type DbPool = Pool<AsyncPgConnection>;

/// A connection borrowed from the pool. Returned automatically when dropped.
/// Repos take `&mut DbConn`.
pub type DbConn = diesel_async::pooled_connection::deadpool::Object<AsyncPgConnection>;

/// Every folder under ./migrations, compiled into the binary at build time.
/// This is why the Docker image doesn't need diesel_cli or the migrations dir.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn create_pool(database_url: &str, max_size: usize) -> anyhow::Result<DbPool> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Ok(Pool::builder(manager).max_size(max_size).build()?)
}

/// Applies any migration not yet recorded in `__diesel_schema_migrations`.
///
/// Diesel's migration harness is synchronous, so this opens one dedicated connection, wraps it
/// so the sync harness can drive it and runs on a blocking thread. It happens once which is before
/// the pool exists.
pub async fn run_migrations(database_url: &str) -> anyhow::Result<()> {
    let url = database_url.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut conn = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&url)?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok::<(), anyhow::Error>(())
    })
    .await??; // outer ?: the blocking task panicked. inner ?: a migration failed
    Ok(())
}
