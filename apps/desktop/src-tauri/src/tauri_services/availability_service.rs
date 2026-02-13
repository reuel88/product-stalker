//! Tauri-specific availability service with event emission.
//!
//! This wraps the domain's AvailabilityService and adds:
//! - Tauri event emission for progress tracking
//! - Desktop notification composition
//! - Settings integration for headless browser toggle

use std::time::Duration;

use sea_orm::DatabaseConnection;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::core::services::{SettingService, SettingsCache};
use crate::core::AppError;
use crate::domain::services::{
    AvailabilityService, BulkCheckSummary, DomainSettingService, DomainSettingsCache,
    NotificationData, ProductService,
};

/// Delay in milliseconds between consecutive product checks during bulk operations.
const RATE_LIMIT_BETWEEN_CHECKS_MS: u64 = 500;

/// Event emitted for each product check during bulk operations
#[derive(Debug, Clone, Serialize)]
pub struct BulkCheckProgressEvent {
    pub product_id: String,
    pub status: String,
    pub current: usize,
    pub total: usize,
}

/// Re-export domain's CheckResultWithNotification for use by commands
pub use crate::domain::services::CheckResultWithNotification;

/// Result of checking all products with notification
pub struct TauriBulkCheckResult {
    pub summary: BulkCheckSummary,
    pub notification: Option<NotificationData>,
}

/// Tauri-aware availability service
pub struct TauriAvailabilityService;

impl TauriAvailabilityService {
    /// Check availability for a single product and send notification if back in stock.
    ///
    /// Delegates to domain's `AvailabilityService::check_product_with_notification`
    /// after fetching Tauri-specific settings.
    pub async fn check_product_with_notification(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<CheckResultWithNotification, AppError> {
        let settings = SettingService::get(conn).await?;
        let domain_settings = DomainSettingService::get(conn).await?;
        AvailabilityService::check_product_with_notification(
            conn,
            product_id,
            domain_settings.enable_headless_browser,
            settings.enable_notifications,
            domain_settings.allow_manual_verification,
            domain_settings.session_cache_duration_days,
        )
        .await
    }

    /// Check all products with progress events and bulk notification.
    ///
    /// Emits "availability:check-progress" events for each product checked.
    /// Delegates per-product checking to domain's `AvailabilityService::check_single_product`.
    /// Uses settings caching to avoid repeated database reads during bulk processing.
    pub async fn check_all_products_with_notification(
        conn: &DatabaseConnection,
        app: &AppHandle,
    ) -> Result<TauriBulkCheckResult, AppError> {
        // Load settings once and cache for the entire bulk operation
        let settings_cache = SettingsCache::load(conn).await?;
        let domain_cache = DomainSettingsCache::load(conn).await?;
        let enable_headless = domain_cache.enable_headless_browser();
        let allow_manual_verification = domain_cache.allow_manual_verification();
        let session_cache_duration = domain_cache.session_cache_duration_days();

        let products = ProductService::get_all(conn).await?;
        let total = products.len();

        if total == 0 {
            return Ok(TauriBulkCheckResult {
                summary: BulkCheckSummary {
                    total: 0,
                    successful: 0,
                    failed: 0,
                    back_in_stock_count: 0,
                    price_drop_count: 0,
                    results: vec![],
                },
                notification: None,
            });
        }

        let mut paired_results = Vec::with_capacity(total);

        for (index, product) in products.iter().enumerate() {
            if index > 0 {
                tokio::time::sleep(Duration::from_millis(RATE_LIMIT_BETWEEN_CHECKS_MS)).await;
            }

            let (bulk_result, processing_result) = AvailabilityService::check_single_product(
                conn,
                product,
                enable_headless,
                allow_manual_verification,
                session_cache_duration,
            )
            .await;

            let _ = app.emit(
                "availability:check-progress",
                &BulkCheckProgressEvent {
                    product_id: product.id.to_string(),
                    status: bulk_result.status.as_str().to_string(),
                    current: index + 1,
                    total,
                },
            );

            paired_results.push((bulk_result, processing_result));
        }

        let summary = AvailabilityService::build_summary_from_results(total, paired_results);

        let notification = AvailabilityService::build_bulk_notification_with_settings(
            settings_cache.enable_notifications(),
            &summary,
        );

        Ok(TauriBulkCheckResult {
            summary,
            notification,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bulk_check_progress_event_serializes() {
        let event = BulkCheckProgressEvent {
            product_id: "test-id".to_string(),
            status: "in_stock".to_string(),
            current: 1,
            total: 10,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("test-id"));
        assert!(json.contains("in_stock"));
        assert!(json.contains("\"current\":1"));
        assert!(json.contains("\"total\":10"));
    }
}
