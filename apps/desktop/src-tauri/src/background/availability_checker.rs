use std::sync::Arc;
use std::time::Duration;

use sea_orm::DatabaseConnection;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::services::{AvailabilityService, SettingService};

/// State for managing the background checker task
#[derive(Default)]
pub struct BackgroundCheckerState {
    _private: (),
}

impl BackgroundCheckerState {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

/// Spawns the background availability checker task.
///
/// The task periodically checks all products for availability based on settings.
/// It sends desktop notifications when products come back in stock.
pub fn spawn_background_checker(
    app: AppHandle,
    conn: Arc<DatabaseConnection>,
) -> BackgroundCheckerState {
    let state = BackgroundCheckerState::new();

    // Use tauri::async_runtime::spawn which works within Tauri's runtime context
    tauri::async_runtime::spawn(background_checker_loop(app, conn));

    state
}

async fn background_checker_loop(app: AppHandle, conn: Arc<DatabaseConnection>) {
    log::info!("Background availability checker started");

    loop {
        // Get current settings
        let settings = match SettingService::get(&conn).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to get settings in background checker: {}", e);
                // Wait before retrying
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }
        };

        // Check if background checking is enabled
        if !settings.background_check_enabled {
            log::debug!("Background checking disabled, sleeping for 60 seconds");
            tokio::time::sleep(Duration::from_secs(60)).await;
            continue;
        }

        // Perform the check
        log::info!("Starting background availability check");
        match AvailabilityService::check_all_products(&conn).await {
            Ok(summary) => {
                log::info!(
                    "Background check complete: {}/{} successful, {} back in stock",
                    summary.successful,
                    summary.total,
                    summary.back_in_stock_count
                );

                // Send notification if products are back in stock
                if summary.back_in_stock_count > 0 && settings.enable_notifications {
                    let back_in_stock_products: Vec<&str> = summary
                        .results
                        .iter()
                        .filter(|r| r.is_back_in_stock)
                        .map(|r| r.product_name.as_str())
                        .collect();

                    let notification_body = if back_in_stock_products.len() == 1 {
                        format!("{} is back in stock!", back_in_stock_products[0])
                    } else {
                        format!(
                            "{} products are back in stock: {}",
                            back_in_stock_products.len(),
                            back_in_stock_products.join(", ")
                        )
                    };

                    if let Err(e) = app
                        .notification()
                        .builder()
                        .title("Products Back in Stock!")
                        .body(&notification_body)
                        .show()
                    {
                        log::warn!("Failed to send background check notification: {}", e);
                    }
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
    use super::*;

    #[test]
    fn test_background_checker_state_new() {
        let state = BackgroundCheckerState::new();
        // Just verify it can be created
        assert!(true, "BackgroundCheckerState created successfully");
        drop(state);
    }

    #[test]
    fn test_background_checker_state_default() {
        let state = BackgroundCheckerState::default();
        assert!(true, "BackgroundCheckerState default created successfully");
        drop(state);
    }
}
