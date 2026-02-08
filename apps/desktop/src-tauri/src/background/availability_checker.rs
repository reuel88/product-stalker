use std::sync::Arc;
use std::time::Duration;

use sea_orm::DatabaseConnection;
use tauri::AppHandle;

use crate::core::services::SettingService;
use crate::tauri_services::{send_desktop_notification, TauriAvailabilityService};

/// Delay in seconds before retrying after a settings fetch error.
///
/// When the background checker fails to load settings (e.g., database error),
/// it waits this long before trying again to avoid tight error loops.
const ERROR_RETRY_DELAY_SECS: u64 = 60;

/// Polling interval in seconds when background checking is disabled.
///
/// The checker periodically re-checks settings even when disabled,
/// so it can start checking when the user enables the feature.
const DISABLED_POLL_INTERVAL_SECS: u64 = 60;

/// State for managing the background checker task.
///
/// Stores the `JoinHandle` so the task can be cancelled if needed (e.g., on app shutdown).
pub struct BackgroundCheckerState {
    _handle: tauri::async_runtime::JoinHandle<()>,
}

/// Spawns the background availability checker task.
///
/// The task periodically checks all products for availability based on settings.
/// It sends desktop notifications when products come back in stock.
pub fn spawn_background_checker(
    app: AppHandle,
    conn: Arc<DatabaseConnection>,
) -> BackgroundCheckerState {
    let handle = tauri::async_runtime::spawn(background_checker_loop(app, conn));

    BackgroundCheckerState { _handle: handle }
}

async fn background_checker_loop(app: AppHandle, conn: Arc<DatabaseConnection>) {
    log::info!("Background availability checker started");

    loop {
        // Get current settings
        let settings = match SettingService::get(&conn).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to get settings in background checker: {}", e);
                tokio::time::sleep(Duration::from_secs(ERROR_RETRY_DELAY_SECS)).await;
                continue;
            }
        };

        // Check if background checking is enabled
        if !settings.background_check_enabled {
            log::debug!(
                "Background checking disabled, sleeping for {} seconds",
                DISABLED_POLL_INTERVAL_SECS
            );
            tokio::time::sleep(Duration::from_secs(DISABLED_POLL_INTERVAL_SECS)).await;
            continue;
        }

        // Perform the check (includes notification logic)
        log::info!("Starting background availability check");
        match TauriAvailabilityService::check_all_products_with_notification(&conn, &app).await {
            Ok(result) => {
                log::info!(
                    "Background check complete: {}/{} successful, {} back in stock, {} price drops",
                    result.summary.successful,
                    result.summary.total,
                    result.summary.back_in_stock_count,
                    result.summary.price_drop_count
                );

                if let Some(notification) = result.notification {
                    send_desktop_notification(&app, &notification);
                }
            }
            Err(e) => {
                log::error!("Background availability check failed: {}", e);
            }
        }

        // Sleep for the configured interval
        let interval_secs = (settings.background_check_interval_minutes as u64) * 60;
        log::debug!(
            "Background checker sleeping for {} minutes",
            settings.background_check_interval_minutes
        );
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_error_retry_delay_is_reasonable() {
        assert!(super::ERROR_RETRY_DELAY_SECS > 0);
        assert!(super::ERROR_RETRY_DELAY_SECS <= 300);
    }

    #[test]
    fn test_disabled_poll_interval_is_reasonable() {
        assert!(super::DISABLED_POLL_INTERVAL_SECS > 0);
        assert!(super::DISABLED_POLL_INTERVAL_SECS <= 300);
    }
}
