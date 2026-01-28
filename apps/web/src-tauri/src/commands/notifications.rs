use serde::Deserialize;
use tauri::State;
use tauri_plugin_notification::NotificationExt;

use crate::db::DbState;
use crate::error::AppError;
use crate::services::SettingService;

/// Input for sending a notification
#[derive(Debug, Deserialize)]
pub struct SendNotificationInput {
    pub title: String,
    pub body: String,
}

/// Check if notifications are enabled
#[tauri::command]
pub async fn are_notifications_enabled(db: State<'_, DbState>) -> Result<bool, AppError> {
    let settings = SettingService::get(db.conn()).await?;
    Ok(settings.enable_notifications)
}

/// Send a desktop notification (respects enable_notifications setting)
#[tauri::command]
pub async fn send_notification(
    app: tauri::AppHandle,
    input: SendNotificationInput,
    db: State<'_, DbState>,
) -> Result<bool, AppError> {
    let settings = SettingService::get(db.conn()).await?;

    if !settings.enable_notifications {
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
