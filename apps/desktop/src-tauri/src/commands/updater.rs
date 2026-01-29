use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub version: Option<String>,
    pub body: Option<String>,
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<UpdateInfo, AppError> {
    let updater = app.updater().map_err(|e| {
        log::error!("Failed to get updater: {}", e);
        AppError::Internal(format!("Failed to get updater: {}", e))
    })?;

    match updater.check().await {
        Ok(Some(update)) => {
            log::info!("Update available: v{}", update.version);
            Ok(UpdateInfo {
                available: true,
                version: Some(update.version.clone()),
                body: update.body.clone(),
            })
        }
        Ok(None) => {
            log::info!("No update available");
            Ok(UpdateInfo {
                available: false,
                version: None,
                body: None,
            })
        }
        Err(e) => {
            log::error!("Failed to check for updates: {}", e);
            Err(AppError::Internal(format!(
                "Failed to check for updates: {}",
                e
            )))
        }
    }
}

#[tauri::command]
pub async fn download_and_install_update(app: AppHandle) -> Result<(), AppError> {
    let updater = app.updater().map_err(|e| {
        log::error!("Failed to get updater: {}", e);
        AppError::Internal(format!("Failed to get updater: {}", e))
    })?;

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => {
            return Err(AppError::Internal("No update available".to_string()));
        }
        Err(e) => {
            log::error!("Failed to check for updates: {}", e);
            return Err(AppError::Internal(format!(
                "Failed to check for updates: {}",
                e
            )));
        }
    };

    log::info!("Downloading update v{}...", update.version);

    // Download and install the update
    let mut downloaded: usize = 0;
    if let Err(e) = update
        .download_and_install(
            |chunk_length, content_length| {
                downloaded += chunk_length;
                log::debug!("Downloaded {} of {:?} bytes", downloaded, content_length);
            },
            || {
                log::info!("Download finished, installing update...");
            },
        )
        .await
    {
        log::error!("Failed to download and install update: {}", e);
        return Err(AppError::Internal(format!(
            "Failed to download and install update: {}",
            e
        )));
    }

    log::info!("Update installed, restarting app...");

    // Restart the app to apply the update
    app.restart();
}

#[tauri::command]
pub fn get_current_version(app: AppHandle) -> Result<String, AppError> {
    let version = app.package_info().version.to_string();
    Ok(version)
}
