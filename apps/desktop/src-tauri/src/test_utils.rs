//! Shared test utilities for database setup and test data creation.
//!
//! This module is only compiled in test mode.

use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Schema};
use uuid::Uuid;

use crate::domain::entities::availability_check::Entity as AvailabilityCheckEntity;
use crate::domain::entities::product::Entity as ProductEntity;
use crate::domain::repositories::{CreateProductRepoParams, ProductRepository};
use product_stalker_core::entities::app_setting::Entity as AppSettingEntity;

/// Creates an in-memory SQLite test database with products table only
pub async fn setup_products_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.unwrap();
    let schema = Schema::new(DatabaseBackend::Sqlite);
    let stmt = schema.create_table_from_entity(ProductEntity);
    conn.execute(conn.get_database_backend().build(&stmt))
        .await
        .unwrap();
    conn
}

/// Creates an in-memory SQLite test database with products and availability_checks tables
pub async fn setup_availability_db() -> DatabaseConnection {
    let conn = Database::connect("sqlite::memory:").await.unwrap();
    let schema = Schema::new(DatabaseBackend::Sqlite);

    // Create products table first (for foreign key)
    let stmt = schema.create_table_from_entity(ProductEntity);
    conn.execute(conn.get_database_backend().build(&stmt))
        .await
        .unwrap();

    // Create availability_checks table
    let stmt = schema.create_table_from_entity(AvailabilityCheckEntity);
    conn.execute(conn.get_database_backend().build(&stmt))
        .await
        .unwrap();

    conn
}

/// Creates a test product with the given URL
pub async fn create_test_product(conn: &DatabaseConnection, url: &str) -> Uuid {
    let id = Uuid::new_v4();
    ProductRepository::create(
        conn,
        id,
        CreateProductRepoParams {
            name: "Test Product".to_string(),
            url: url.to_string(),
            description: None,
            notes: None,
        },
    )
    .await
    .unwrap();
    id
}

/// Creates a test product with default URL (https://example.com/product)
pub async fn create_test_product_default(conn: &DatabaseConnection) -> Uuid {
    create_test_product(conn, "https://example.com/product").await
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
