use sea_orm_migration::prelude::*;

use super::m20240101_000001_create_products_table;
use super::m20240103_000001_create_availability_checks_table;
use super::m20250205_000001_add_price_tracking;
use super::m20260211_000001_add_product_currency;
use super::m20260212_000001_rename_price_cents_to_price_minor_units;
use super::m20260213_000001_add_multi_retailer;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20240101_000001_create_products_table::Migration),
        Box::new(m20240103_000001_create_availability_checks_table::Migration),
        Box::new(m20250205_000001_add_price_tracking::Migration),
        Box::new(m20260211_000001_add_product_currency::Migration),
        Box::new(m20260212_000001_rename_price_cents_to_price_minor_units::Migration),
        Box::new(m20260213_000001_add_multi_retailer::Migration),
    ]
}
