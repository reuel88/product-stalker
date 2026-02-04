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

        // Validate background check interval if provided
        if let Some(interval) = params.background_check_interval_minutes {
            Self::validate_background_check_interval(interval)?;
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

    fn validate_background_check_interval(interval: i32) -> Result<(), AppError> {
        if interval <= 0 {
            return Err(AppError::Validation(
                "Background check interval must be a positive number of minutes".to_string(),
            ));
        }
        Ok(())
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

    #[test]
    fn test_validate_background_check_interval_valid() {
        assert!(SettingService::validate_background_check_interval(15).is_ok());
        assert!(SettingService::validate_background_check_interval(30).is_ok());
        assert!(SettingService::validate_background_check_interval(60).is_ok());
        assert!(SettingService::validate_background_check_interval(1440).is_ok());
    }

    #[test]
    fn test_validate_background_check_interval_zero() {
        let result = SettingService::validate_background_check_interval(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_background_check_interval_negative() {
        let result = SettingService::validate_background_check_interval(-1);
        assert!(result.is_err());
        let result = SettingService::validate_background_check_interval(-100);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::test_utils::setup_settings_db;

    #[tokio::test]
    async fn test_get_creates_defaults() {
        let conn = setup_settings_db().await;
        let result = SettingService::get(&conn).await;

        assert!(result.is_ok());
        let settings = result.unwrap();
        // Verify default values are set
        assert_eq!(settings.id, 1);
    }

    #[tokio::test]
    async fn test_get_returns_same_settings() {
        let conn = setup_settings_db().await;

        let first = SettingService::get(&conn).await.unwrap();
        let second = SettingService::get(&conn).await.unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(first.theme, second.theme);
    }

    #[tokio::test]
    async fn test_update_validates_theme() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: Some("invalid".to_string()),
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: None,
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_validates_log_level() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: Some("invalid_level".to_string()),
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: None,
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_theme_success() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: Some("dark".to_string()),
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: None,
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().theme, "dark");
    }

    #[tokio::test]
    async fn test_update_log_level_success() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: Some("debug".to_string()),
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: None,
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().log_level, "debug");
    }

    #[tokio::test]
    async fn test_update_show_in_tray() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: Some(false),
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: None,
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().show_in_tray);
    }

    #[tokio::test]
    async fn test_update_launch_at_login() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: Some(true),
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: None,
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(result.unwrap().launch_at_login);
    }

    #[tokio::test]
    async fn test_update_enable_logging() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: Some(false),
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: None,
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().enable_logging);
    }

    #[tokio::test]
    async fn test_update_enable_notifications() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: Some(false),
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: None,
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().enable_notifications);
    }

    #[tokio::test]
    async fn test_update_sidebar_expanded() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: Some(true),
            background_check_enabled: None,
            background_check_interval_minutes: None,
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(result.unwrap().sidebar_expanded);
    }

    #[tokio::test]
    async fn test_update_multiple_fields() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: Some("light".to_string()),
            show_in_tray: Some(true),
            launch_at_login: Some(true),
            enable_logging: Some(true),
            log_level: Some("trace".to_string()),
            enable_notifications: Some(true),
            sidebar_expanded: Some(true),
            background_check_enabled: Some(true),
            background_check_interval_minutes: Some(30),
            enable_headless_browser: Some(false),
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.theme, "light");
        assert!(settings.show_in_tray);
        assert!(settings.launch_at_login);
        assert!(settings.enable_logging);
        assert_eq!(settings.log_level, "trace");
        assert!(settings.enable_notifications);
        assert!(settings.sidebar_expanded);
        assert!(settings.background_check_enabled);
        assert_eq!(settings.background_check_interval_minutes, 30);
        assert!(!settings.enable_headless_browser);
    }

    #[tokio::test]
    async fn test_update_no_fields_does_not_error() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: None,
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_validates_background_check_interval_negative() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: Some(-1),
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_validates_background_check_interval_zero() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: Some(0),
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_background_check_interval_success() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: Some(30),
            enable_headless_browser: None,
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().background_check_interval_minutes, 30);
    }

    #[tokio::test]
    async fn test_update_headless_browser_success() {
        let conn = setup_settings_db().await;
        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: None,
            background_check_interval_minutes: None,
            enable_headless_browser: Some(false),
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().enable_headless_browser);
    }
}
