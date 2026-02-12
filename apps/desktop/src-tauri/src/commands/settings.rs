use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::core::services::{SettingService, Settings, UpdateSettingsParams};
use crate::db::DbState;
use crate::domain::services::{DomainSettingService, DomainSettings, UpdateDomainSettingsParams};
use crate::tauri_error::CommandError;
use crate::TrayState;

/// Response DTO for settings.
///
/// Merges core `Settings` and domain `DomainSettings` into a single flat response
/// for the frontend. Converts `updated_at` from `DateTime<Utc>` to RFC 3339 `String`.
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
    pub color_palette: String,
    pub display_timezone: String,
    pub date_format: String,
    pub updated_at: String,
}

impl SettingsResponse {
    pub fn from_merged(settings: Settings, domain: DomainSettings) -> Self {
        Self {
            theme: settings.theme,
            show_in_tray: settings.show_in_tray,
            launch_at_login: settings.launch_at_login,
            enable_logging: settings.enable_logging,
            log_level: settings.log_level,
            enable_notifications: settings.enable_notifications,
            sidebar_expanded: settings.sidebar_expanded,
            background_check_enabled: domain.background_check_enabled,
            background_check_interval_minutes: domain.background_check_interval_minutes,
            enable_headless_browser: domain.enable_headless_browser,
            color_palette: settings.color_palette,
            display_timezone: settings.display_timezone,
            date_format: settings.date_format,
            updated_at: settings.updated_at.to_rfc3339(),
        }
    }
}

/// Combined update params from the frontend.
///
/// The frontend sends a flat object with all settings fields. This struct
/// captures them all and splits them for the appropriate service.
#[derive(Default, Deserialize)]
pub struct CombinedUpdateParams {
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
    pub color_palette: Option<String>,
    pub display_timezone: Option<String>,
    pub date_format: Option<String>,
}

/// Get current settings
#[tauri::command]
pub async fn get_settings(db: State<'_, DbState>) -> Result<SettingsResponse, CommandError> {
    let settings = SettingService::get(db.conn()).await?;
    let domain = DomainSettingService::get(db.conn()).await?;
    Ok(SettingsResponse::from_merged(settings, domain))
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
    input: CombinedUpdateParams,
    db: State<'_, DbState>,
) -> Result<SettingsResponse, CommandError> {
    let show_in_tray_value = input.show_in_tray;

    // Split into core and domain params
    let core_params = UpdateSettingsParams {
        theme: input.theme,
        show_in_tray: input.show_in_tray,
        launch_at_login: input.launch_at_login,
        enable_logging: input.enable_logging,
        log_level: input.log_level,
        enable_notifications: input.enable_notifications,
        sidebar_expanded: input.sidebar_expanded,
        color_palette: input.color_palette,
        display_timezone: input.display_timezone,
        date_format: input.date_format,
    };

    let domain_params = UpdateDomainSettingsParams {
        background_check_enabled: input.background_check_enabled,
        background_check_interval_minutes: input.background_check_interval_minutes,
        enable_headless_browser: input.enable_headless_browser,
    };

    let settings = SettingService::update(db.conn(), core_params).await?;
    let domain = DomainSettingService::update(db.conn(), domain_params).await?;

    if let Some(show_in_tray) = show_in_tray_value {
        update_tray_visibility(&app, show_in_tray);
    }

    Ok(SettingsResponse::from_merged(settings, domain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn test_core_settings() -> Settings {
        Settings {
            theme: "dark".to_string(),
            show_in_tray: true,
            launch_at_login: false,
            enable_logging: true,
            log_level: "info".to_string(),
            enable_notifications: true,
            sidebar_expanded: false,
            color_palette: "default".to_string(),
            display_timezone: "auto".to_string(),
            date_format: "system".to_string(),
            updated_at: Utc::now(),
        }
    }

    fn test_domain_settings() -> DomainSettings {
        DomainSettings {
            background_check_enabled: false,
            background_check_interval_minutes: 60,
            enable_headless_browser: true,
        }
    }

    #[test]
    fn test_settings_response_from_merged() {
        let response = SettingsResponse::from_merged(test_core_settings(), test_domain_settings());

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
        assert_eq!(response.color_palette, "default");
        assert_eq!(response.display_timezone, "auto");
        assert_eq!(response.date_format, "system");
    }

    #[test]
    fn test_settings_response_from_merged_light_theme() {
        let settings = Settings {
            theme: "light".to_string(),
            show_in_tray: false,
            launch_at_login: true,
            enable_logging: false,
            log_level: "error".to_string(),
            enable_notifications: false,
            sidebar_expanded: true,
            color_palette: "ocean".to_string(),
            display_timezone: "America/New_York".to_string(),
            date_format: "MM/DD/YYYY".to_string(),
            updated_at: Utc::now(),
        };
        let domain = DomainSettings {
            background_check_enabled: true,
            background_check_interval_minutes: 30,
            enable_headless_browser: false,
        };

        let response = SettingsResponse::from_merged(settings, domain);

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
        assert_eq!(response.color_palette, "ocean");
        assert_eq!(response.display_timezone, "America/New_York");
        assert_eq!(response.date_format, "MM/DD/YYYY");
    }

    #[test]
    fn test_settings_response_from_merged_system_theme() {
        let settings = Settings {
            theme: "system".to_string(),
            log_level: "debug".to_string(),
            color_palette: "rose".to_string(),
            ..Settings::default()
        };

        let response = SettingsResponse::from_merged(settings, test_domain_settings());

        assert_eq!(response.theme, "system");
        assert_eq!(response.log_level, "debug");
        assert_eq!(response.color_palette, "rose");
    }

    #[test]
    fn test_settings_response_serializes_to_json() {
        let response = SettingsResponse::from_merged(test_core_settings(), test_domain_settings());
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"theme\":\"dark\""));
        assert!(json.contains("\"show_in_tray\":true"));
        assert!(json.contains("\"log_level\":\"info\""));
        assert!(json.contains("\"background_check_enabled\":false"));
        assert!(json.contains("\"background_check_interval_minutes\":60"));
        assert!(json.contains("\"enable_headless_browser\":true"));
        assert!(json.contains("\"color_palette\":\"default\""));
        assert!(json.contains("\"display_timezone\":\"auto\""));
        assert!(json.contains("\"date_format\":\"system\""));
    }

    #[test]
    fn test_settings_response_timestamp_is_rfc3339() {
        let response = SettingsResponse::from_merged(test_core_settings(), test_domain_settings());

        // RFC3339 format includes 'T' separator
        assert!(response.updated_at.contains('T'));
    }

    #[test]
    fn test_combined_update_params_deserializes_partial() {
        let json = r#"{"theme":"light"}"#;
        let input: CombinedUpdateParams = serde_json::from_str(json).unwrap();

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
        assert!(input.color_palette.is_none());
        assert!(input.display_timezone.is_none());
        assert!(input.date_format.is_none());
    }

    #[test]
    fn test_combined_update_params_deserializes_all_fields() {
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
            "enable_headless_browser": false,
            "color_palette": "ocean",
            "display_timezone": "Europe/London",
            "date_format": "DD/MM/YYYY"
        }"#;
        let input: CombinedUpdateParams = serde_json::from_str(json).unwrap();

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
        assert_eq!(input.color_palette, Some("ocean".to_string()));
        assert_eq!(input.display_timezone, Some("Europe/London".to_string()));
        assert_eq!(input.date_format, Some("DD/MM/YYYY".to_string()));
    }

    #[test]
    fn test_combined_update_params_deserializes_empty() {
        let json = r#"{}"#;
        let input: CombinedUpdateParams = serde_json::from_str(json).unwrap();

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
        assert!(input.color_palette.is_none());
        assert!(input.display_timezone.is_none());
        assert!(input.date_format.is_none());
    }

    #[test]
    fn test_combined_update_params_deserializes_booleans_only() {
        let json = r#"{"show_in_tray":false,"launch_at_login":true}"#;
        let input: CombinedUpdateParams = serde_json::from_str(json).unwrap();

        assert!(input.theme.is_none());
        assert_eq!(input.show_in_tray, Some(false));
        assert_eq!(input.launch_at_login, Some(true));
    }

    #[test]
    fn test_combined_update_params_deserializes_log_level_only() {
        let json = r#"{"log_level":"trace"}"#;
        let input: CombinedUpdateParams = serde_json::from_str(json).unwrap();

        assert_eq!(input.log_level, Some("trace".to_string()));
        assert!(input.theme.is_none());
    }

    #[test]
    fn test_combined_update_params_deserializes_background_check_only() {
        let json = r#"{"background_check_enabled":true,"background_check_interval_minutes":15}"#;
        let input: CombinedUpdateParams = serde_json::from_str(json).unwrap();

        assert_eq!(input.background_check_enabled, Some(true));
        assert_eq!(input.background_check_interval_minutes, Some(15));
        assert!(input.theme.is_none());
    }

    #[test]
    fn test_combined_update_params_deserializes_headless_browser_only() {
        let json = r#"{"enable_headless_browser":false}"#;
        let input: CombinedUpdateParams = serde_json::from_str(json).unwrap();

        assert_eq!(input.enable_headless_browser, Some(false));
        assert!(input.theme.is_none());
        assert!(input.background_check_enabled.is_none());
    }
}
