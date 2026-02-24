//! Shared test utilities for database setup and test data creation.
//!
//! This module is only compiled in test mode.

use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Schema};

use crate::entities::app_setting::Entity as AppSettingEntity;
use crate::entities::exchange_rate::Entity as ExchangeRateEntity;

/// Creates a bare in-memory SQLite test database with no tables
pub async fn setup_in_memory_db() -> DatabaseConnection {
    Database::connect("sqlite::memory:").await.unwrap()
}

/// Creates an in-memory SQLite test database with core tables (app_settings + exchange_rates)
pub async fn setup_app_settings_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.unwrap();
    let schema = Schema::new(DatabaseBackend::Sqlite);

    let stmt = schema.create_table_from_entity(AppSettingEntity);
    conn.execute(conn.get_database_backend().build(&stmt))
        .await
        .unwrap();

    let stmt = schema.create_table_from_entity(ExchangeRateEntity);
    conn.execute(conn.get_database_backend().build(&stmt))
        .await
        .unwrap();

    // Create the unique index needed for upsert operations
    conn.execute_unprepared(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_exchange_rates_currency_pair ON exchange_rates (from_currency, to_currency)",
    )
    .await
    .unwrap();

    conn
}
