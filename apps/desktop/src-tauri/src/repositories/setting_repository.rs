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
    pub background_check_enabled: Option<bool>,
    pub background_check_interval_minutes: Option<i32>,
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
            background_check_enabled: Set(default.background_check_enabled),
            background_check_interval_minutes: Set(default.background_check_interval_minutes),
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
        if let Some(background_check_enabled) = params.background_check_enabled {
            active_model.background_check_enabled = Set(background_check_enabled);
        }
        if let Some(background_check_interval_minutes) = params.background_check_interval_minutes {
            active_model.background_check_interval_minutes = Set(background_check_interval_minutes);
        }
        active_model.updated_at = Set(chrono::Utc::now());

        let updated = active_model.update(conn).await?;
        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    async fn test_get_or_create_creates_defaults() {
        let conn = setup_test_db().await;
        let settings = SettingRepository::get_or_create(&conn).await.unwrap();

        assert_eq!(settings.id, 1);
        assert_eq!(settings.theme, "system");
        assert!(settings.show_in_tray);
    }

    #[tokio::test]
    async fn test_get_or_create_returns_existing() {
        let conn = setup_test_db().await;

        let first = SettingRepository::get_or_create(&conn).await.unwrap();
        let second = SettingRepository::get_or_create(&conn).await.unwrap();

        assert_eq!(first.id, second.id);
    }

    #[tokio::test]
    async fn test_update_settings() {
        let conn = setup_test_db().await;
        let settings = SettingRepository::get_or_create(&conn).await.unwrap();

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
        };

        let updated = SettingRepository::update(&conn, settings, params)
            .await
            .unwrap();
        assert_eq!(updated.theme, "dark");
    }

    #[tokio::test]
    async fn test_update_background_check_settings() {
        let conn = setup_test_db().await;
        let settings = SettingRepository::get_or_create(&conn).await.unwrap();

        // Verify defaults
        assert!(!settings.background_check_enabled);
        assert_eq!(settings.background_check_interval_minutes, 60);

        let params = UpdateSettingsParams {
            theme: None,
            show_in_tray: None,
            launch_at_login: None,
            enable_logging: None,
            log_level: None,
            enable_notifications: None,
            sidebar_expanded: None,
            background_check_enabled: Some(true),
            background_check_interval_minutes: Some(30),
        };

        let updated = SettingRepository::update(&conn, settings, params)
            .await
            .unwrap();
        assert!(updated.background_check_enabled);
        assert_eq!(updated.background_check_interval_minutes, 30);
    }
}
