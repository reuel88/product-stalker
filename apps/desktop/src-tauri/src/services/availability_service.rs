use std::time::Duration;

use sea_orm::DatabaseConnection;
use serde::Serialize;
use uuid::Uuid;

use crate::entities::availability_check::AvailabilityStatus;
use crate::entities::prelude::AvailabilityCheckModel;
use crate::error::AppError;
use crate::repositories::{AvailabilityCheckRepository, CreateCheckParams, ProductRepository};
use crate::services::{ScraperService, SettingService};

/// Result of a single product availability check in a bulk operation
#[derive(Debug, Serialize)]
pub struct BulkCheckResult {
    pub product_id: String,
    pub product_name: String,
    pub status: String,
    pub previous_status: Option<String>,
    pub is_back_in_stock: bool,
    pub price_cents: Option<i64>,
    pub price_currency: Option<String>,
    pub previous_price_cents: Option<i64>,
    pub is_price_drop: bool,
    pub error: Option<String>,
}

/// Summary of a bulk check operation
#[derive(Debug, Serialize)]
pub struct BulkCheckSummary {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub back_in_stock_count: usize,
    pub price_drop_count: usize,
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

/// Result of processing a single availability check
struct CheckProcessingResult {
    status: String,
    price_cents: Option<i64>,
    price_currency: Option<String>,
    error: Option<String>,
    is_back_in_stock: bool,
    is_price_drop: bool,
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

        // Get the headless browser setting
        let settings = SettingService::get(conn).await?;
        let enable_headless = settings.enable_headless_browser;

        // Attempt to check availability
        let check_id = Uuid::new_v4();
        let result =
            ScraperService::check_availability_with_headless(&product.url, enable_headless).await;

        match result {
            Ok(scraping_result) => {
                // Success - store the result with price info
                AvailabilityCheckRepository::create(
                    conn,
                    check_id,
                    product_id,
                    CreateCheckParams {
                        status: scraping_result.status.as_str().to_string(),
                        raw_availability: scraping_result.raw_availability,
                        error_message: None,
                        price_cents: scraping_result.price.price_cents,
                        price_currency: scraping_result.price.price_currency,
                        raw_price: scraping_result.price.raw_price,
                    },
                )
                .await
            }
            Err(e) => {
                // Error - store the error but still create a record
                AvailabilityCheckRepository::create(
                    conn,
                    check_id,
                    product_id,
                    CreateCheckParams {
                        status: AvailabilityStatus::Unknown.as_str().to_string(),
                        error_message: Some(e.to_string()),
                        ..Default::default()
                    },
                )
                .await
            }
        }
    }

    /// Process the result of an availability check into a structured result
    fn process_check_result(
        check_result: Result<AvailabilityCheckModel, AppError>,
        previous_status: &Option<String>,
        previous_price_cents: Option<i64>,
    ) -> CheckProcessingResult {
        match check_result {
            Ok(check) => {
                if check.error_message.is_some() {
                    // Scraper failed but record was created
                    CheckProcessingResult {
                        status: check.status,
                        price_cents: check.price_cents,
                        price_currency: check.price_currency,
                        error: check.error_message,
                        is_back_in_stock: false,
                        is_price_drop: false,
                    }
                } else {
                    // True success
                    let is_back_in_stock = Self::is_back_in_stock(previous_status, &check.status);
                    let is_price_drop =
                        Self::is_price_drop(previous_price_cents, check.price_cents);
                    CheckProcessingResult {
                        status: check.status,
                        price_cents: check.price_cents,
                        price_currency: check.price_currency,
                        error: None,
                        is_back_in_stock,
                        is_price_drop,
                    }
                }
            }
            Err(e) => {
                // Database/infrastructure error
                CheckProcessingResult {
                    status: AvailabilityStatus::Unknown.as_str().to_string(),
                    price_cents: None,
                    price_currency: None,
                    error: Some(e.to_string()),
                    is_back_in_stock: false,
                    is_price_drop: false,
                }
            }
        }
    }

    /// Check availability for all products with rate limiting
    ///
    /// Returns a summary including which products are back in stock and price drops.
    pub async fn check_all_products(
        conn: &DatabaseConnection,
    ) -> Result<BulkCheckSummary, AppError> {
        let products = ProductRepository::find_all(conn).await?;
        let total = products.len();
        let mut results = Vec::with_capacity(total);
        let mut successful = 0;
        let mut failed = 0;
        let mut back_in_stock_count = 0;
        let mut price_drop_count = 0;

        for product in products {
            // Rate limiting: wait 500ms between requests
            tokio::time::sleep(Duration::from_millis(500)).await;

            // Get previous status and price before checking
            let previous_check = Self::get_latest(conn, product.id).await?;
            let previous_status = previous_check.as_ref().map(|c| c.status.clone());
            let previous_price_cents =
                AvailabilityCheckRepository::find_previous_price(conn, product.id).await?;

            // Check availability and process result
            let check_result = Self::check_product(conn, product.id).await;
            let result =
                Self::process_check_result(check_result, &previous_status, previous_price_cents);

            // Update counters
            if result.error.is_some() {
                failed += 1;
            } else {
                successful += 1;
                if result.is_back_in_stock {
                    back_in_stock_count += 1;
                }
                if result.is_price_drop {
                    price_drop_count += 1;
                }
            }

            results.push(BulkCheckResult {
                product_id: product.id.to_string(),
                product_name: product.name,
                status: result.status,
                previous_status,
                is_back_in_stock: result.is_back_in_stock,
                price_cents: result.price_cents,
                price_currency: result.price_currency,
                previous_price_cents,
                is_price_drop: result.is_price_drop,
                error: result.error,
            });
        }

        Ok(BulkCheckSummary {
            total,
            successful,
            failed,
            back_in_stock_count,
            price_drop_count,
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

    /// Check if the price dropped from previous check
    pub fn is_price_drop(previous_price: Option<i64>, new_price: Option<i64>) -> bool {
        match (previous_price, new_price) {
            (Some(prev), Some(new)) => new < prev,
            _ => false, // No price drop if either is None
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
        if summary.back_in_stock_count == 0 && summary.price_drop_count == 0 {
            return Ok(None);
        }

        let settings = SettingService::get(conn).await?;
        if !settings.enable_notifications {
            return Ok(None);
        }

        let back_in_stock = Self::collect_product_names(&summary.results, |r| r.is_back_in_stock);
        let price_drops = Self::collect_product_names(&summary.results, |r| r.is_price_drop);

        let body = Self::compose_notification_body(&back_in_stock, &price_drops);
        let title = Self::compose_notification_title(&back_in_stock, &price_drops);

        Ok(Some(NotificationData { title, body }))
    }

    /// Collect product names from results based on a filter predicate
    fn collect_product_names<F>(results: &[BulkCheckResult], predicate: F) -> Vec<&str>
    where
        F: Fn(&BulkCheckResult) -> bool,
    {
        results
            .iter()
            .filter(|r| predicate(r))
            .map(|r| r.product_name.as_str())
            .collect()
    }

    /// Compose the notification body from back-in-stock and price drop product lists
    fn compose_notification_body(back_in_stock: &[&str], price_drops: &[&str]) -> String {
        let mut parts = Vec::new();

        if !back_in_stock.is_empty() {
            parts.push(Self::format_back_in_stock_message(back_in_stock));
        }

        if !price_drops.is_empty() {
            parts.push(Self::format_price_drop_message(price_drops));
        }

        parts.join(" ")
    }

    /// Format the back-in-stock portion of a notification message
    fn format_back_in_stock_message(products: &[&str]) -> String {
        if products.len() == 1 {
            format!("{} is back in stock!", products[0])
        } else {
            format!(
                "{} products back in stock: {}",
                products.len(),
                products.join(", ")
            )
        }
    }

    /// Format the price drop portion of a notification message
    fn format_price_drop_message(products: &[&str]) -> String {
        if products.len() == 1 {
            format!("{} has a price drop!", products[0])
        } else {
            format!(
                "{} products have price drops: {}",
                products.len(),
                products.join(", ")
            )
        }
    }

    /// Compose the notification title based on what events occurred
    fn compose_notification_title(back_in_stock: &[&str], price_drops: &[&str]) -> String {
        match (!back_in_stock.is_empty(), !price_drops.is_empty()) {
            (true, true) => "Stock & Price Updates!".to_string(),
            (true, false) => "Products Back in Stock!".to_string(),
            (false, true) => "Price Drops!".to_string(),
            (false, false) => String::new(), // Should not happen given earlier checks
        }
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
                CreateCheckParams {
                    status: "in_stock".to_string(),
                    ..Default::default()
                },
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

    #[test]
    fn test_is_back_in_stock_to_out_of_stock() {
        let previous = Some("in_stock".to_string());
        let is_back = AvailabilityService::is_back_in_stock(&previous, "out_of_stock");
        assert!(!is_back); // Going out of stock is not "back in stock"
    }

    #[test]
    fn test_is_back_in_stock_to_back_order() {
        let previous = Some("in_stock".to_string());
        let is_back = AvailabilityService::is_back_in_stock(&previous, "back_order");
        assert!(!is_back);
    }

    #[test]
    fn test_is_back_in_stock_to_unknown() {
        let previous = Some("in_stock".to_string());
        let is_back = AvailabilityService::is_back_in_stock(&previous, "unknown");
        assert!(!is_back);
    }

    // NotificationData tests

    #[test]
    fn test_notification_data_clone() {
        let notification = NotificationData {
            title: "Test Title".to_string(),
            body: "Test Body".to_string(),
        };
        let cloned = notification.clone();
        assert_eq!(notification.title, cloned.title);
        assert_eq!(notification.body, cloned.body);
    }

    #[test]
    fn test_notification_data_debug() {
        let notification = NotificationData {
            title: "Title".to_string(),
            body: "Body".to_string(),
        };
        let debug_str = format!("{:?}", notification);
        assert!(debug_str.contains("NotificationData"));
        assert!(debug_str.contains("Title"));
        assert!(debug_str.contains("Body"));
    }

    #[test]
    fn test_notification_data_serialize() {
        let notification = NotificationData {
            title: "Product Back!".to_string(),
            body: "Your product is available".to_string(),
        };
        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("Product Back!"));
        assert!(json.contains("Your product is available"));
    }

    // BulkCheckResult tests

    #[test]
    fn test_bulk_check_result_serialize() {
        let result = BulkCheckResult {
            product_id: "test-id-123".to_string(),
            product_name: "Test Product".to_string(),
            status: "in_stock".to_string(),
            previous_status: Some("out_of_stock".to_string()),
            is_back_in_stock: true,
            price_cents: Some(78900),
            price_currency: Some("USD".to_string()),
            previous_price_cents: Some(89900),
            is_price_drop: true,
            error: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test-id-123"));
        assert!(json.contains("Test Product"));
        assert!(json.contains("in_stock"));
        assert!(json.contains("out_of_stock"));
        assert!(json.contains("78900"));
    }

    #[test]
    fn test_bulk_check_result_with_error() {
        let result = BulkCheckResult {
            product_id: "error-id".to_string(),
            product_name: "Error Product".to_string(),
            status: "unknown".to_string(),
            previous_status: None,
            is_back_in_stock: false,
            price_cents: None,
            price_currency: None,
            previous_price_cents: None,
            is_price_drop: false,
            error: Some("Failed to fetch".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Failed to fetch"));
        assert!(json.contains("unknown"));
    }

    #[test]
    fn test_bulk_check_result_debug() {
        let result = BulkCheckResult {
            product_id: "id".to_string(),
            product_name: "name".to_string(),
            status: "in_stock".to_string(),
            previous_status: None,
            is_back_in_stock: false,
            price_cents: None,
            price_currency: None,
            previous_price_cents: None,
            is_price_drop: false,
            error: None,
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("BulkCheckResult"));
    }

    // BulkCheckSummary tests

    #[test]
    fn test_bulk_check_summary_serialize() {
        let summary = BulkCheckSummary {
            total: 10,
            successful: 8,
            failed: 2,
            back_in_stock_count: 3,
            price_drop_count: 2,
            results: vec![],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("10"));
        assert!(json.contains("\"successful\":8"));
        assert!(json.contains("\"failed\":2"));
        assert!(json.contains("\"back_in_stock_count\":3"));
        assert!(json.contains("\"price_drop_count\":2"));
    }

    #[test]
    fn test_bulk_check_summary_with_results() {
        let result = BulkCheckResult {
            product_id: "p1".to_string(),
            product_name: "Product 1".to_string(),
            status: "in_stock".to_string(),
            previous_status: Some("out_of_stock".to_string()),
            is_back_in_stock: true,
            price_cents: Some(78900),
            price_currency: Some("USD".to_string()),
            previous_price_cents: None,
            is_price_drop: false,
            error: None,
        };
        let summary = BulkCheckSummary {
            total: 1,
            successful: 1,
            failed: 0,
            back_in_stock_count: 1,
            price_drop_count: 0,
            results: vec![result],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("Product 1"));
        assert!(json.contains("p1"));
    }

    #[test]
    fn test_bulk_check_summary_debug() {
        let summary = BulkCheckSummary {
            total: 5,
            successful: 3,
            failed: 2,
            back_in_stock_count: 1,
            price_drop_count: 0,
            results: vec![],
        };
        let debug_str = format!("{:?}", summary);
        assert!(debug_str.contains("BulkCheckSummary"));
    }

    // Price drop tests

    #[test]
    fn test_is_price_drop_from_higher() {
        assert!(AvailabilityService::is_price_drop(Some(10000), Some(8000)));
    }

    #[test]
    fn test_is_price_drop_same_price() {
        assert!(!AvailabilityService::is_price_drop(
            Some(10000),
            Some(10000)
        ));
    }

    #[test]
    fn test_is_price_drop_price_increase() {
        assert!(!AvailabilityService::is_price_drop(Some(8000), Some(10000)));
    }

    #[test]
    fn test_is_price_drop_no_previous() {
        assert!(!AvailabilityService::is_price_drop(None, Some(10000)));
    }

    #[test]
    fn test_is_price_drop_no_new() {
        assert!(!AvailabilityService::is_price_drop(Some(10000), None));
    }

    #[test]
    fn test_is_price_drop_both_none() {
        assert!(!AvailabilityService::is_price_drop(None, None));
    }

    // Notification helper method tests

    #[test]
    fn test_format_back_in_stock_message_single_product() {
        let products = vec!["Product A"];
        let message = AvailabilityService::format_back_in_stock_message(&products);
        assert_eq!(message, "Product A is back in stock!");
    }

    #[test]
    fn test_format_back_in_stock_message_multiple_products() {
        let products = vec!["Product A", "Product B", "Product C"];
        let message = AvailabilityService::format_back_in_stock_message(&products);
        assert_eq!(message, "3 products back in stock: Product A, Product B, Product C");
    }

    #[test]
    fn test_format_price_drop_message_single_product() {
        let products = vec!["Product A"];
        let message = AvailabilityService::format_price_drop_message(&products);
        assert_eq!(message, "Product A has a price drop!");
    }

    #[test]
    fn test_format_price_drop_message_multiple_products() {
        let products = vec!["Product A", "Product B"];
        let message = AvailabilityService::format_price_drop_message(&products);
        assert_eq!(message, "2 products have price drops: Product A, Product B");
    }

    #[test]
    fn test_compose_notification_title_both_events() {
        let back_in_stock = vec!["Product A"];
        let price_drops = vec!["Product B"];
        let title = AvailabilityService::compose_notification_title(&back_in_stock, &price_drops);
        assert_eq!(title, "Stock & Price Updates!");
    }

    #[test]
    fn test_compose_notification_title_only_back_in_stock() {
        let back_in_stock = vec!["Product A"];
        let price_drops: Vec<&str> = vec![];
        let title = AvailabilityService::compose_notification_title(&back_in_stock, &price_drops);
        assert_eq!(title, "Products Back in Stock!");
    }

    #[test]
    fn test_compose_notification_title_only_price_drops() {
        let back_in_stock: Vec<&str> = vec![];
        let price_drops = vec!["Product B"];
        let title = AvailabilityService::compose_notification_title(&back_in_stock, &price_drops);
        assert_eq!(title, "Price Drops!");
    }

    #[test]
    fn test_compose_notification_body_both_events() {
        let back_in_stock = vec!["Product A"];
        let price_drops = vec!["Product B"];
        let body = AvailabilityService::compose_notification_body(&back_in_stock, &price_drops);
        assert_eq!(body, "Product A is back in stock! Product B has a price drop!");
    }

    #[test]
    fn test_compose_notification_body_only_back_in_stock() {
        let back_in_stock = vec!["Product A", "Product B"];
        let price_drops: Vec<&str> = vec![];
        let body = AvailabilityService::compose_notification_body(&back_in_stock, &price_drops);
        assert_eq!(body, "2 products back in stock: Product A, Product B");
    }

    #[test]
    fn test_compose_notification_body_only_price_drops() {
        let back_in_stock: Vec<&str> = vec![];
        let price_drops = vec!["Product C"];
        let body = AvailabilityService::compose_notification_body(&back_in_stock, &price_drops);
        assert_eq!(body, "Product C has a price drop!");
    }

    #[test]
    fn test_collect_product_names_filters_correctly() {
        let results = vec![
            BulkCheckResult {
                product_id: "1".to_string(),
                product_name: "Product A".to_string(),
                status: "in_stock".to_string(),
                previous_status: Some("out_of_stock".to_string()),
                is_back_in_stock: true,
                price_cents: None,
                price_currency: None,
                previous_price_cents: None,
                is_price_drop: false,
                error: None,
            },
            BulkCheckResult {
                product_id: "2".to_string(),
                product_name: "Product B".to_string(),
                status: "in_stock".to_string(),
                previous_status: Some("in_stock".to_string()),
                is_back_in_stock: false,
                price_cents: Some(5000),
                price_currency: Some("USD".to_string()),
                previous_price_cents: Some(7000),
                is_price_drop: true,
                error: None,
            },
            BulkCheckResult {
                product_id: "3".to_string(),
                product_name: "Product C".to_string(),
                status: "out_of_stock".to_string(),
                previous_status: None,
                is_back_in_stock: false,
                price_cents: None,
                price_currency: None,
                previous_price_cents: None,
                is_price_drop: false,
                error: None,
            },
        ];

        let back_in_stock = AvailabilityService::collect_product_names(&results, |r| r.is_back_in_stock);
        assert_eq!(back_in_stock, vec!["Product A"]);

        let price_drops = AvailabilityService::collect_product_names(&results, |r| r.is_price_drop);
        assert_eq!(price_drops, vec!["Product B"]);
    }

    // CheckResultWithNotification tests

    #[test]
    fn test_check_result_with_notification_serialize() {
        let check = AvailabilityCheckModel {
            id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            status: "in_stock".to_string(),
            raw_availability: Some("http://schema.org/InStock".to_string()),
            error_message: None,
            checked_at: chrono::Utc::now(),
            price_cents: Some(78900),
            price_currency: Some("USD".to_string()),
            raw_price: Some("789.00".to_string()),
        };
        let result = CheckResultWithNotification {
            check,
            notification: Some(NotificationData {
                title: "Back in Stock!".to_string(),
                body: "Product available".to_string(),
            }),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("in_stock"));
        assert!(json.contains("Back in Stock!"));
    }

    #[test]
    fn test_check_result_with_notification_none() {
        let check = AvailabilityCheckModel {
            id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            status: "out_of_stock".to_string(),
            raw_availability: None,
            error_message: None,
            checked_at: chrono::Utc::now(),
            price_cents: None,
            price_currency: None,
            raw_price: None,
        };
        let result = CheckResultWithNotification {
            check,
            notification: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("out_of_stock"));
        assert!(json.contains("null") || !json.contains("notification"));
    }

    #[test]
    fn test_check_result_with_notification_debug() {
        let check = AvailabilityCheckModel {
            id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            status: "in_stock".to_string(),
            raw_availability: None,
            error_message: None,
            checked_at: chrono::Utc::now(),
            price_cents: None,
            price_currency: None,
            raw_price: None,
        };
        let result = CheckResultWithNotification {
            check,
            notification: None,
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("CheckResultWithNotification"));
    }

    // BulkCheckResultWithNotification tests

    #[test]
    fn test_bulk_check_result_with_notification_serialize() {
        let summary = BulkCheckSummary {
            total: 2,
            successful: 2,
            failed: 0,
            back_in_stock_count: 1,
            price_drop_count: 0,
            results: vec![],
        };
        let result = BulkCheckResultWithNotification {
            summary,
            notification: Some(NotificationData {
                title: "Products Back!".to_string(),
                body: "1 product available".to_string(),
            }),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Products Back!"));
        assert!(json.contains("\"total\":2"));
    }

    #[test]
    fn test_bulk_check_result_with_notification_debug() {
        let summary = BulkCheckSummary {
            total: 0,
            successful: 0,
            failed: 0,
            back_in_stock_count: 0,
            price_drop_count: 0,
            results: vec![],
        };
        let result = BulkCheckResultWithNotification {
            summary,
            notification: None,
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("BulkCheckResultWithNotification"));
    }

    // Integration test for get_history without limit
    #[tokio::test]
    async fn test_get_history_without_limit() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product(&conn, "https://example.com").await;

        // Create 3 check records
        for i in 0..3 {
            AvailabilityCheckRepository::create(
                &conn,
                Uuid::new_v4(),
                product_id,
                CreateCheckParams {
                    status: if i % 2 == 0 {
                        "in_stock".to_string()
                    } else {
                        "out_of_stock".to_string()
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }

        let history = AvailabilityService::get_history(&conn, product_id, None)
            .await
            .unwrap();

        assert_eq!(history.len(), 3);
    }

    // Test get_latest with multiple checks
    #[tokio::test]
    async fn test_get_latest_with_multiple_checks() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product(&conn, "https://example.com").await;

        // Create multiple checks
        for _ in 0..3 {
            AvailabilityCheckRepository::create(
                &conn,
                Uuid::new_v4(),
                product_id,
                CreateCheckParams {
                    status: "in_stock".to_string(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }

        let latest = AvailabilityService::get_latest(&conn, product_id)
            .await
            .unwrap();

        assert!(latest.is_some());
        assert_eq!(latest.unwrap().status, "in_stock");
    }
}
