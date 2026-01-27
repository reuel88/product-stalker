use sea_orm_migration::prelude::*;

use super::m20240101_000001_create_products_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(
            m20240101_000001_create_products_table::Migration,
        )]
    }
}
