use sea_orm::DatabaseConnection;
use serde::Deserialize;
use tauri::State;
use tauri_plugin_notification::NotificationExt;

use crate::db::DbState;
use crate::error::AppError;
use crate::services::SettingService;

/// Input for sending a notification
#[derive(Debug, Clone, Deserialize)]
pub struct SendNotificationInput {
    pub title: String,
    pub body: String,
}

impl SendNotificationInput {
    /// Create a new notification input (used in tests)
    #[cfg(test)]
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
        }
    }

    /// Validate the notification input
    pub fn validate(&self) -> Result<(), AppError> {
        if self.title.trim().is_empty() {
            return Err(AppError::Validation(
                "Notification title cannot be empty".to_string(),
            ));
        }
        if self.body.trim().is_empty() {
            return Err(AppError::Validation(
                "Notification body cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

/// Check if notifications are enabled (testable version)
pub async fn check_notifications_enabled(conn: &DatabaseConnection) -> Result<bool, AppError> {
    let settings = SettingService::get(conn).await?;
    Ok(settings.enable_notifications)
}

/// Check if notifications are enabled
#[tauri::command]
pub async fn are_notifications_enabled(db: State<'_, DbState>) -> Result<bool, AppError> {
    check_notifications_enabled(db.conn()).await
}

/// Determine if a notification should be sent based on settings
pub async fn should_send_notification(conn: &DatabaseConnection) -> Result<bool, AppError> {
    let settings = SettingService::get(conn).await?;
    Ok(settings.enable_notifications)
}

/// Send a desktop notification (respects enable_notifications setting)
#[tauri::command]
pub async fn send_notification(
    app: tauri::AppHandle,
    input: SendNotificationInput,
    db: State<'_, DbState>,
) -> Result<bool, AppError> {
    // Validate input
    input.validate()?;

    if !should_send_notification(db.conn()).await? {
        log::debug!(
            "Notification skipped (notifications disabled): {}",
            input.title
        );
        return Ok(false);
    }

    app.notification()
        .builder()
        .title(&input.title)
        .body(&input.body)
        .show()
        .map_err(|e| AppError::Validation(format!("Failed to send notification: {}", e)))?;

    log::info!("Notification sent: {}", input.title);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_notification_input_new() {
        let input = SendNotificationInput::new("Test Title", "Test Body");
        assert_eq!(input.title, "Test Title");
        assert_eq!(input.body, "Test Body");
    }

    #[test]
    fn test_send_notification_input_new_with_string() {
        let input = SendNotificationInput::new(String::from("Title"), String::from("Body"));
        assert_eq!(input.title, "Title");
        assert_eq!(input.body, "Body");
    }

    #[test]
    fn test_send_notification_input_clone() {
        let input = SendNotificationInput::new("Title", "Body");
        let cloned = input.clone();
        assert_eq!(input.title, cloned.title);
        assert_eq!(input.body, cloned.body);
    }

    #[test]
    fn test_send_notification_input_debug() {
        let input = SendNotificationInput::new("Title", "Body");
        let debug_str = format!("{:?}", input);
        assert!(debug_str.contains("Title"));
        assert!(debug_str.contains("Body"));
    }

    #[test]
    fn test_validate_valid_input() {
        let input = SendNotificationInput::new("Valid Title", "Valid Body");
        assert!(input.validate().is_ok());
    }

    #[test]
    fn test_validate_empty_title() {
        let input = SendNotificationInput::new("", "Valid Body");
        let result = input.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("title cannot be empty"));
    }

    #[test]
    fn test_validate_whitespace_title() {
        let input = SendNotificationInput::new("   ", "Valid Body");
        let result = input.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_body() {
        let input = SendNotificationInput::new("Valid Title", "");
        let result = input.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("body cannot be empty"));
    }

    #[test]
    fn test_validate_whitespace_body() {
        let input = SendNotificationInput::new("Valid Title", "   ");
        let result = input.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_notification_input() {
        let json = r#"{"title": "Test", "body": "Message"}"#;
        let input: SendNotificationInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.title, "Test");
        assert_eq!(input.body, "Message");
    }

    #[test]
    fn test_deserialize_notification_input_with_unicode() {
        let json = r#"{"title": "测试通知", "body": "这是一条消息 🔔"}"#;
        let input: SendNotificationInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.title, "测试通知");
        assert!(input.body.contains("🔔"));
    }

    #[test]
    fn test_deserialize_missing_field_fails() {
        let json = r#"{"title": "Test"}"#;
        let result: Result<SendNotificationInput, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::entities::setting::Entity as Setting;
    use crate::repositories::{SettingRepository, UpdateSettingsParams};
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
    async fn test_check_notifications_enabled_default() {
        let conn = setup_test_db().await;

        // Default settings have notifications enabled
        let enabled = check_notifications_enabled(&conn).await.unwrap();
        assert!(enabled);
    }

    #[tokio::test]
    async fn test_check_notifications_enabled_when_disabled() {
        let conn = setup_test_db().await;

        // Create settings first
        let settings = SettingRepository::get_or_create(&conn).await.unwrap();

        // Disable notifications
        SettingRepository::update(
            &conn,
            settings,
            UpdateSettingsParams {
                enable_notifications: Some(false),
                theme: None,
                show_in_tray: None,
                launch_at_login: None,
                enable_logging: None,
                log_level: None,
                sidebar_expanded: None,
            },
        )
        .await
        .unwrap();

        let enabled = check_notifications_enabled(&conn).await.unwrap();
        assert!(!enabled);
    }

    #[tokio::test]
    async fn test_should_send_notification_default() {
        let conn = setup_test_db().await;

        let should_send = should_send_notification(&conn).await.unwrap();
        assert!(should_send);
    }

    #[tokio::test]
    async fn test_should_send_notification_when_disabled() {
        let conn = setup_test_db().await;

        // Create and disable notifications
        let settings = SettingRepository::get_or_create(&conn).await.unwrap();
        SettingRepository::update(
            &conn,
            settings,
            UpdateSettingsParams {
                enable_notifications: Some(false),
                theme: None,
                show_in_tray: None,
                launch_at_login: None,
                enable_logging: None,
                log_level: None,
                sidebar_expanded: None,
            },
        )
        .await
        .unwrap();

        let should_send = should_send_notification(&conn).await.unwrap();
        assert!(!should_send);
    }

    #[tokio::test]
    async fn test_notification_respects_setting_toggle() {
        let conn = setup_test_db().await;

        // Initially enabled
        assert!(check_notifications_enabled(&conn).await.unwrap());

        // Disable
        let settings = SettingRepository::get_or_create(&conn).await.unwrap();
        let settings = SettingRepository::update(
            &conn,
            settings,
            UpdateSettingsParams {
                enable_notifications: Some(false),
                theme: None,
                show_in_tray: None,
                launch_at_login: None,
                enable_logging: None,
                log_level: None,
                sidebar_expanded: None,
            },
        )
        .await
        .unwrap();

        assert!(!check_notifications_enabled(&conn).await.unwrap());

        // Re-enable
        SettingRepository::update(
            &conn,
            settings,
            UpdateSettingsParams {
                enable_notifications: Some(true),
                theme: None,
                show_in_tray: None,
                launch_at_login: None,
                enable_logging: None,
                log_level: None,
                sidebar_expanded: None,
            },
        )
        .await
        .unwrap();

        assert!(check_notifications_enabled(&conn).await.unwrap());
    }
}
