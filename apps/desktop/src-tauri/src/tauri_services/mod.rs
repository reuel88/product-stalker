//! Tauri-specific service wrappers.
//!
//! This module provides Tauri-aware wrappers around the domain services,
//! adding event emission and notification handling that requires Tauri's AppHandle.

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::domain::services::NotificationData;

mod availability_service;

pub use availability_service::TauriAvailabilityService;

/// Send a desktop notification via the Tauri notification plugin.
pub fn send_desktop_notification(app: &AppHandle, notification: &NotificationData) {
    if let Err(e) = app
        .notification()
        .builder()
        .title(&notification.title)
        .body(&notification.body)
        .show()
    {
        log::warn!("Failed to send notification: {}", e);
    } else {
        log::info!("Sent notification: {}", notification.title);
    }
}
