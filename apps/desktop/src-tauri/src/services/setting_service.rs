use sea_orm::DatabaseConnection;

use crate::entities::setting::Model as SettingModel;
use crate::error::AppError;
use crate::repositories::{SettingRepository, UpdateSettingsParams};

/// Service layer for settings business logic
///
/// Validates inputs and orchestrates repository calls.
pub struct SettingService;

impl SettingService {
    /// Get current settings (creates defaults if first run)
    pub async fn get(conn: &DatabaseConnection) -> Result<SettingModel, AppError> {
        SettingRepository::get_or_create(conn).await
    }

    /// Update settings with validation
    pub async fn update(
        conn: &DatabaseConnection,
        params: UpdateSettingsParams,
    ) -> Result<SettingModel, AppError> {
        // Validate theme if provided
        if let Some(ref theme) = params.theme {
            Self::validate_theme(theme)?;
        }

        // Validate log level if provided
        if let Some(ref level) = params.log_level {
            Self::validate_log_level(level)?;
        }

        // Get existing settings
        let settings = SettingRepository::get_or_create(conn).await?;

        // Update settings
        SettingRepository::update(conn, settings, params).await
    }

    fn validate_theme(theme: &str) -> Result<(), AppError> {
        match theme {
            "light" | "dark" | "system" => Ok(()),
            _ => Err(AppError::Validation(format!(
                "Invalid theme: {}. Must be 'light', 'dark', or 'system'",
                theme
            ))),
        }
    }

    fn validate_log_level(level: &str) -> Result<(), AppError> {
        match level {
            "error" | "warn" | "info" | "debug" | "trace" => Ok(()),
            _ => Err(AppError::Validation(format!(
                "Invalid log level: {}. Must be 'error', 'warn', 'info', 'debug', or 'trace'",
                level
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_theme_light() {
        assert!(SettingService::validate_theme("light").is_ok());
    }

    #[test]
    fn test_validate_theme_dark() {
        assert!(SettingService::validate_theme("dark").is_ok());
    }

    #[test]
    fn test_validate_theme_system() {
        assert!(SettingService::validate_theme("system").is_ok());
    }

    #[test]
    fn test_validate_theme_invalid() {
        let result = SettingService::validate_theme("blue");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_log_level_all_valid() {
        assert!(SettingService::validate_log_level("error").is_ok());
        assert!(SettingService::validate_log_level("warn").is_ok());
        assert!(SettingService::validate_log_level("info").is_ok());
        assert!(SettingService::validate_log_level("debug").is_ok());
        assert!(SettingService::validate_log_level("trace").is_ok());
    }

    #[test]
    fn test_validate_log_level_invalid() {
        let result = SettingService::validate_log_level("verbose");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::entities::setting::Entity as Setting;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Schema};

    async fn setup_test_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(Setting);
        conn.execute(conn.get_database_backend().build(&stmt))
            .await
            .unwrap();
        conn
    }

    #[tokio::test]
    async fn test_update_validates_theme() {
        let conn = setup_test_db().await;
        let params = UpdateSettingsParams {
            theme: Some("invalid".to_string()),
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }
}
