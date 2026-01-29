use sea_orm_migration::prelude::*;

use super::m20240101_000001_create_products_table;
use super::m20240102_000001_create_settings_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_products_table::Migration),
            Box::new(m20240102_000001_create_settings_table::Migration),
        ]
    }
}
