use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::entities::app_setting::SettingScope;
use crate::error::AppError;
use crate::repositories::{ScopedSettingsReader, SettingsHelpers};

/// Setting keys for global settings
pub mod keys {
    pub const THEME: &str = "theme";
    pub const SHOW_IN_TRAY: &str = "show_in_tray";
    pub const LAUNCH_AT_LOGIN: &str = "launch_at_login";
    pub const ENABLE_LOGGING: &str = "enable_logging";
    pub const LOG_LEVEL: &str = "log_level";
    pub const ENABLE_NOTIFICATIONS: &str = "enable_notifications";
    pub const SIDEBAR_EXPANDED: &str = "sidebar_expanded";
    pub const BACKGROUND_CHECK_ENABLED: &str = "background_check_enabled";
    pub const BACKGROUND_CHECK_INTERVAL_MINUTES: &str = "background_check_interval_minutes";
    pub const ENABLE_HEADLESS_BROWSER: &str = "enable_headless_browser";
}

/// Default values for settings
pub mod defaults {
    pub const THEME: &str = "system";
    pub const SHOW_IN_TRAY: bool = true;
    pub const LAUNCH_AT_LOGIN: bool = false;
    pub const ENABLE_LOGGING: bool = true;
    pub const LOG_LEVEL: &str = "info";
    pub const ENABLE_NOTIFICATIONS: bool = true;
    pub const SIDEBAR_EXPANDED: bool = true;
    pub const BACKGROUND_CHECK_ENABLED: bool = false;
    pub const BACKGROUND_CHECK_INTERVAL_MINUTES: i32 = 60;
    pub const ENABLE_HEADLESS_BROWSER: bool = true;
}

/// Settings model returned by the service
///
/// Maintains the same shape as the old SettingModel for backward compatibility
/// with the frontend.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub show_in_tray: bool,
    pub launch_at_login: bool,
    pub enable_logging: bool,
    pub log_level: String,
    pub enable_notifications: bool,
    pub sidebar_expanded: bool,
    pub background_check_enabled: bool,
    pub background_check_interval_minutes: i32,
    pub enable_headless_browser: bool,
    pub updated_at: DateTime<Utc>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: defaults::THEME.to_string(),
            show_in_tray: defaults::SHOW_IN_TRAY,
            launch_at_login: defaults::LAUNCH_AT_LOGIN,
            enable_logging: defaults::ENABLE_LOGGING,
            log_level: defaults::LOG_LEVEL.to_string(),
            enable_notifications: defaults::ENABLE_NOTIFICATIONS,
            sidebar_expanded: defaults::SIDEBAR_EXPANDED,
            background_check_enabled: defaults::BACKGROUND_CHECK_ENABLED,
            background_check_interval_minutes: defaults::BACKGROUND_CHECK_INTERVAL_MINUTES,
            enable_headless_browser: defaults::ENABLE_HEADLESS_BROWSER,
            updated_at: Utc::now(),
        }
    }
}

/// Parameters for updating settings (all fields optional for partial updates)
#[derive(Default)]
pub struct UpdateSettingsParams {
    pub theme: Option<String>,
    pub show_in_tray: Option<bool>,
    pub launch_at_login: Option<bool>,
    pub enable_logging: Option<bool>,
    pub log_level: Option<String>,
    pub enable_notifications: Option<bool>,
    pub sidebar_expanded: Option<bool>,
    pub background_check_enabled: Option<bool>,
    pub background_check_interval_minutes: Option<i32>,
    pub enable_headless_browser: Option<bool>,
}

/// Service layer for settings business logic
///
/// Validates inputs and orchestrates EAV repository calls.
pub struct SettingService;

impl SettingService {
    /// Get current settings, reading from EAV storage with defaults
    pub async fn get(conn: &DatabaseConnection) -> Result<Settings, AppError> {
        let scope = SettingScope::Global;
        let r = ScopedSettingsReader::new(conn, &scope);

        Ok(Settings {
            theme: r.string(keys::THEME, defaults::THEME).await?,
            show_in_tray: r.bool(keys::SHOW_IN_TRAY, defaults::SHOW_IN_TRAY).await?,
            launch_at_login: r
                .bool(keys::LAUNCH_AT_LOGIN, defaults::LAUNCH_AT_LOGIN)
                .await?,
            enable_logging: r
                .bool(keys::ENABLE_LOGGING, defaults::ENABLE_LOGGING)
                .await?,
            log_level: r.string(keys::LOG_LEVEL, defaults::LOG_LEVEL).await?,
            enable_notifications: r
                .bool(keys::ENABLE_NOTIFICATIONS, defaults::ENABLE_NOTIFICATIONS)
                .await?,
            sidebar_expanded: r
                .bool(keys::SIDEBAR_EXPANDED, defaults::SIDEBAR_EXPANDED)
                .await?,
            background_check_enabled: r
                .bool(
                    keys::BACKGROUND_CHECK_ENABLED,
                    defaults::BACKGROUND_CHECK_ENABLED,
                )
                .await?,
            background_check_interval_minutes: r
                .i32(
                    keys::BACKGROUND_CHECK_INTERVAL_MINUTES,
                    defaults::BACKGROUND_CHECK_INTERVAL_MINUTES,
                )
                .await?,
            enable_headless_browser: r
                .bool(
                    keys::ENABLE_HEADLESS_BROWSER,
                    defaults::ENABLE_HEADLESS_BROWSER,
                )
                .await?,
            updated_at: Utc::now(),
        })
    }

    /// Update settings with validation
    pub async fn update(
        conn: &DatabaseConnection,
        params: UpdateSettingsParams,
    ) -> Result<Settings, AppError> {
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

        let scope = SettingScope::Global;

        // Update each setting independently if a value was provided.
        // This pattern enables partial updates where clients only send changed fields.
        // String settings (theme, log_level) are handled first, followed by boolean
        // settings, then integer settings.
        if let Some(theme) = params.theme {
            SettingsHelpers::set_string(conn, &scope, keys::THEME, &theme).await?;
        }
        if let Some(log_level) = params.log_level {
            SettingsHelpers::set_string(conn, &scope, keys::LOG_LEVEL, &log_level).await?;
        }
        if let Some(v) = params.show_in_tray {
            SettingsHelpers::set_bool(conn, &scope, keys::SHOW_IN_TRAY, v).await?;
        }
        if let Some(v) = params.launch_at_login {
            SettingsHelpers::set_bool(conn, &scope, keys::LAUNCH_AT_LOGIN, v).await?;
        }
        if let Some(v) = params.enable_logging {
            SettingsHelpers::set_bool(conn, &scope, keys::ENABLE_LOGGING, v).await?;
        }
        if let Some(v) = params.enable_notifications {
            SettingsHelpers::set_bool(conn, &scope, keys::ENABLE_NOTIFICATIONS, v).await?;
        }
        if let Some(v) = params.sidebar_expanded {
            SettingsHelpers::set_bool(conn, &scope, keys::SIDEBAR_EXPANDED, v).await?;
        }
        if let Some(v) = params.background_check_enabled {
            SettingsHelpers::set_bool(conn, &scope, keys::BACKGROUND_CHECK_ENABLED, v).await?;
        }
        if let Some(v) = params.enable_headless_browser {
            SettingsHelpers::set_bool(conn, &scope, keys::ENABLE_HEADLESS_BROWSER, v).await?;
        }
        if let Some(v) = params.background_check_interval_minutes {
            SettingsHelpers::set_i32(conn, &scope, keys::BACKGROUND_CHECK_INTERVAL_MINUTES, v)
                .await?;
        }

        // Return current settings
        Self::get(conn).await
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
    fn test_validate_theme_accepts_light() {
        assert!(SettingService::validate_theme("light").is_ok());
    }

    #[test]
    fn test_validate_theme_accepts_dark() {
        assert!(SettingService::validate_theme("dark").is_ok());
    }

    #[test]
    fn test_validate_theme_accepts_system() {
        assert!(SettingService::validate_theme("system").is_ok());
    }

    #[test]
    fn test_validate_theme_rejects_invalid_value() {
        let result = SettingService::validate_theme("blue");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_log_level_accepts_all_valid_levels() {
        assert!(SettingService::validate_log_level("error").is_ok());
        assert!(SettingService::validate_log_level("warn").is_ok());
        assert!(SettingService::validate_log_level("info").is_ok());
        assert!(SettingService::validate_log_level("debug").is_ok());
        assert!(SettingService::validate_log_level("trace").is_ok());
    }

    #[test]
    fn test_validate_log_level_rejects_invalid_value() {
        let result = SettingService::validate_log_level("verbose");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_background_check_interval_accepts_positive_values() {
        assert!(SettingService::validate_background_check_interval(15).is_ok());
        assert!(SettingService::validate_background_check_interval(30).is_ok());
        assert!(SettingService::validate_background_check_interval(60).is_ok());
        assert!(SettingService::validate_background_check_interval(1440).is_ok());
    }

    #[test]
    fn test_validate_background_check_interval_rejects_zero() {
        let result = SettingService::validate_background_check_interval(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_background_check_interval_rejects_negative_values() {
        let result = SettingService::validate_background_check_interval(-1);
        assert!(result.is_err());
        let result = SettingService::validate_background_check_interval(-100);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.theme, "system");
        assert!(settings.show_in_tray);
        assert!(!settings.launch_at_login);
        assert!(settings.enable_logging);
        assert_eq!(settings.log_level, "info");
        assert!(settings.enable_notifications);
        assert!(settings.sidebar_expanded);
        assert!(!settings.background_check_enabled);
        assert_eq!(settings.background_check_interval_minutes, 60);
        assert!(settings.enable_headless_browser);
    }

    #[test]
    fn test_settings_serialize() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"theme\":\"system\""));
        assert!(json.contains("\"show_in_tray\":true"));
        assert!(json.contains("\"launch_at_login\":false"));
        assert!(json.contains("\"enable_logging\":true"));
        assert!(json.contains("\"log_level\":\"info\""));
        assert!(json.contains("\"enable_notifications\":true"));
        assert!(json.contains("\"sidebar_expanded\":true"));
        assert!(json.contains("\"background_check_enabled\":false"));
        assert!(json.contains("\"background_check_interval_minutes\":60"));
        assert!(json.contains("\"enable_headless_browser\":true"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::test_utils::setup_app_settings_db;

    #[tokio::test]
    async fn test_get_returns_defaults() {
        let conn = setup_app_settings_db().await;
        let result = SettingService::get(&conn).await;

        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.theme, "system");
        assert!(settings.show_in_tray);
        assert!(!settings.launch_at_login);
        assert!(settings.enable_logging);
        assert_eq!(settings.log_level, "info");
        assert!(settings.enable_notifications);
        assert!(settings.sidebar_expanded);
        assert!(!settings.background_check_enabled);
        assert_eq!(settings.background_check_interval_minutes, 60);
        assert!(settings.enable_headless_browser);
    }

    #[tokio::test]
    async fn test_get_returns_same_settings() {
        let conn = setup_app_settings_db().await;

        let first = SettingService::get(&conn).await.unwrap();
        let second = SettingService::get(&conn).await.unwrap();

        assert_eq!(first.theme, second.theme);
        assert_eq!(first.show_in_tray, second.show_in_tray);
    }

    #[tokio::test]
    async fn test_update_validates_theme() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            theme: Some("invalid".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_validates_log_level() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            log_level: Some("invalid_level".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_theme_success() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            theme: Some("dark".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().theme, "dark");
    }

    #[tokio::test]
    async fn test_update_log_level_success() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            log_level: Some("debug".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().log_level, "debug");
    }

    #[tokio::test]
    async fn test_update_show_in_tray() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            show_in_tray: Some(false),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().show_in_tray);
    }

    #[tokio::test]
    async fn test_update_launch_at_login() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            launch_at_login: Some(true),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(result.unwrap().launch_at_login);
    }

    #[tokio::test]
    async fn test_update_enable_logging() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            enable_logging: Some(false),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().enable_logging);
    }

    #[tokio::test]
    async fn test_update_enable_notifications() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            enable_notifications: Some(false),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().enable_notifications);
    }

    #[tokio::test]
    async fn test_update_sidebar_expanded() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            sidebar_expanded: Some(true),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(result.unwrap().sidebar_expanded);
    }

    #[tokio::test]
    async fn test_update_multiple_fields() {
        let conn = setup_app_settings_db().await;
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
        let conn = setup_app_settings_db().await;
        let result = SettingService::update(&conn, UpdateSettingsParams::default()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_validates_background_check_interval_negative() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            background_check_interval_minutes: Some(-1),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_validates_background_check_interval_zero() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            background_check_interval_minutes: Some(0),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_background_check_interval_success() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            background_check_interval_minutes: Some(30),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().background_check_interval_minutes, 30);
    }

    #[tokio::test]
    async fn test_update_headless_browser_success() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            enable_headless_browser: Some(false),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().enable_headless_browser);
    }

    #[tokio::test]
    async fn test_settings_persist_across_calls() {
        let conn = setup_app_settings_db().await;

        // Update theme
        let params = UpdateSettingsParams {
            theme: Some("dark".to_string()),
            ..Default::default()
        };
        SettingService::update(&conn, params).await.unwrap();

        // Get settings and verify theme persisted
        let settings = SettingService::get(&conn).await.unwrap();
        assert_eq!(settings.theme, "dark");
    }
}
