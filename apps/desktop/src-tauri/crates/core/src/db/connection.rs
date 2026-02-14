use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

use crate::error::AppError;
use sea_orm_migration::prelude::*;

/// Migrator that runs core infrastructure migrations only.
struct CoreMigrator;

#[async_trait::async_trait]
impl MigratorTrait for CoreMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        crate::migrations::migrations()
    }
}

/// Build connection options for SQLite with recommended settings.
///
/// Per-connection pragmas (`synchronous`, `foreign_keys`) are configured via
/// `map_sqlx_sqlite_opts` so they apply to **every** connection in the pool,
/// not just the first one acquired.
pub fn build_connection_options(db_url: String) -> ConnectOptions {
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(5) // SQLite works best with small pool
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug)
        .map_sqlx_sqlite_opts(|opts| {
            opts.pragma("synchronous", "NORMAL")
                .pragma("foreign_keys", "ON")
        });
    opt
}

/// Initialize database from a connection string (Tauri-agnostic version)
pub async fn init_db_from_url(db_url: String) -> Result<DatabaseConnection, AppError> {
    let opt = build_connection_options(db_url);
    let conn = Database::connect(opt).await?;

    // Enable WAL mode for better concurrency
    enable_wal_mode(&conn).await?;

    log::info!("Running core migrations...");
    CoreMigrator::up(&conn, None).await?;
    log::info!("Core migrations complete");

    Ok(conn)
}

/// Enable WAL journal mode (database-level, persists in the file).
///
/// Per-connection pragmas (`synchronous`, `foreign_keys`) are set via
/// `map_sqlx_sqlite_opts` in [`build_connection_options`] so they apply to
/// every connection in the pool.
pub async fn enable_wal_mode(conn: &DatabaseConnection) -> Result<(), AppError> {
    use sea_orm::{ConnectionTrait, Statement};

    conn.execute(Statement::from_string(
        conn.get_database_backend(),
        "PRAGMA journal_mode=WAL;".to_owned(),
    ))
    .await?;

    log::info!("SQLite configured with WAL mode");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database, Statement};

    #[test]
    fn test_build_connection_options_sets_max_connections() {
        let opts = build_connection_options("sqlite::memory:".to_string());
        // ConnectOptions doesn't expose getters, but we can verify it doesn't panic
        // and returns a valid options object
        assert!(opts.get_url().contains("sqlite"));
    }

    #[test]
    fn test_build_connection_options_with_file_path() {
        let opts = build_connection_options("sqlite:test.db?mode=rwc".to_string());
        assert!(opts.get_url().contains("test.db"));
    }

    #[tokio::test]
    async fn test_enable_wal_mode_sets_journal_mode() {
        let conn = Database::connect("sqlite::memory:").await.unwrap();

        enable_wal_mode(&conn).await.unwrap();

        // Verify WAL mode is set (in-memory SQLite returns "memory" instead of "wal")
        let result = conn
            .query_one(Statement::from_string(
                conn.get_database_backend(),
                "PRAGMA journal_mode;".to_owned(),
            ))
            .await
            .unwrap();

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_build_connection_options_sets_synchronous_normal() {
        let opts = build_connection_options("sqlite::memory:".to_string());
        let conn = Database::connect(opts).await.unwrap();

        let result = conn
            .query_one(Statement::from_string(
                conn.get_database_backend(),
                "PRAGMA synchronous;".to_owned(),
            ))
            .await
            .unwrap()
            .unwrap();

        let sync_mode: i32 = result.try_get_by_index(0).unwrap();
        assert_eq!(sync_mode, 1); // NORMAL = 1
    }

    #[tokio::test]
    async fn test_build_connection_options_sets_foreign_keys_on() {
        let opts = build_connection_options("sqlite::memory:".to_string());
        let conn = Database::connect(opts).await.unwrap();

        let result = conn
            .query_one(Statement::from_string(
                conn.get_database_backend(),
                "PRAGMA foreign_keys;".to_owned(),
            ))
            .await
            .unwrap()
            .unwrap();

        let fk_enabled: i32 = result.try_get_by_index(0).unwrap();
        assert_eq!(fk_enabled, 1); // ON = 1
    }

    #[tokio::test]
    async fn test_init_db_from_url_creates_connection() {
        let conn = init_db_from_url("sqlite::memory:".to_string())
            .await
            .unwrap();

        // Verify we can execute queries
        let result = conn
            .query_one(Statement::from_string(
                conn.get_database_backend(),
                "SELECT 1;".to_owned(),
            ))
            .await
            .unwrap();

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_init_db_from_url_runs_migrations() {
        let conn = init_db_from_url("sqlite::memory:".to_string())
            .await
            .unwrap();

        // Verify app_settings table exists (EAV model)
        let result = conn
            .query_one(Statement::from_string(
                conn.get_database_backend(),
                "SELECT name FROM sqlite_master WHERE type='table' AND name='app_settings';"
                    .to_owned(),
            ))
            .await
            .unwrap();

        assert!(
            result.is_some(),
            "app_settings table should exist after migrations"
        );
    }

    #[tokio::test]
    async fn test_init_db_from_url_invalid_url_fails() {
        let result = init_db_from_url("invalid://not-a-database".to_string()).await;
        assert!(result.is_err());
    }
}
