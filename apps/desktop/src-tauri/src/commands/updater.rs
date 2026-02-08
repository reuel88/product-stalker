use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

use crate::core::AppError;
use crate::tauri_error::CommandError;

/// Information about an available update
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UpdateInfo {
    /// Whether an update is available
    pub available: bool,
    /// The version of the available update (None if no update)
    pub version: Option<String>,
    /// Release notes for the update (None if no update or no notes)
    pub body: Option<String>,
}

impl UpdateInfo {
    /// Create an UpdateInfo indicating an available update
    pub fn available(version: impl Into<String>, body: Option<String>) -> Self {
        Self {
            available: true,
            version: Some(version.into()),
            body,
        }
    }

    /// Create an UpdateInfo indicating no update is available
    pub fn none() -> Self {
        Self {
            available: false,
            version: None,
            body: None,
        }
    }

    /// Check if an update is available
    #[cfg(test)]
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Get the version string if available
    #[cfg(test)]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Get the release notes if available
    #[cfg(test)]
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}

impl Default for UpdateInfo {
    fn default() -> Self {
        Self::none()
    }
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<UpdateInfo, CommandError> {
    let updater = app.updater().map_err(|e| {
        log::error!("Failed to get updater: {}", e);
        AppError::Internal(format!("Failed to get updater: {}", e))
    })?;

    match updater.check().await {
        Ok(Some(update)) => {
            log::info!("Update available: v{}", update.version);
            Ok(UpdateInfo::available(
                update.version.clone(),
                update.body.clone(),
            ))
        }
        Ok(None) => {
            log::info!("No update available");
            Ok(UpdateInfo::none())
        }
        Err(e) => {
            log::error!("Failed to check for updates: {}", e);
            Err(AppError::Internal(format!("Failed to check for updates: {}", e)).into())
        }
    }
}

#[tauri::command]
pub async fn download_and_install_update(app: AppHandle) -> Result<(), CommandError> {
    let updater = app.updater().map_err(|e| {
        log::error!("Failed to get updater: {}", e);
        AppError::Internal(format!("Failed to get updater: {}", e))
    })?;

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => {
            return Err(AppError::Internal("No update available".to_string()).into());
        }
        Err(e) => {
            log::error!("Failed to check for updates: {}", e);
            return Err(AppError::Internal(format!("Failed to check for updates: {}", e)).into());
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
        return Err(
            AppError::Internal(format!("Failed to download and install update: {}", e)).into(),
        );
    }

    log::info!("Update installed, restarting app...");

    // Restart the app to apply the update
    app.restart();
}

#[tauri::command]
pub fn get_current_version(app: AppHandle) -> Result<String, CommandError> {
    let version = app.package_info().version.to_string();
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    // UpdateInfo construction tests

    #[test]
    fn test_update_info_available_with_body() {
        let info = UpdateInfo::available("1.2.3", Some("Release notes".to_string()));

        assert!(info.available);
        assert_eq!(info.version, Some("1.2.3".to_string()));
        assert_eq!(info.body, Some("Release notes".to_string()));
    }

    #[test]
    fn test_update_info_available_without_body() {
        let info = UpdateInfo::available("2.0.0", None);

        assert!(info.available);
        assert_eq!(info.version, Some("2.0.0".to_string()));
        assert_eq!(info.body, None);
    }

    #[test]
    fn test_update_info_available_with_string() {
        let info = UpdateInfo::available(String::from("1.0.0"), None);

        assert!(info.available);
        assert_eq!(info.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_update_info_none() {
        let info = UpdateInfo::none();

        assert!(!info.available);
        assert_eq!(info.version, None);
        assert_eq!(info.body, None);
    }

    #[test]
    fn test_update_info_default() {
        let info = UpdateInfo::default();

        assert!(!info.available);
        assert_eq!(info.version, None);
        assert_eq!(info.body, None);
    }

    #[test]
    fn test_update_info_default_equals_none() {
        assert_eq!(UpdateInfo::default(), UpdateInfo::none());
    }

    // Accessor method tests

    #[test]
    fn test_is_available_true() {
        let info = UpdateInfo::available("1.0.0", None);
        assert!(info.is_available());
    }

    #[test]
    fn test_is_available_false() {
        let info = UpdateInfo::none();
        assert!(!info.is_available());
    }

    #[test]
    fn test_version_accessor_some() {
        let info = UpdateInfo::available("3.2.1", None);
        assert_eq!(info.version(), Some("3.2.1"));
    }

    #[test]
    fn test_version_accessor_none() {
        let info = UpdateInfo::none();
        assert_eq!(info.version(), None);
    }

    #[test]
    fn test_body_accessor_some() {
        let info = UpdateInfo::available("1.0.0", Some("Bug fixes".to_string()));
        assert_eq!(info.body(), Some("Bug fixes"));
    }

    #[test]
    fn test_body_accessor_none() {
        let info = UpdateInfo::available("1.0.0", None);
        assert_eq!(info.body(), None);
    }

    // Trait implementation tests

    #[test]
    fn test_update_info_clone() {
        let info = UpdateInfo::available("1.0.0", Some("Notes".to_string()));
        let cloned = info.clone();

        assert_eq!(info, cloned);
    }

    #[test]
    fn test_update_info_debug() {
        let info = UpdateInfo::available("1.0.0", None);
        let debug_str = format!("{:?}", info);

        assert!(debug_str.contains("UpdateInfo"));
        assert!(debug_str.contains("available: true"));
        assert!(debug_str.contains("1.0.0"));
    }

    #[test]
    fn test_update_info_partial_eq() {
        let info1 = UpdateInfo::available("1.0.0", None);
        let info2 = UpdateInfo::available("1.0.0", None);
        let info3 = UpdateInfo::available("2.0.0", None);

        assert_eq!(info1, info2);
        assert_ne!(info1, info3);
    }

    #[test]
    fn test_update_info_eq_with_different_body() {
        let info1 = UpdateInfo::available("1.0.0", Some("Notes A".to_string()));
        let info2 = UpdateInfo::available("1.0.0", Some("Notes B".to_string()));

        assert_ne!(info1, info2);
    }

    // Serialization tests

    #[test]
    fn test_serialize_available_update() {
        let info = UpdateInfo::available("1.2.3", Some("New features".to_string()));
        let json = serde_json::to_string(&info).unwrap();

        assert!(json.contains(r#""available":true"#));
        assert!(json.contains(r#""version":"1.2.3""#));
        assert!(json.contains(r#""body":"New features""#));
    }

    #[test]
    fn test_serialize_no_update() {
        let info = UpdateInfo::none();
        let json = serde_json::to_string(&info).unwrap();

        assert!(json.contains(r#""available":false"#));
        assert!(json.contains(r#""version":null"#));
        assert!(json.contains(r#""body":null"#));
    }

    #[test]
    fn test_serialize_available_without_body() {
        let info = UpdateInfo::available("1.0.0", None);
        let json = serde_json::to_string(&info).unwrap();

        assert!(json.contains(r#""available":true"#));
        assert!(json.contains(r#""version":"1.0.0""#));
        assert!(json.contains(r#""body":null"#));
    }

    // Edge case tests

    #[test]
    fn test_update_info_with_semver_prerelease() {
        let info = UpdateInfo::available("1.0.0-beta.1", None);
        assert_eq!(info.version(), Some("1.0.0-beta.1"));
    }

    #[test]
    fn test_update_info_with_long_version() {
        let info = UpdateInfo::available("1.2.3.4.5", None);
        assert_eq!(info.version(), Some("1.2.3.4.5"));
    }

    #[test]
    fn test_update_info_with_multiline_body() {
        let body = "## Changes\n- Bug fix 1\n- Bug fix 2\n\n## Notes\nImportant update";
        let info = UpdateInfo::available("1.0.0", Some(body.to_string()));

        assert!(info.body().unwrap().contains("Bug fix 1"));
        assert!(info.body().unwrap().contains("\n"));
    }

    #[test]
    fn test_update_info_with_unicode_body() {
        let info = UpdateInfo::available("1.0.0", Some("更新说明 🚀".to_string()));
        assert!(info.body().unwrap().contains("🚀"));
    }

    #[test]
    fn test_update_info_with_empty_body() {
        let info = UpdateInfo::available("1.0.0", Some(String::new()));
        assert_eq!(info.body(), Some(""));
    }
}
