use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use crate::core::services::{SettingService, Settings, UpdateSettingsParams};
use crate::db::DbState;
use crate::tauri_error::CommandError;
use crate::TrayState;

/// Response DTO for settings.
///
/// Mirrors `Settings` fields but converts `updated_at` from `DateTime<Utc>` to an
/// RFC 3339 `String` for JSON serialization. Keep fields in sync with `Settings`.
#[derive(Debug, Serialize)]
pub struct SettingsResponse {
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
    pub updated_at: String,
}

impl From<Settings> for SettingsResponse {
    fn from(settings: Settings) -> Self {
        Self {
            theme: settings.theme,
            show_in_tray: settings.show_in_tray,
            launch_at_login: settings.launch_at_login,
            enable_logging: settings.enable_logging,
            log_level: settings.log_level,
            enable_notifications: settings.enable_notifications,
            sidebar_expanded: settings.sidebar_expanded,
            background_check_enabled: settings.background_check_enabled,
            background_check_interval_minutes: settings.background_check_interval_minutes,
            enable_headless_browser: settings.enable_headless_browser,
            updated_at: settings.updated_at.to_rfc3339(),
        }
    }
}

/// Get current settings
#[tauri::command]
pub async fn get_settings(db: State<'_, DbState>) -> Result<SettingsResponse, CommandError> {
    let settings = SettingService::get(db.conn()).await?;
    Ok(SettingsResponse::from(settings))
}

fn update_tray_visibility(app: &AppHandle, visible: bool) {
    let Some(tray_state) = app.try_state::<TrayState>() else {
        return;
    };

    let Ok(guard) = tray_state.0.lock() else {
        return;
    };

    let Some(tray) = guard.as_ref() else {
        return;
    };

    match tray.set_visible(visible) {
        Ok(()) => log::info!("Tray visibility set to: {}", visible),
        Err(e) => log::error!("Failed to set tray visibility: {}", e),
    }
}

/// Update settings
#[tauri::command]
pub async fn update_settings(
    app: AppHandle,
    input: UpdateSettingsParams,
    db: State<'_, DbState>,
) -> Result<SettingsResponse, CommandError> {
    let show_in_tray_value = input.show_in_tray;

    let settings = SettingService::update(db.conn(), input).await?;

    if let Some(show_in_tray) = show_in_tray_value {
        update_tray_visibility(&app, show_in_tray);
    }

    Ok(SettingsResponse::from(settings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_settings_response_from_settings() {
        let now = Utc::now();
        let settings = Settings {
            theme: "dark".to_string(),
            show_in_tray: true,
            launch_at_login: false,
            enable_logging: true,
            log_level: "info".to_string(),
            enable_notifications: true,
            sidebar_expanded: false,
            background_check_enabled: false,
            background_check_interval_minutes: 60,
            enable_headless_browser: true,
            updated_at: now,
        };

        let response = SettingsResponse::from(settings);

        assert_eq!(response.theme, "dark");
        assert!(response.show_in_tray);
        assert!(!response.launch_at_login);
        assert!(response.enable_logging);
        assert_eq!(response.log_level, "info");
        assert!(response.enable_notifications);
        assert!(!response.sidebar_expanded);
        assert!(!response.background_check_enabled);
        assert_eq!(response.background_check_interval_minutes, 60);
        assert!(response.enable_headless_browser);
    }

    #[test]
    fn test_settings_response_from_settings_light_theme() {
        let now = Utc::now();
        let settings = Settings {
            theme: "light".to_string(),
            show_in_tray: false,
            launch_at_login: true,
            enable_logging: false,
            log_level: "error".to_string(),
            enable_notifications: false,
            sidebar_expanded: true,
            background_check_enabled: true,
            background_check_interval_minutes: 30,
            enable_headless_browser: false,
            updated_at: now,
        };

        let response = SettingsResponse::from(settings);

        assert_eq!(response.theme, "light");
        assert!(!response.show_in_tray);
        assert!(response.launch_at_login);
        assert!(!response.enable_logging);
        assert_eq!(response.log_level, "error");
        assert!(!response.enable_notifications);
        assert!(response.sidebar_expanded);
        assert!(response.background_check_enabled);
        assert_eq!(response.background_check_interval_minutes, 30);
        assert!(!response.enable_headless_browser);
    }

    #[test]
    fn test_settings_response_from_settings_system_theme() {
        let now = Utc::now();
        let settings = Settings {
            theme: "system".to_string(),
            show_in_tray: true,
            launch_at_login: true,
            enable_logging: true,
            log_level: "debug".to_string(),
            enable_notifications: true,
            sidebar_expanded: true,
            background_check_enabled: false,
            background_check_interval_minutes: 60,
            enable_headless_browser: true,
            updated_at: now,
        };

        let response = SettingsResponse::from(settings);

        assert_eq!(response.theme, "system");
        assert_eq!(response.log_level, "debug");
    }

    #[test]
    fn test_settings_response_serializes_to_json() {
        let now = Utc::now();
        let settings = Settings {
            theme: "dark".to_string(),
            show_in_tray: true,
            launch_at_login: false,
            enable_logging: true,
            log_level: "info".to_string(),
            enable_notifications: true,
            sidebar_expanded: false,
            background_check_enabled: false,
            background_check_interval_minutes: 60,
            enable_headless_browser: true,
            updated_at: now,
        };

        let response = SettingsResponse::from(settings);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"theme\":\"dark\""));
        assert!(json.contains("\"show_in_tray\":true"));
        assert!(json.contains("\"log_level\":\"info\""));
        assert!(json.contains("\"background_check_enabled\":false"));
        assert!(json.contains("\"background_check_interval_minutes\":60"));
        assert!(json.contains("\"enable_headless_browser\":true"));
    }

    #[test]
    fn test_settings_response_timestamp_is_rfc3339() {
        let now = Utc::now();
        let settings = Settings {
            theme: "dark".to_string(),
            show_in_tray: true,
            launch_at_login: false,
            enable_logging: true,
            log_level: "info".to_string(),
            enable_notifications: true,
            sidebar_expanded: false,
            background_check_enabled: false,
            background_check_interval_minutes: 60,
            enable_headless_browser: true,
            updated_at: now,
        };

        let response = SettingsResponse::from(settings);

        // RFC3339 format includes 'T' separator
        assert!(response.updated_at.contains('T'));
    }

    #[test]
    fn test_update_settings_input_deserializes_partial() {
        let json = r#"{"theme":"light"}"#;
        let input: UpdateSettingsParams = serde_json::from_str(json).unwrap();

        assert_eq!(input.theme, Some("light".to_string()));
        assert!(input.show_in_tray.is_none());
        assert!(input.launch_at_login.is_none());
        assert!(input.enable_logging.is_none());
        assert!(input.log_level.is_none());
        assert!(input.enable_notifications.is_none());
        assert!(input.sidebar_expanded.is_none());
        assert!(input.background_check_enabled.is_none());
        assert!(input.background_check_interval_minutes.is_none());
        assert!(input.enable_headless_browser.is_none());
    }

    #[test]
    fn test_update_settings_input_deserializes_all_fields() {
        let json = r#"{
            "theme": "dark",
            "show_in_tray": true,
            "launch_at_login": false,
            "enable_logging": true,
            "log_level": "debug",
            "enable_notifications": false,
            "sidebar_expanded": true,
            "background_check_enabled": true,
            "background_check_interval_minutes": 30,
            "enable_headless_browser": false
        }"#;
        let input: UpdateSettingsParams = serde_json::from_str(json).unwrap();

        assert_eq!(input.theme, Some("dark".to_string()));
        assert_eq!(input.show_in_tray, Some(true));
        assert_eq!(input.launch_at_login, Some(false));
        assert_eq!(input.enable_logging, Some(true));
        assert_eq!(input.log_level, Some("debug".to_string()));
        assert_eq!(input.enable_notifications, Some(false));
        assert_eq!(input.sidebar_expanded, Some(true));
        assert_eq!(input.background_check_enabled, Some(true));
        assert_eq!(input.background_check_interval_minutes, Some(30));
        assert_eq!(input.enable_headless_browser, Some(false));
    }

    #[test]
    fn test_update_settings_input_deserializes_empty() {
        let json = r#"{}"#;
        let input: UpdateSettingsParams = serde_json::from_str(json).unwrap();

        assert!(input.theme.is_none());
        assert!(input.show_in_tray.is_none());
        assert!(input.launch_at_login.is_none());
        assert!(input.enable_logging.is_none());
        assert!(input.log_level.is_none());
        assert!(input.enable_notifications.is_none());
        assert!(input.sidebar_expanded.is_none());
        assert!(input.background_check_enabled.is_none());
        assert!(input.background_check_interval_minutes.is_none());
        assert!(input.enable_headless_browser.is_none());
    }

    #[test]
    fn test_update_settings_input_deserializes_booleans_only() {
        let json = r#"{"show_in_tray":false,"launch_at_login":true}"#;
        let input: UpdateSettingsParams = serde_json::from_str(json).unwrap();

        assert!(input.theme.is_none());
        assert_eq!(input.show_in_tray, Some(false));
        assert_eq!(input.launch_at_login, Some(true));
    }

    #[test]
    fn test_update_settings_input_deserializes_log_level_only() {
        let json = r#"{"log_level":"trace"}"#;
        let input: UpdateSettingsParams = serde_json::from_str(json).unwrap();

        assert_eq!(input.log_level, Some("trace".to_string()));
        assert!(input.theme.is_none());
    }

    #[test]
    fn test_update_settings_input_deserializes_background_check_only() {
        let json = r#"{"background_check_enabled":true,"background_check_interval_minutes":15}"#;
        let input: UpdateSettingsParams = serde_json::from_str(json).unwrap();

        assert_eq!(input.background_check_enabled, Some(true));
        assert_eq!(input.background_check_interval_minutes, Some(15));
        assert!(input.theme.is_none());
    }

    #[test]
    fn test_update_settings_input_deserializes_headless_browser_only() {
        let json = r#"{"enable_headless_browser":false}"#;
        let input: UpdateSettingsParams = serde_json::from_str(json).unwrap();

        assert_eq!(input.enable_headless_browser, Some(false));
        assert!(input.theme.is_none());
        assert!(input.background_check_enabled.is_none());
    }
}
