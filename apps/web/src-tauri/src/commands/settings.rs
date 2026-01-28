use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::db::DbState;
use crate::entities::setting::Model as SettingModel;
use crate::error::AppError;
use crate::services::SettingService;
use crate::TrayState;

/// Input for updating settings (all fields optional for partial updates)
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsInput {
    pub theme: Option<String>,
    pub show_in_tray: Option<bool>,
    pub launch_at_login: Option<bool>,
    pub enable_logging: Option<bool>,
    pub log_level: Option<String>,
    pub enable_notifications: Option<bool>,
    pub sidebar_expanded: Option<bool>,
}

/// Response DTO for settings
#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub theme: String,
    pub show_in_tray: bool,
    pub launch_at_login: bool,
    pub enable_logging: bool,
    pub log_level: String,
    pub enable_notifications: bool,
    pub sidebar_expanded: bool,
    pub updated_at: String,
}

impl From<SettingModel> for SettingsResponse {
    fn from(model: SettingModel) -> Self {
        Self {
            theme: model.theme,
            show_in_tray: model.show_in_tray,
            launch_at_login: model.launch_at_login,
            enable_logging: model.enable_logging,
            log_level: model.log_level,
            enable_notifications: model.enable_notifications,
            sidebar_expanded: model.sidebar_expanded,
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

/// Get current settings
#[tauri::command]
pub async fn get_settings(db: State<'_, DbState>) -> Result<SettingsResponse, AppError> {
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
    input: UpdateSettingsInput,
    db: State<'_, DbState>,
) -> Result<SettingsResponse, AppError> {
    let show_in_tray_value = input.show_in_tray;

    let settings = SettingService::update(
        db.conn(),
        input.theme,
        input.show_in_tray,
        input.launch_at_login,
        input.enable_logging,
        input.log_level,
        input.enable_notifications,
        input.sidebar_expanded,
    )
    .await?;

    if let Some(show_in_tray) = show_in_tray_value {
        update_tray_visibility(&app, show_in_tray);
    }

    Ok(SettingsResponse::from(settings))
}
