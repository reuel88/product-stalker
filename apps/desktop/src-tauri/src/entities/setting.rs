use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Settings entity
///
/// Single-row settings table (id=1) for app-wide configuration.
/// Stores theme, tray, autostart, logging, and notification preferences.
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
            updated_at: chrono::Utc::now(),
        }
    }
}
