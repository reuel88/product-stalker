use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::entities::setting::{ActiveModel, Entity as Setting, Model as SettingModel};
use crate::error::AppError;

/// Parameters for updating settings
pub struct UpdateSettingsParams {
    pub theme: Option<String>,
    pub show_in_tray: Option<bool>,
    pub launch_at_login: Option<bool>,
    pub enable_logging: Option<bool>,
    pub log_level: Option<String>,
    pub enable_notifications: Option<bool>,
    pub sidebar_expanded: Option<bool>,
}

/// Repository for settings data access
///
/// Implements single-row pattern where settings always have id=1.
pub struct SettingRepository;

impl SettingRepository {
    /// Get settings or create with defaults if not exists
    pub async fn get_or_create(conn: &DatabaseConnection) -> Result<SettingModel, AppError> {
        // Try to find existing settings
        if let Some(settings) = Setting::find_by_id(1).one(conn).await? {
            return Ok(settings);
        }

        // Create default settings
        let default = SettingModel::default();
        let active_model = ActiveModel {
            id: Set(default.id),
            theme: Set(default.theme),
            show_in_tray: Set(default.show_in_tray),
            launch_at_login: Set(default.launch_at_login),
            enable_logging: Set(default.enable_logging),
            log_level: Set(default.log_level),
            enable_notifications: Set(default.enable_notifications),
            sidebar_expanded: Set(default.sidebar_expanded),
            updated_at: Set(default.updated_at),
        };

        let settings = active_model.insert(conn).await?;
        Ok(settings)
    }

    /// Update settings (partial update)
    pub async fn update(
        conn: &DatabaseConnection,
        model: SettingModel,
        params: UpdateSettingsParams,
    ) -> Result<SettingModel, AppError> {
        let mut active_model: ActiveModel = model.into();

        if let Some(theme) = params.theme {
            active_model.theme = Set(theme);
        }
        if let Some(show_in_tray) = params.show_in_tray {
            active_model.show_in_tray = Set(show_in_tray);
        }
        if let Some(launch_at_login) = params.launch_at_login {
            active_model.launch_at_login = Set(launch_at_login);
        }
        if let Some(enable_logging) = params.enable_logging {
            active_model.enable_logging = Set(enable_logging);
        }
        if let Some(log_level) = params.log_level {
            active_model.log_level = Set(log_level);
        }
        if let Some(enable_notifications) = params.enable_notifications {
            active_model.enable_notifications = Set(enable_notifications);
        }
        if let Some(sidebar_expanded) = params.sidebar_expanded {
            active_model.sidebar_expanded = Set(sidebar_expanded);
        }
        active_model.updated_at = Set(chrono::Utc::now());

        let updated = active_model.update(conn).await?;
        Ok(updated)
    }
}
