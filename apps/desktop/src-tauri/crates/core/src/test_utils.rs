//! Shared test utilities for database setup and test data creation.
//!
//! This module is only compiled in test mode.

use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Schema};

use crate::entities::app_setting::Entity as AppSettingEntity;

/// Creates a bare in-memory SQLite test database with no tables
pub async fn setup_in_memory_db() -> DatabaseConnection {
    Database::connect("sqlite::memory:").await.unwrap()
}

/// Creates an in-memory SQLite test database with app_settings table only (EAV model)
pub async fn setup_app_settings_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.unwrap();
    let schema = Schema::new(DatabaseBackend::Sqlite);
    let stmt = schema.create_table_from_entity(AppSettingEntity);
    conn.execute(conn.get_database_backend().build(&stmt))
        .await
        .unwrap();
    conn
}
