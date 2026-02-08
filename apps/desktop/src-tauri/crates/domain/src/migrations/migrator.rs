use sea_orm_migration::prelude::*;

use super::m20240101_000001_create_products_table;
use super::m20240103_000001_create_availability_checks_table;
use super::m20250205_000001_add_price_tracking;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20240101_000001_create_products_table::Migration),
        Box::new(m20240103_000001_create_availability_checks_table::Migration),
        Box::new(m20250205_000001_add_price_tracking::Migration),
    ]
}
