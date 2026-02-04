use std::time::Duration;

use sea_orm::DatabaseConnection;
use serde::Serialize;
use uuid::Uuid;

use crate::entities::prelude::AvailabilityCheckModel;
use crate::error::AppError;
use crate::repositories::{AvailabilityCheckRepository, ProductRepository};
use crate::services::{ScraperService, SettingService};

/// Result of a single product availability check in a bulk operation
#[derive(Debug, Serialize)]
pub struct BulkCheckResult {
    pub product_id: String,
    pub product_name: String,
    pub status: String,
    pub previous_status: Option<String>,
    pub is_back_in_stock: bool,
    pub error: Option<String>,
}

/// Summary of a bulk check operation
#[derive(Debug, Serialize)]
pub struct BulkCheckSummary {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub back_in_stock_count: usize,
    pub results: Vec<BulkCheckResult>,
}

/// Data needed to display a notification (Tauri-agnostic)
#[derive(Debug, Clone, Serialize)]
pub struct NotificationData {
    pub title: String,
    pub body: String,
}

/// Result of an availability check with optional notification data
#[derive(Debug, Serialize)]
pub struct CheckResultWithNotification {
    pub check: AvailabilityCheckModel,
    pub notification: Option<NotificationData>,
}

/// Result of a bulk check with optional notification data
#[derive(Debug, Serialize)]
pub struct BulkCheckResultWithNotification {
    pub summary: BulkCheckSummary,
    pub notification: Option<NotificationData>,
}

/// Service layer for availability checking business logic
pub struct AvailabilityService;

impl AvailabilityService {
    /// Check the availability of a product by its ID
    ///
    /// Fetches the product's URL, scrapes the page for availability info,
    /// and stores the result in the database.
    pub async fn check_product(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<AvailabilityCheckModel, AppError> {
        // Get the product to get its URL
        let product = ProductRepository::find_by_id(conn, product_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", product_id)))?;

        // Attempt to check availability
        let check_id = Uuid::new_v4();
        let result = ScraperService::check_availability(&product.url).await;

        match result {
            Ok(scraping_result) => {
                // Success - store the result
                AvailabilityCheckRepository::create(
                    conn,
                    check_id,
                    product_id,
                    scraping_result.status.as_str(),
                    scraping_result.raw_availability,
                    None,
                )
                .await
            }
            Err(e) => {
                // Error - store the error but still create a record
                AvailabilityCheckRepository::create(
                    conn,
                    check_id,
                    product_id,
                    "unknown",
                    None,
                    Some(e.to_string()),
                )
                .await
            }
        }
    }

    /// Check availability for all products with rate limiting
    ///
    /// Returns a summary including which products are back in stock.
    pub async fn check_all_products(
        conn: &DatabaseConnection,
    ) -> Result<BulkCheckSummary, AppError> {
        let products = ProductRepository::find_all(conn).await?;
        let total = products.len();
        let mut results = Vec::with_capacity(total);
        let mut successful = 0;
        let mut failed = 0;
        let mut back_in_stock_count = 0;

        for product in products {
            // Rate limiting: wait 500ms between requests
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Get previous status before checking
            let previous_check = Self::get_latest(conn, product.id).await?;
            let previous_status = previous_check.map(|c| c.status);

            // Check availability
            let check_result = Self::check_product(conn, product.id).await;

            let (status, error, is_back_in_stock) = match check_result {
                Ok(check) => {
                    if check.error_message.is_some() {
                        // Scraper failed but record was created
                        failed += 1;
                        (check.status, check.error_message, false)
                    } else {
                        // True success
                        successful += 1;
                        let back_in_stock = Self::is_back_in_stock(&previous_status, &check.status);
                        if back_in_stock {
                            back_in_stock_count += 1;
                        }
                        (check.status, None, back_in_stock)
                    }
                }
                Err(e) => {
                    // Database/infrastructure error
                    failed += 1;
                    ("unknown".to_string(), Some(e.to_string()), false)
                }
            };

            results.push(BulkCheckResult {
                product_id: product.id.to_string(),
                product_name: product.name,
                status,
                previous_status,
                is_back_in_stock,
                error,
            });
        }

        Ok(BulkCheckSummary {
            total,
            successful,
            failed,
            back_in_stock_count,
            results,
        })
    }

    /// Check if a product transitioned to back in stock
    pub fn is_back_in_stock(previous_status: &Option<String>, new_status: &str) -> bool {
        match previous_status {
            Some(prev) => prev != "in_stock" && new_status == "in_stock",
            None => false, // First check, not a transition
        }
    }

    /// Get the latest availability check for a product
    pub async fn get_latest(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<Option<AvailabilityCheckModel>, AppError> {
        AvailabilityCheckRepository::find_latest_for_product(conn, product_id).await
    }

    /// Get the availability check history for a product
    pub async fn get_history(
        conn: &DatabaseConnection,
        product_id: Uuid,
        limit: Option<u64>,
    ) -> Result<Vec<AvailabilityCheckModel>, AppError> {
        AvailabilityCheckRepository::find_all_for_product(conn, product_id, limit).await
    }

    /// Check product availability and return notification data if applicable
    ///
    /// Encapsulates all business logic for:
    /// - Getting previous status
    /// - Checking availability
    /// - Determining if notification should be sent (based on back-in-stock + settings)
    /// - Composing notification title/body
    pub async fn check_product_with_notification(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<CheckResultWithNotification, AppError> {
        // Get previous status before checking
        let previous_check = Self::get_latest(conn, product_id).await?;
        let previous_status = previous_check.map(|c| c.status);

        // Perform the check
        let check = Self::check_product(conn, product_id).await?;

        // Determine if we should send a notification
        let notification =
            Self::build_single_notification(conn, product_id, &previous_status, &check.status)
                .await?;

        Ok(CheckResultWithNotification {
            check,
            notification,
        })
    }

    /// Check all products and return notification data if applicable
    ///
    /// Encapsulates all business logic for bulk checks including notification composition.
    pub async fn check_all_products_with_notification(
        conn: &DatabaseConnection,
    ) -> Result<BulkCheckResultWithNotification, AppError> {
        let summary = Self::check_all_products(conn).await?;

        // Determine if we should send a notification
        let notification = Self::build_bulk_notification(conn, &summary).await?;

        Ok(BulkCheckResultWithNotification {
            summary,
            notification,
        })
    }

    /// Build notification data for a single product check if applicable
    async fn build_single_notification(
        conn: &DatabaseConnection,
        product_id: Uuid,
        previous_status: &Option<String>,
        new_status: &str,
    ) -> Result<Option<NotificationData>, AppError> {
        // Check if product is back in stock
        if !Self::is_back_in_stock(previous_status, new_status) {
            return Ok(None);
        }

        // Check if notifications are enabled
        let settings = SettingService::get(conn).await?;
        if !settings.enable_notifications {
            return Ok(None);
        }

        // Get product name for the notification
        let product = ProductRepository::find_by_id(conn, product_id).await?;
        let Some(product) = product else {
            return Ok(None);
        };

        Ok(Some(NotificationData {
            title: "Product Back in Stock!".to_string(),
            body: format!("{} is now available!", product.name),
        }))
    }

    /// Build notification data for a bulk check if applicable
    async fn build_bulk_notification(
        conn: &DatabaseConnection,
        summary: &BulkCheckSummary,
    ) -> Result<Option<NotificationData>, AppError> {
        // No products back in stock
        if summary.back_in_stock_count == 0 {
            return Ok(None);
        }

        // Check if notifications are enabled
        let settings = SettingService::get(conn).await?;
        if !settings.enable_notifications {
            return Ok(None);
        }

        // Collect product names that are back in stock
        let back_in_stock_products: Vec<&str> = summary
            .results
            .iter()
            .filter(|r| r.is_back_in_stock)
            .map(|r| r.product_name.as_str())
            .collect();

        let body = if back_in_stock_products.len() == 1 {
            format!("{} is back in stock!", back_in_stock_products[0])
        } else {
            format!(
                "{} products are back in stock: {}",
                back_in_stock_products.len(),
                back_in_stock_products.join(", ")
            )
        };

        Ok(Some(NotificationData {
            title: "Products Back in Stock!".to_string(),
            body,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_product, setup_availability_db};

    #[tokio::test]
    async fn test_get_latest_none() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product(&conn, "https://example.com").await;

        let latest = AvailabilityService::get_latest(&conn, product_id)
            .await
            .unwrap();

        assert!(latest.is_none());
    }

    #[tokio::test]
    async fn test_get_history_empty() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product(&conn, "https://example.com").await;

        let history = AvailabilityService::get_history(&conn, product_id, None)
            .await
            .unwrap();

        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_get_history_with_limit() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product(&conn, "https://example.com").await;

        // Create some check records directly
        for _ in 0..5 {
            AvailabilityCheckRepository::create(
                &conn,
                Uuid::new_v4(),
                product_id,
                "in_stock",
                None,
                None,
            )
            .await
            .unwrap();
        }

        let history = AvailabilityService::get_history(&conn, product_id, Some(3))
            .await
            .unwrap();

        assert_eq!(history.len(), 3);
    }

    #[tokio::test]
    async fn test_check_product_not_found() {
        let conn = setup_availability_db().await;
        let fake_id = Uuid::new_v4();

        let result = AvailabilityService::check_product(&conn, fake_id).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn test_is_back_in_stock_from_out_of_stock() {
        let previous = Some("out_of_stock".to_string());
        let is_back = AvailabilityService::is_back_in_stock(&previous, "in_stock");
        assert!(is_back);
    }

    #[test]
    fn test_is_back_in_stock_from_back_order() {
        let previous = Some("back_order".to_string());
        let is_back = AvailabilityService::is_back_in_stock(&previous, "in_stock");
        assert!(is_back);
    }

    #[test]
    fn test_is_back_in_stock_from_unknown() {
        let previous = Some("unknown".to_string());
        let is_back = AvailabilityService::is_back_in_stock(&previous, "in_stock");
        assert!(is_back);
    }

    #[test]
    fn test_is_back_in_stock_already_in_stock() {
        let previous = Some("in_stock".to_string());
        let is_back = AvailabilityService::is_back_in_stock(&previous, "in_stock");
        assert!(!is_back); // Not a transition
    }

    #[test]
    fn test_is_back_in_stock_still_out_of_stock() {
        let previous = Some("out_of_stock".to_string());
        let is_back = AvailabilityService::is_back_in_stock(&previous, "out_of_stock");
        assert!(!is_back);
    }

    #[test]
    fn test_is_back_in_stock_no_previous() {
        let previous: Option<String> = None;
        let is_back = AvailabilityService::is_back_in_stock(&previous, "in_stock");
        assert!(!is_back); // First check, not a transition
    }
}
