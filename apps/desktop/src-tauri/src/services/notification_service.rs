use sea_orm::DatabaseConnection;
use serde::Serialize;
use uuid::Uuid;

use crate::entities::setting::Model as SettingModel;
use crate::error::AppError;
use crate::repositories::ProductRepository;
use crate::services::availability_service::BulkCheckResult;

/// Data needed to display a notification (Tauri-agnostic)
#[derive(Debug, Clone, Serialize)]
pub struct NotificationData {
    pub title: String,
    pub body: String,
}

/// Service layer for notification building business logic
///
/// This service is responsible for composing notification content based on
/// availability check results. It does not send notifications directly -
/// that responsibility belongs to the Tauri command layer.
pub struct NotificationService;

impl NotificationService {
    /// Build notification data for a single product check using pre-fetched settings
    ///
    /// Returns `Some(NotificationData)` if:
    /// - The product transitioned to "back in stock"
    /// - Notifications are enabled in settings
    ///
    /// This is the preferred method when settings have already been fetched
    /// by the orchestrator, avoiding duplicate database queries.
    pub async fn build_single_notification(
        conn: &DatabaseConnection,
        product_id: Uuid,
        settings: &SettingModel,
        is_back_in_stock: bool,
    ) -> Result<Option<NotificationData>, AppError> {
        if !is_back_in_stock {
            return Ok(None);
        }

        if !settings.enable_notifications {
            return Ok(None);
        }

        // Get product name for the notification
        let product = ProductRepository::find_by_id(conn, product_id).await?;
        let Some(product) = product else {
            return Ok(None);
        };

        Ok(Some(Self::compose_single_back_in_stock(&product.name)))
    }

    /// Build notification data for a single product that is back in stock
    fn compose_single_back_in_stock(product_name: &str) -> NotificationData {
        NotificationData {
            title: "Product Back in Stock!".to_string(),
            body: format!("{} is now available!", product_name),
        }
    }

    /// Build notification data for a bulk check using pre-fetched settings
    ///
    /// Returns `Some(NotificationData)` if:
    /// - There are products back in stock OR price drops
    /// - Notifications are enabled in settings
    ///
    /// This is the preferred method when settings have already been fetched
    /// by the orchestrator, avoiding duplicate database queries.
    pub fn build_bulk_notification(
        settings: &SettingModel,
        back_in_stock_count: usize,
        price_drop_count: usize,
        results: &[BulkCheckResult],
    ) -> Option<NotificationData> {
        if back_in_stock_count == 0 && price_drop_count == 0 {
            return None;
        }

        if !settings.enable_notifications {
            return None;
        }

        let back_in_stock = Self::collect_product_names(results, |r| r.is_back_in_stock);
        let price_drops = Self::collect_product_names(results, |r| r.is_price_drop);

        let body = Self::compose_notification_body(&back_in_stock, &price_drops);
        let title = Self::compose_notification_title(&back_in_stock, &price_drops);

        Some(NotificationData { title, body })
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
    pub fn format_back_in_stock_message(products: &[&str]) -> String {
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
    pub fn format_price_drop_message(products: &[&str]) -> String {
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
    pub fn compose_notification_title(back_in_stock: &[&str], price_drops: &[&str]) -> String {
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

    /// Tests for NotificationData struct
    mod notification_data_tests {
        use super::*;

        #[test]
        fn test_clone() {
            let notification = NotificationData {
                title: "Test Title".to_string(),
                body: "Test Body".to_string(),
            };
            let cloned = notification.clone();
            assert_eq!(notification.title, cloned.title);
            assert_eq!(notification.body, cloned.body);
        }

        #[test]
        fn test_debug() {
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
        fn test_serialize() {
            let notification = NotificationData {
                title: "Product Back!".to_string(),
                body: "Your product is available".to_string(),
            };
            let json = serde_json::to_string(&notification).unwrap();
            assert!(json.contains("Product Back!"));
            assert!(json.contains("Your product is available"));
        }
    }

    /// Tests for notification composition helper methods
    mod notification_composition_tests {
        use super::*;

        #[test]
        fn test_format_back_in_stock_message_single_product() {
            let products = vec!["Product A"];
            let message = NotificationService::format_back_in_stock_message(&products);
            assert_eq!(message, "Product A is back in stock!");
        }

        #[test]
        fn test_format_back_in_stock_message_multiple_products() {
            let products = vec!["Product A", "Product B", "Product C"];
            let message = NotificationService::format_back_in_stock_message(&products);
            assert_eq!(
                message,
                "3 products back in stock: Product A, Product B, Product C"
            );
        }

        #[test]
        fn test_format_price_drop_message_single_product() {
            let products = vec!["Product A"];
            let message = NotificationService::format_price_drop_message(&products);
            assert_eq!(message, "Product A has a price drop!");
        }

        #[test]
        fn test_format_price_drop_message_multiple_products() {
            let products = vec!["Product A", "Product B"];
            let message = NotificationService::format_price_drop_message(&products);
            assert_eq!(message, "2 products have price drops: Product A, Product B");
        }

        #[test]
        fn test_compose_notification_title_both_events() {
            let back_in_stock = vec!["Product A"];
            let price_drops = vec!["Product B"];
            let title =
                NotificationService::compose_notification_title(&back_in_stock, &price_drops);
            assert_eq!(title, "Stock & Price Updates!");
        }

        #[test]
        fn test_compose_notification_title_only_back_in_stock() {
            let back_in_stock = vec!["Product A"];
            let price_drops: Vec<&str> = vec![];
            let title =
                NotificationService::compose_notification_title(&back_in_stock, &price_drops);
            assert_eq!(title, "Products Back in Stock!");
        }

        #[test]
        fn test_compose_notification_title_only_price_drops() {
            let back_in_stock: Vec<&str> = vec![];
            let price_drops = vec!["Product B"];
            let title =
                NotificationService::compose_notification_title(&back_in_stock, &price_drops);
            assert_eq!(title, "Price Drops!");
        }

        #[test]
        fn test_compose_notification_body_both_events() {
            let back_in_stock = vec!["Product A"];
            let price_drops = vec!["Product B"];
            let body = NotificationService::compose_notification_body(&back_in_stock, &price_drops);
            assert_eq!(
                body,
                "Product A is back in stock! Product B has a price drop!"
            );
        }

        #[test]
        fn test_compose_notification_body_only_back_in_stock() {
            let back_in_stock = vec!["Product A", "Product B"];
            let price_drops: Vec<&str> = vec![];
            let body = NotificationService::compose_notification_body(&back_in_stock, &price_drops);
            assert_eq!(body, "2 products back in stock: Product A, Product B");
        }

        #[test]
        fn test_compose_notification_body_only_price_drops() {
            let back_in_stock: Vec<&str> = vec![];
            let price_drops = vec!["Product C"];
            let body = NotificationService::compose_notification_body(&back_in_stock, &price_drops);
            assert_eq!(body, "Product C has a price drop!");
        }

        #[test]
        fn test_compose_single_back_in_stock() {
            let notification = NotificationService::compose_single_back_in_stock("Test Product");
            assert_eq!(notification.title, "Product Back in Stock!");
            assert_eq!(notification.body, "Test Product is now available!");
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

            let back_in_stock =
                NotificationService::collect_product_names(&results, |r| r.is_back_in_stock);
            assert_eq!(back_in_stock, vec!["Product A"]);

            let price_drops =
                NotificationService::collect_product_names(&results, |r| r.is_price_drop);
            assert_eq!(price_drops, vec!["Product B"]);
        }
    }

    /// Tests for build_bulk_notification
    mod build_bulk_notification_tests {
        use super::*;
        use crate::entities::setting::Model as SettingModel;

        fn create_test_settings(enable_notifications: bool) -> SettingModel {
            SettingModel {
                enable_notifications,
                ..Default::default()
            }
        }

        #[test]
        fn test_no_notification_when_no_events() {
            let settings = create_test_settings(true);
            let results: Vec<BulkCheckResult> = vec![];

            let notification =
                NotificationService::build_bulk_notification(&settings, 0, 0, &results);

            assert!(notification.is_none());
        }

        #[test]
        fn test_no_notification_when_disabled() {
            let settings = create_test_settings(false);
            let results = vec![BulkCheckResult {
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
            }];

            let notification =
                NotificationService::build_bulk_notification(&settings, 1, 0, &results);

            assert!(notification.is_none());
        }

        #[test]
        fn test_notification_with_back_in_stock() {
            let settings = create_test_settings(true);
            let results = vec![BulkCheckResult {
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
            }];

            let notification =
                NotificationService::build_bulk_notification(&settings, 1, 0, &results);

            assert!(notification.is_some());
            let notification = notification.unwrap();
            assert_eq!(notification.title, "Products Back in Stock!");
            assert_eq!(notification.body, "Product A is back in stock!");
        }

        #[test]
        fn test_notification_with_price_drop() {
            let settings = create_test_settings(true);
            let results = vec![BulkCheckResult {
                product_id: "1".to_string(),
                product_name: "Product A".to_string(),
                status: "in_stock".to_string(),
                previous_status: Some("in_stock".to_string()),
                is_back_in_stock: false,
                price_cents: Some(5000),
                price_currency: Some("USD".to_string()),
                previous_price_cents: Some(7000),
                is_price_drop: true,
                error: None,
            }];

            let notification =
                NotificationService::build_bulk_notification(&settings, 0, 1, &results);

            assert!(notification.is_some());
            let notification = notification.unwrap();
            assert_eq!(notification.title, "Price Drops!");
            assert_eq!(notification.body, "Product A has a price drop!");
        }

        #[test]
        fn test_notification_with_both_events() {
            let settings = create_test_settings(true);
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
            ];

            let notification =
                NotificationService::build_bulk_notification(&settings, 1, 1, &results);

            assert!(notification.is_some());
            let notification = notification.unwrap();
            assert_eq!(notification.title, "Stock & Price Updates!");
            assert!(notification.body.contains("Product A is back in stock!"));
            assert!(notification.body.contains("Product B has a price drop!"));
        }
    }
}
