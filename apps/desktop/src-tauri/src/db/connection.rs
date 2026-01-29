use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Manager};

use crate::error::AppError;
use crate::migrations::Migrator;
use sea_orm_migration::MigratorTrait;

pub fn get_db_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Database(sea_orm::DbErr::Custom(e.to_string())))?;

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| AppError::Database(sea_orm::DbErr::Custom(e.to_string())))?;

    Ok(app_data_dir.join("product_stalker.db"))
}

pub async fn init_db(app: &AppHandle) -> Result<DatabaseConnection, AppError> {
    let db_path = get_db_path(app)?;
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    log::info!("Initializing database at: {}", db_path.display());

    // Configure connection options for SQLite with WAL mode
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(5) // SQLite works best with small pool
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    let conn = Database::connect(opt).await?;

    // Enable WAL mode for better concurrency
    enable_wal_mode(&conn).await?;

    log::info!("Running migrations...");
    Migrator::up(&conn, None).await?;
    log::info!("Database initialized and migrations complete");

    Ok(conn)
}

async fn enable_wal_mode(conn: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm::{ConnectionTrait, Statement};

    // Enable WAL mode
    conn.execute(Statement::from_string(
        conn.get_database_backend(),
        "PRAGMA journal_mode=WAL;".to_owned(),
    ))
    .await?;

    // Set synchronous mode to NORMAL for better performance with WAL
    conn.execute(Statement::from_string(
        conn.get_database_backend(),
        "PRAGMA synchronous=NORMAL;".to_owned(),
    ))
    .await?;

    // Enable foreign keys
    conn.execute(Statement::from_string(
        conn.get_database_backend(),
        "PRAGMA foreign_keys=ON;".to_owned(),
    ))
    .await?;

    log::info!("SQLite configured with WAL mode");
    Ok(())
}
