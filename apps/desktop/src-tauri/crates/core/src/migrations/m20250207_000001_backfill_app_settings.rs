use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Check if old settings table exists and has data
        let has_old_settings = db
            .query_one(Statement::from_string(
                manager.get_database_backend(),
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='settings'".to_string(),
            ))
            .await?
            .is_some();

        if !has_old_settings {
            // No old settings table, nothing to migrate
            return Ok(());
        }

        // Check if there's data in the old settings table
        let old_settings = db
            .query_one(Statement::from_string(
                manager.get_database_backend(),
                "SELECT theme, show_in_tray, launch_at_login, enable_logging, log_level, enable_notifications, sidebar_expanded, background_check_enabled, background_check_interval_minutes, enable_headless_browser FROM settings WHERE id = 1".to_string(),
            ))
            .await?;

        if let Some(row) = old_settings {
            // Extract values from old settings
            let theme: String = row
                .try_get("", "theme")
                .unwrap_or_else(|_| "system".to_string());
            let show_in_tray: bool = row.try_get("", "show_in_tray").unwrap_or(true);
            let launch_at_login: bool = row.try_get("", "launch_at_login").unwrap_or(false);
            let enable_logging: bool = row.try_get("", "enable_logging").unwrap_or(true);
            let log_level: String = row
                .try_get("", "log_level")
                .unwrap_or_else(|_| "info".to_string());
            let enable_notifications: bool =
                row.try_get("", "enable_notifications").unwrap_or(true);
            let sidebar_expanded: bool = row.try_get("", "sidebar_expanded").unwrap_or(true);
            let background_check_enabled: bool =
                row.try_get("", "background_check_enabled").unwrap_or(false);
            let background_check_interval_minutes: i32 = row
                .try_get("", "background_check_interval_minutes")
                .unwrap_or(60);
            let enable_headless_browser: bool =
                row.try_get("", "enable_headless_browser").unwrap_or(true);

            let now = chrono::Utc::now().to_rfc3339();

            // Insert each setting into the EAV table
            // Using JSON format for values (strings quoted, bools/ints as-is)
            let settings = vec![
                ("theme", format!("\"{}\"", theme)),
                ("show_in_tray", show_in_tray.to_string()),
                ("launch_at_login", launch_at_login.to_string()),
                ("enable_logging", enable_logging.to_string()),
                ("log_level", format!("\"{}\"", log_level)),
                ("enable_notifications", enable_notifications.to_string()),
                ("sidebar_expanded", sidebar_expanded.to_string()),
                (
                    "background_check_enabled",
                    background_check_enabled.to_string(),
                ),
                (
                    "background_check_interval_minutes",
                    background_check_interval_minutes.to_string(),
                ),
                (
                    "enable_headless_browser",
                    enable_headless_browser.to_string(),
                ),
            ];

            for (key, value) in settings {
                db.execute(Statement::from_string(
                    manager.get_database_backend(),
                    format!(
                        "INSERT OR REPLACE INTO app_settings (scope_type, scope_id, key, value, updated_at) VALUES ('global', NULL, '{}', '{}', '{}')",
                        key, value, now
                    ),
                ))
                .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Delete all migrated settings (only global scope settings)
        let db = manager.get_connection();
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            "DELETE FROM app_settings WHERE scope_type = 'global'".to_string(),
        ))
        .await?;
        Ok(())
    }
}
