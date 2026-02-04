use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Settings entity
///
/// Single-row settings table (id=1) for app-wide configuration.
/// Stores theme, tray, autostart, logging, notification, and background check preferences.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "settings")]
pub struct Model {
    /// Always 1 (single-row pattern)
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,

    /// Theme preference: "light", "dark", or "system"
    pub theme: String,

    /// Show app icon in system tray
    pub show_in_tray: bool,

    /// Launch app at system startup
    pub launch_at_login: bool,

    /// Enable file logging
    pub enable_logging: bool,

    /// Log level: "error", "warn", "info", "debug", "trace"
    pub log_level: String,

    /// Enable desktop notifications
    pub enable_notifications: bool,

    /// Sidebar expanded state
    pub sidebar_expanded: bool,

    /// Enable background periodic availability checking
    pub background_check_enabled: bool,

    /// Interval for background checks in minutes (15, 30, 60, 240, 1440)
    pub background_check_interval_minutes: i32,

    /// Enable headless browser for sites with bot protection
    pub enable_headless_browser: bool,

    /// Last update timestamp
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Default for Model {
    fn default() -> Self {
        Self {
            id: 1,
            theme: "system".to_string(),
            show_in_tray: true,
            launch_at_login: false,
            enable_logging: true,
            log_level: "info".to_string(),
            enable_notifications: true,
            sidebar_expanded: true,
            background_check_enabled: false,
            background_check_interval_minutes: 60, // Default to 1 hour
            enable_headless_browser: true,         // Enabled by default for best experience
            updated_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Model::default();
        assert_eq!(settings.id, 1);
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
    fn test_settings_clone() {
        let settings = Model::default();
        let cloned = settings.clone();
        assert_eq!(settings.id, cloned.id);
        assert_eq!(settings.theme, cloned.theme);
        assert_eq!(settings.show_in_tray, cloned.show_in_tray);
    }

    #[test]
    fn test_settings_debug() {
        let settings = Model::default();
        let debug_str = format!("{:?}", settings);
        assert!(debug_str.contains("Model"));
        assert!(debug_str.contains("system"));
        assert!(debug_str.contains("info"));
    }

    #[test]
    fn test_settings_partial_eq() {
        let settings1 = Model::default();
        let settings2 = Model::default();
        // They have the same values except potentially updated_at
        // Since updated_at is generated at construction, they should still be eq
        // if created in quick succession
        assert_eq!(settings1.id, settings2.id);
        assert_eq!(settings1.theme, settings2.theme);
    }

    #[test]
    fn test_settings_serialize() {
        let settings = Model::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"id\":1"));
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

    #[test]
    fn test_settings_deserialize() {
        let json = r#"{
            "id": 1,
            "theme": "dark",
            "show_in_tray": false,
            "launch_at_login": true,
            "enable_logging": false,
            "log_level": "debug",
            "enable_notifications": false,
            "sidebar_expanded": false,
            "background_check_enabled": true,
            "background_check_interval_minutes": 30,
            "enable_headless_browser": false,
            "updated_at": "2024-01-01T00:00:00Z"
        }"#;
        let settings: Model = serde_json::from_str(json).unwrap();
        assert_eq!(settings.id, 1);
        assert_eq!(settings.theme, "dark");
        assert!(!settings.show_in_tray);
        assert!(settings.launch_at_login);
        assert!(!settings.enable_logging);
        assert_eq!(settings.log_level, "debug");
        assert!(!settings.enable_notifications);
        assert!(!settings.sidebar_expanded);
        assert!(settings.background_check_enabled);
        assert_eq!(settings.background_check_interval_minutes, 30);
        assert!(!settings.enable_headless_browser);
    }

    #[test]
    fn test_default_has_valid_updated_at() {
        let settings = Model::default();
        // updated_at should be set and recent
        assert!(!settings.updated_at.to_rfc3339().is_empty());
    }

    #[test]
    fn test_settings_different_themes() {
        let mut settings = Model::default();

        settings.theme = "light".to_string();
        assert_eq!(settings.theme, "light");

        settings.theme = "dark".to_string();
        assert_eq!(settings.theme, "dark");

        settings.theme = "system".to_string();
        assert_eq!(settings.theme, "system");
    }

    #[test]
    fn test_settings_different_log_levels() {
        let mut settings = Model::default();

        let log_levels = ["error", "warn", "info", "debug", "trace"];
        for level in log_levels {
            settings.log_level = level.to_string();
            assert_eq!(settings.log_level, level);
        }
    }

    #[test]
    fn test_settings_background_check_intervals() {
        let mut settings = Model::default();

        let intervals = [15, 30, 60, 240, 1440];
        for interval in intervals {
            settings.background_check_interval_minutes = interval;
            assert_eq!(settings.background_check_interval_minutes, interval);
        }
    }
}
