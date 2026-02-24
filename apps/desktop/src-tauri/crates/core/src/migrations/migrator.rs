use sea_orm_migration::prelude::*;

use super::m20240102_000001_create_settings_table;
use super::m20240104_000001_add_background_check_settings;
use super::m20240105_000001_add_headless_browser_setting;
use super::m20250206_000001_create_app_settings_table;
use super::m20250207_000001_backfill_app_settings;
use super::m20250208_000001_drop_old_settings_table;
use super::m20250214_000001_create_verified_sessions;
use super::m20260216_000001_create_exchange_rates_table;
use super::m20260216_000002_rename_tables_to_plural;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20240102_000001_create_settings_table::Migration),
        Box::new(m20240104_000001_add_background_check_settings::Migration),
        Box::new(m20240105_000001_add_headless_browser_setting::Migration),
        Box::new(m20250206_000001_create_app_settings_table::Migration),
        Box::new(m20250207_000001_backfill_app_settings::Migration),
        Box::new(m20250208_000001_drop_old_settings_table::Migration),
        Box::new(m20250214_000001_create_verified_sessions::Migration),
        Box::new(m20260216_000001_create_exchange_rates_table::Migration),
        Box::new(m20260216_000002_rename_tables_to_plural::Migration),
    ]
}
