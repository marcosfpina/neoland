// NEOLAND Persistent Storage Module
// Phase 4.7: Performance Optimization

pub mod vector_store;

pub use vector_store::{PersistentVectorStore, StoredDocument};

/// Applies the SQL migrations embedded from `migrations/` to the database.
///
/// Single migration mechanism for the project: used by `neoland migrate` and,
/// when `NEOLAND_AUTO_MIGRATE` is set, by the server on startup. Databases
/// migrated by hand (without the `_sqlx_migrations` table) must not enable
/// auto-migrate — sqlx would try to re-apply migration 001.
pub async fn run_migrations(database_url: &str) -> anyhow::Result<()> {
    use sqlx::postgres::PgPoolOptions;
    let pool = PgPoolOptions::new().max_connections(1).connect(database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(())
}
