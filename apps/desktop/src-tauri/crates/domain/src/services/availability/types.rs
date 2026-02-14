//! Data types for availability checks and bulk operations.

use serde::Serialize;

use crate::entities::availability_check::AvailabilityStatus;
use crate::entities::prelude::{AvailabilityCheckModel, ProductModel};
use crate::services::currency;
use product_stalker_core::services::notification_helpers::NotificationData;

/// Result of a single product availability check in a bulk operation
#[derive(Debug, Clone, Default, Serialize)]
pub struct BulkCheckResult {
    pub product_id: String,
    pub product_name: String,
    pub product_retailer_id: Option<String>,
    pub url: Option<String>,
    pub status: AvailabilityStatus,
    pub previous_status: Option<AvailabilityStatus>,
    pub is_back_in_stock: bool,
    pub price_minor_units: Option<i64>,
    pub price_currency: Option<String>,
    pub currency_exponent: Option<u32>,
    pub today_average_price_minor_units: Option<i64>,
    pub yesterday_average_price_minor_units: Option<i64>,
    pub is_price_drop: bool,
    pub error: Option<String>,
}

/// Summary of a bulk check operation
#[derive(Debug, Clone, Serialize)]
pub struct BulkCheckSummary {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub back_in_stock_count: usize,
    pub price_drop_count: usize,
    pub results: Vec<BulkCheckResult>,
}

/// Result of an availability check with optional notification data
#[derive(Debug, Serialize)]
pub struct CheckResultWithNotification {
    pub check: AvailabilityCheckModel,
    pub notification: Option<NotificationData>,
    pub daily_comparison: DailyPriceComparison,
}

/// Result of processing a single availability check
pub struct CheckProcessingResult {
    pub status: AvailabilityStatus,
    pub price_minor_units: Option<i64>,
    pub price_currency: Option<String>,
    pub error: Option<String>,
    pub is_back_in_stock: bool,
    pub is_price_drop: bool,
}

/// Context for checking a single product in a bulk operation
pub struct ProductCheckContext {
    pub previous_status: Option<AvailabilityStatus>,
}

/// Accumulated counters for bulk check results
#[derive(Default)]
pub struct BulkCheckCounters {
    pub successful: usize,
    pub failed: usize,
    pub back_in_stock_count: usize,
    pub price_drop_count: usize,
}

/// Result of comparing today's average price vs yesterday's average price
#[derive(Debug, Clone, Default, Serialize)]
pub struct DailyPriceComparison {
    pub today_average_minor_units: Option<i64>,
    pub yesterday_average_minor_units: Option<i64>,
}

impl BulkCheckResult {
    /// Build a result from a successful processing result with daily comparison data
    pub fn from_processing_result(
        product: &ProductModel,
        result: &CheckProcessingResult,
        context: &ProductCheckContext,
        daily_comparison: &DailyPriceComparison,
    ) -> Self {
        let currency_exponent = result
            .price_currency
            .as_deref()
            .or(product.currency.as_deref())
            .map(currency::currency_exponent);
        Self {
            product_id: product.id.to_string(),
            product_name: product.name.clone(),
            product_retailer_id: None,
            url: None,
            status: result.status.clone(),
            previous_status: context.previous_status.clone(),
            is_back_in_stock: result.is_back_in_stock,
            price_minor_units: result.price_minor_units,
            price_currency: result.price_currency.clone(),
            currency_exponent,
            today_average_price_minor_units: daily_comparison.today_average_minor_units,
            yesterday_average_price_minor_units: daily_comparison.yesterday_average_minor_units,
            is_price_drop: result.is_price_drop,
            error: result.error.clone(),
        }
    }

    /// Set retailer info on the result.
    pub fn with_retailer(
        mut self,
        product_retailer: &crate::entities::prelude::ProductRetailerModel,
    ) -> Self {
        self.product_retailer_id = Some(product_retailer.id.to_string());
        self.url = Some(product_retailer.url.clone());
        self
    }

    /// Build an error result when context or infrastructure fails
    pub fn error_for_product(product: &ProductModel, error_message: String) -> Self {
        Self {
            product_id: product.id.to_string(),
            product_name: product.name.clone(),
            error: Some(error_message),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use product_stalker_core::services::notification_helpers::NotificationData;
    use uuid::Uuid;

    /// Tests for BulkCheckResult struct
    mod bulk_check_result_tests {
        use super::*;

        #[test]
        fn test_serialize() {
            let result = BulkCheckResult {
                product_id: "test-id-123".to_string(),
                product_name: "Test Product".to_string(),
                status: AvailabilityStatus::InStock,
                previous_status: Some(AvailabilityStatus::OutOfStock),
                is_back_in_stock: true,
                price_minor_units: Some(78900),
                price_currency: Some("USD".to_string()),
                today_average_price_minor_units: Some(78900),
                yesterday_average_price_minor_units: Some(89900),
                is_price_drop: true,
                ..Default::default()
            };
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("test-id-123"));
            assert!(json.contains("Test Product"));
            assert!(json.contains("in_stock"));
            assert!(json.contains("out_of_stock"));
            assert!(json.contains("78900"));
        }

        #[test]
        fn test_with_error() {
            let result = BulkCheckResult {
                product_id: "error-id".to_string(),
                product_name: "Error Product".to_string(),
                error: Some("Failed to fetch".to_string()),
                ..Default::default()
            };
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("Failed to fetch"));
            assert!(json.contains("unknown"));
        }

        #[test]
        fn test_with_retailer() {
            use crate::entities::prelude::ProductRetailerModel;

            let pr = ProductRetailerModel {
                id: Uuid::new_v4(),
                product_id: Uuid::new_v4(),
                retailer_id: Uuid::new_v4(),
                url: "https://amazon.com/dp/B123".to_string(),
                label: Some("64GB".to_string()),
                sort_order: 0,
                created_at: chrono::Utc::now(),
            };

            let result = BulkCheckResult {
                product_id: "p1".to_string(),
                product_name: "Product 1".to_string(),
                ..Default::default()
            }
            .with_retailer(&pr);

            assert_eq!(result.product_retailer_id, Some(pr.id.to_string()));
            assert_eq!(result.url, Some("https://amazon.com/dp/B123".to_string()));
        }
    }

    /// Tests for BulkCheckSummary struct
    mod bulk_check_summary_tests {
        use super::*;

        #[test]
        fn test_serialize() {
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
        fn test_with_results() {
            let result = BulkCheckResult {
                product_id: "p1".to_string(),
                product_name: "Product 1".to_string(),
                status: AvailabilityStatus::InStock,
                previous_status: Some(AvailabilityStatus::OutOfStock),
                is_back_in_stock: true,
                price_minor_units: Some(78900),
                price_currency: Some("USD".to_string()),
                ..Default::default()
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
    }

    /// Tests for CheckResultWithNotification struct
    mod check_result_with_notification_tests {
        use super::*;

        #[test]
        fn test_serialize() {
            let check = AvailabilityCheckModel {
                id: Uuid::new_v4(),
                product_id: Uuid::new_v4(),
                product_retailer_id: None,
                status: "in_stock".to_string(),
                raw_availability: Some("http://schema.org/InStock".to_string()),
                error_message: None,
                checked_at: chrono::Utc::now(),
                price_minor_units: Some(78900),
                price_currency: Some("USD".to_string()),
                raw_price: Some("789.00".to_string()),
                normalized_price_minor_units: None,
                normalized_currency: None,
            };
            let result = CheckResultWithNotification {
                check,
                notification: Some(NotificationData {
                    title: "Back in Stock!".to_string(),
                    body: "Product available".to_string(),
                }),
                daily_comparison: DailyPriceComparison {
                    today_average_minor_units: Some(78900),
                    yesterday_average_minor_units: Some(89900),
                },
            };
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("in_stock"));
            assert!(json.contains("Back in Stock!"));
        }

        #[test]
        fn test_with_none_notification() {
            let check = AvailabilityCheckModel {
                id: Uuid::new_v4(),
                product_id: Uuid::new_v4(),
                product_retailer_id: None,
                status: "out_of_stock".to_string(),
                raw_availability: None,
                error_message: None,
                checked_at: chrono::Utc::now(),
                price_minor_units: None,
                price_currency: None,
                raw_price: None,
                normalized_price_minor_units: None,
                normalized_currency: None,
            };
            let result = CheckResultWithNotification {
                check,
                notification: None,
                daily_comparison: DailyPriceComparison::default(),
            };
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains("out_of_stock"));
            assert!(json.contains("null") || !json.contains("notification"));
        }
    }
}
