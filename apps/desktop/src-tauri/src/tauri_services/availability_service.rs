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

use crate::core::services::SettingService;
use crate::core::AppError;
use crate::domain::entities::prelude::{AvailabilityCheckModel, ProductModel};
use crate::domain::services::{
    AvailabilityService, BulkCheckResult, BulkCheckSummary, DailyPriceComparison, NotificationData,
    NotificationService, ProductService,
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

/// Result of checking a single product with notification
pub struct CheckResultWithNotification {
    pub check: AvailabilityCheckModel,
    pub daily_comparison: DailyPriceComparison,
    pub notification: Option<NotificationData>,
}

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
    /// This is the main entry point for checking a single product from commands.
    pub async fn check_product_with_notification(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<CheckResultWithNotification, AppError> {
        // Get settings to check headless browser preference
        let settings = SettingService::get(conn).await?;
        let enable_headless = settings.enable_headless_browser;

        // Get previous check context first (before the new check)
        let context = AvailabilityService::get_product_check_context(conn, product_id).await?;

        // Perform the check
        let check = AvailabilityService::check_product(conn, product_id, enable_headless).await?;

        // Get daily price comparison
        let daily_comparison =
            AvailabilityService::get_daily_price_comparison(conn, product_id).await?;

        // Determine if back in stock
        let is_back_in_stock =
            AvailabilityService::is_back_in_stock(&context.previous_status, &check.status);

        // Build notification if applicable
        let notification = if settings.enable_notifications && is_back_in_stock {
            NotificationService::build_single_notification(conn, product_id, &settings, true)
                .await?
        } else {
            None
        };

        Ok(CheckResultWithNotification {
            check,
            daily_comparison,
            notification,
        })
    }

    /// Check all products with progress events and bulk notification.
    ///
    /// Emits "bulk-check-progress" events for each product checked.
    pub async fn check_all_products_with_notification(
        conn: &DatabaseConnection,
        app: &AppHandle,
    ) -> Result<TauriBulkCheckResult, AppError> {
        let settings = SettingService::get(conn).await?;
        let enable_headless = settings.enable_headless_browser;

        // Get all products
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

        let mut results = Vec::with_capacity(total);

        for (index, product) in products.iter().enumerate() {
            if index > 0 {
                tokio::time::sleep(Duration::from_millis(RATE_LIMIT_BETWEEN_CHECKS_MS)).await;
            }

            let result = check_single_product(conn, product, enable_headless).await;

            let _ = app.emit(
                "bulk-check-progress",
                &BulkCheckProgressEvent {
                    product_id: product.id.to_string(),
                    status: result.status.clone(),
                    current: index + 1,
                    total,
                },
            );

            results.push(result);
        }

        let successful = results.iter().filter(|r| r.error.is_none()).count();
        let failed = results.iter().filter(|r| r.error.is_some()).count();
        let back_in_stock_count = results.iter().filter(|r| r.is_back_in_stock).count();
        let price_drop_count = results.iter().filter(|r| r.is_price_drop).count();

        let summary = BulkCheckSummary {
            total,
            successful,
            failed,
            back_in_stock_count,
            price_drop_count,
            results: results.clone(),
        };

        // Build notification if enabled
        let notification = NotificationService::build_bulk_notification(
            &settings,
            back_in_stock_count,
            price_drop_count,
            &results,
        );

        Ok(TauriBulkCheckResult {
            summary,
            notification,
        })
    }
}

/// Check a single product, returning a BulkCheckResult.
/// Errors are captured in the result rather than propagated.
async fn check_single_product(
    conn: &DatabaseConnection,
    product: &ProductModel,
    enable_headless: bool,
) -> BulkCheckResult {
    let context = match AvailabilityService::get_product_check_context(conn, product.id).await {
        Ok(ctx) => ctx,
        Err(e) => return BulkCheckResult::error_for_product(product, e.to_string()),
    };

    let check = match AvailabilityService::check_product(conn, product.id, enable_headless).await {
        Ok(c) => c,
        Err(e) => return BulkCheckResult::error_for_product(product, e.to_string()),
    };

    let daily_comparison = AvailabilityService::get_daily_price_comparison(conn, product.id)
        .await
        .unwrap_or_default();

    let is_back_in_stock =
        AvailabilityService::is_back_in_stock(&context.previous_status, &check.status);
    let is_price_drop = AvailabilityService::is_price_drop(
        daily_comparison.yesterday_average_cents,
        daily_comparison.today_average_cents,
    );

    BulkCheckResult {
        product_id: product.id.to_string(),
        product_name: product.name.clone(),
        status: check.status.clone(),
        previous_status: context.previous_status,
        is_back_in_stock,
        price_cents: check.price_cents,
        price_currency: check.price_currency,
        today_average_price_cents: daily_comparison.today_average_cents,
        yesterday_average_price_cents: daily_comparison.yesterday_average_cents,
        is_price_drop,
        error: None,
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
