use sea_orm_migration::prelude::*;

use super::m20240101_000001_create_products_table;
use super::m20240102_000001_create_settings_table;
use super::m20240103_000001_create_availability_checks_table;
use super::m20240104_000001_add_background_check_settings;
use super::m20240105_000001_add_headless_browser_setting;
use super::m20250205_000001_add_price_tracking;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_products_table::Migration),
            Box::new(m20240102_000001_create_settings_table::Migration),
            Box::new(m20240103_000001_create_availability_checks_table::Migration),
            Box::new(m20240104_000001_add_background_check_settings::Migration),
            Box::new(m20240105_000001_add_headless_browser_setting::Migration),
            Box::new(m20250205_000001_add_price_tracking::Migration),
        ]
    }
}
