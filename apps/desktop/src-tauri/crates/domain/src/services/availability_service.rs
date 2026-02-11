//! Availability service for checking product availability.

use sea_orm::DatabaseConnection;
use serde::Serialize;
use uuid::Uuid;

use crate::entities::availability_check::AvailabilityStatus;
use crate::entities::prelude::{AvailabilityCheckModel, ProductModel};
use crate::repositories::{AvailabilityCheckRepository, CreateCheckParams, ProductRepository};
use product_stalker_core::services::Settings;
use product_stalker_core::AppError;

use product_stalker_core::services::notification_helpers::NotificationData;

use super::{currency, NotificationService, ScraperService};

/// Result of a single product availability check in a bulk operation
#[derive(Debug, Clone, Default, Serialize)]
pub struct BulkCheckResult {
    pub product_id: String,
    pub product_name: String,
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

/// Service layer for availability checking business logic
pub struct AvailabilityService;

impl AvailabilityService {
    /// Build CreateCheckParams from a successful scraping result
    fn params_from_success(result: super::scraper::ScrapingResult) -> CreateCheckParams {
        CreateCheckParams {
            status: result.status,
            raw_availability: result.raw_availability,
            error_message: None,
            price_minor_units: result.price.price_minor_units,
            price_currency: result.price.price_currency,
            raw_price: result.price.raw_price,
        }
    }

    /// Build CreateCheckParams from a scraping error
    fn params_from_error(error: &AppError) -> CreateCheckParams {
        CreateCheckParams {
            error_message: Some(error.to_string()),
            ..Default::default()
        }
    }

    /// Check the availability of a product by its ID
    ///
    /// Fetches the product's URL, scrapes the page for availability info,
    /// and stores the result in the database.
    /// Auto-sets the product's currency on first successful price scrape.
    pub async fn check_product(
        conn: &DatabaseConnection,
        product_id: Uuid,
        enable_headless: bool,
    ) -> Result<AvailabilityCheckModel, AppError> {
        let product = ProductRepository::find_by_id(conn, product_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", product_id)))?;

        let result =
            ScraperService::check_availability_with_headless(&product.url, enable_headless).await;

        let params = match result {
            Ok(scraping_result) => {
                let params = Self::params_from_success(scraping_result);
                Self::auto_set_product_currency(conn, &product, params.price_currency.as_deref())
                    .await;
                params
            }
            Err(e) => Self::params_from_error(&e),
        };

        AvailabilityCheckRepository::create(conn, Uuid::new_v4(), product_id, params).await
    }

    /// Auto-set product currency from scraped price data.
    ///
    /// If the product has no currency set and the scrape found one, saves it.
    /// If the product already has a different currency, logs a warning but keeps the existing one.
    async fn auto_set_product_currency(
        conn: &DatabaseConnection,
        product: &ProductModel,
        scraped_currency: Option<&str>,
    ) {
        let Some(scraped) = scraped_currency else {
            return;
        };

        match &product.currency {
            None => {
                let update = crate::repositories::ProductUpdateInput {
                    currency: Some(Some(scraped.to_string())),
                    ..Default::default()
                };
                if let Err(e) = ProductRepository::update(conn, product.clone(), update).await {
                    log::warn!(
                        "Failed to auto-set currency for product {}: {}",
                        product.id,
                        e
                    );
                }
            }
            Some(existing) if !existing.eq_ignore_ascii_case(scraped) => {
                log::warn!(
                    "Product {} has currency {} but scraped {}; keeping existing",
                    product.id,
                    existing,
                    scraped
                );
            }
            _ => {} // Currency matches, nothing to do
        }
    }

    /// Process the result of an availability check into a structured result
    pub fn process_check_result(
        check_result: Result<AvailabilityCheckModel, AppError>,
        previous_status: &Option<AvailabilityStatus>,
        daily_comparison: &DailyPriceComparison,
    ) -> CheckProcessingResult {
        match check_result {
            Ok(check) if check.error_message.is_some() => Self::result_with_scraper_error(check),
            Ok(check) => {
                Self::result_from_successful_check(check, previous_status, daily_comparison)
            }
            Err(e) => Self::result_from_infrastructure_error(e),
        }
    }

    /// Build result when scraper failed but a record was created
    fn result_with_scraper_error(check: AvailabilityCheckModel) -> CheckProcessingResult {
        CheckProcessingResult {
            status: check.status_enum(),
            price_minor_units: check.price_minor_units,
            price_currency: check.price_currency,
            error: check.error_message,
            is_back_in_stock: false,
            is_price_drop: false,
        }
    }

    /// Build result from a successful availability check
    fn result_from_successful_check(
        check: AvailabilityCheckModel,
        previous_status: &Option<AvailabilityStatus>,
        daily_comparison: &DailyPriceComparison,
    ) -> CheckProcessingResult {
        let status = check.status_enum();
        let is_back_in_stock = Self::is_back_in_stock(previous_status, &status);
        let is_price_drop = Self::is_price_drop(
            daily_comparison.yesterday_average_minor_units,
            daily_comparison.today_average_minor_units,
        );

        CheckProcessingResult {
            status,
            price_minor_units: check.price_minor_units,
            price_currency: check.price_currency,
            error: None,
            is_back_in_stock,
            is_price_drop,
        }
    }

    /// Build result when a database/infrastructure error occurred
    fn result_from_infrastructure_error(error: AppError) -> CheckProcessingResult {
        CheckProcessingResult {
            status: AvailabilityStatus::Unknown,
            price_minor_units: None,
            price_currency: None,
            error: Some(error.to_string()),
            is_back_in_stock: false,
            is_price_drop: false,
        }
    }

    /// Check a single product and return the result along with processing data
    ///
    /// This is the core logic for checking a single product in a bulk operation,
    /// without any event emission.
    pub async fn check_single_product(
        conn: &DatabaseConnection,
        product: &ProductModel,
        enable_headless: bool,
    ) -> (BulkCheckResult, CheckProcessingResult) {
        // Step 1: Get previous check context
        let context = match Self::get_product_check_context(conn, product.id).await {
            Ok(ctx) => ctx,
            Err(e) => return Self::build_context_error_result(product, e),
        };

        // Step 2: Perform the availability check
        let check_result = Self::check_product(conn, product.id, enable_headless).await;

        // Step 3: Get daily price comparison (includes the new check in today's average)
        let daily_comparison = match Self::get_daily_price_comparison(conn, product.id).await {
            Ok(dc) => dc,
            Err(e) => return Self::build_context_error_result(product, e),
        };

        // Step 4: Process the result
        let result =
            Self::process_check_result(check_result, &context.previous_status, &daily_comparison);

        // Step 5: Build the bulk result
        let bulk_result =
            BulkCheckResult::from_processing_result(product, &result, &context, &daily_comparison);

        (bulk_result, result)
    }

    /// Build error result when context fetch fails
    pub fn build_context_error_result(
        product: &ProductModel,
        error: AppError,
    ) -> (BulkCheckResult, CheckProcessingResult) {
        let error_message = error.to_string();
        let result = CheckProcessingResult {
            status: AvailabilityStatus::Unknown,
            price_minor_units: None,
            price_currency: None,
            error: Some(error_message.clone()),
            is_back_in_stock: false,
            is_price_drop: false,
        };
        let bulk_result = BulkCheckResult::error_for_product(product, error_message);
        (bulk_result, result)
    }

    /// Build summary from collected results
    pub fn build_summary_from_results(
        total: usize,
        results: Vec<(BulkCheckResult, CheckProcessingResult)>,
    ) -> BulkCheckSummary {
        let mut counters = BulkCheckCounters::default();
        let mut bulk_results = Vec::with_capacity(results.len());

        for (bulk_result, processing_result) in results {
            Self::update_counters(&mut counters, &processing_result);
            bulk_results.push(bulk_result);
        }

        Self::build_bulk_summary(total, counters, bulk_results)
    }

    /// Get the context needed before checking a product (previous status)
    pub async fn get_product_check_context(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<ProductCheckContext, AppError> {
        let previous_check = Self::get_latest(conn, product_id).await?;
        let previous_status = previous_check.as_ref().map(|c| c.status_enum());

        Ok(ProductCheckContext { previous_status })
    }

    /// Update counters based on the check result
    pub fn update_counters(counters: &mut BulkCheckCounters, result: &CheckProcessingResult) {
        if result.error.is_some() {
            counters.failed += 1;
            return;
        }

        counters.successful += 1;
        if result.is_back_in_stock {
            counters.back_in_stock_count += 1;
        }
        if result.is_price_drop {
            counters.price_drop_count += 1;
        }
    }

    /// Build the final bulk check summary from counters and results
    pub fn build_bulk_summary(
        total: usize,
        counters: BulkCheckCounters,
        results: Vec<BulkCheckResult>,
    ) -> BulkCheckSummary {
        BulkCheckSummary {
            total,
            successful: counters.successful,
            failed: counters.failed,
            back_in_stock_count: counters.back_in_stock_count,
            price_drop_count: counters.price_drop_count,
            results,
        }
    }

    /// Determines if a product has transitioned back to being in stock.
    ///
    /// A product is considered "back in stock" only if:
    /// 1. There was a previous check (first check doesn't count as "back")
    /// 2. The previous status was NOT in_stock
    /// 3. The new status IS in_stock
    ///
    /// This ensures we only notify users about meaningful transitions,
    /// not products that were always in stock or are being checked for the first time.
    pub fn is_back_in_stock(
        previous_status: &Option<AvailabilityStatus>,
        new_status: &AvailabilityStatus,
    ) -> bool {
        match previous_status {
            Some(prev) => {
                *prev != AvailabilityStatus::InStock && *new_status == AvailabilityStatus::InStock
            }
            None => false,
        }
    }

    /// Check if today's average price dropped compared to yesterday's
    pub fn is_price_drop(yesterday_average: Option<i64>, today_average: Option<i64>) -> bool {
        match (yesterday_average, today_average) {
            (Some(prev), Some(new)) => new < prev,
            _ => false, // No price drop if either is None
        }
    }

    /// Get today's and yesterday's average prices for comparison
    ///
    /// Uses rolling 24-hour windows instead of calendar dates for timezone
    /// resilience. "Today" = last 24 hours, "Yesterday" = 24–48 hours ago.
    pub async fn get_daily_price_comparison(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<DailyPriceComparison, AppError> {
        let now = chrono::Utc::now();
        let twenty_four_hours_ago = now - chrono::Duration::hours(24);
        let forty_eight_hours_ago = now - chrono::Duration::hours(48);

        let today_average = AvailabilityCheckRepository::get_average_price_for_period(
            conn,
            product_id,
            twenty_four_hours_ago,
            now,
        )
        .await?;
        let yesterday_average = AvailabilityCheckRepository::get_average_price_for_period(
            conn,
            product_id,
            forty_eight_hours_ago,
            twenty_four_hours_ago,
        )
        .await?;

        Ok(DailyPriceComparison {
            today_average_minor_units: today_average,
            yesterday_average_minor_units: yesterday_average,
        })
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
        settings: &Settings,
    ) -> Result<CheckResultWithNotification, AppError> {
        // Step 1: Get previous status before checking
        let previous_check = Self::get_latest(conn, product_id).await?;
        let previous_status = previous_check.map(|c| c.status_enum());

        // Step 2: Perform the check
        let check = Self::check_product(conn, product_id, settings.enable_headless_browser).await?;

        // Step 3: Get daily price comparison (includes the new check in today's average)
        let daily_comparison = Self::get_daily_price_comparison(conn, product_id).await?;

        // Step 4: Determine if back in stock
        let is_back_in_stock = Self::is_back_in_stock(&previous_status, &check.status_enum());

        // Step 5: Build notification if applicable (using NotificationService)
        let notification = NotificationService::build_single_notification(
            conn,
            product_id,
            settings,
            is_back_in_stock,
        )
        .await?;

        Ok(CheckResultWithNotification {
            check,
            notification,
            daily_comparison,
        })
    }

    /// Build notification data for a bulk check using pre-fetched settings
    ///
    /// Delegates to NotificationService for actual notification composition.
    pub fn build_bulk_notification_with_settings(
        settings: &Settings,
        summary: &BulkCheckSummary,
    ) -> Option<NotificationData> {
        NotificationService::build_bulk_notification(
            settings,
            summary.back_in_stock_count,
            summary.price_drop_count,
            &summary.results,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::CreateCheckParams;
    use crate::test_utils::{create_test_product, setup_availability_db};

    /// Tests for get_latest and get_history methods
    mod history_tests {
        use super::*;

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

            for _ in 0..5 {
                AvailabilityCheckRepository::create(
                    &conn,
                    Uuid::new_v4(),
                    product_id,
                    CreateCheckParams {
                        status: AvailabilityStatus::InStock,
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
        async fn test_get_history_without_limit() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            for i in 0..3 {
                AvailabilityCheckRepository::create(
                    &conn,
                    Uuid::new_v4(),
                    product_id,
                    CreateCheckParams {
                        status: if i % 2 == 0 {
                            AvailabilityStatus::InStock
                        } else {
                            AvailabilityStatus::OutOfStock
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

        #[tokio::test]
        async fn test_get_latest_with_multiple_checks() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            for _ in 0..3 {
                AvailabilityCheckRepository::create(
                    &conn,
                    Uuid::new_v4(),
                    product_id,
                    CreateCheckParams {
                        status: AvailabilityStatus::InStock,
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

    /// Tests for check_product method
    mod check_product_tests {
        use super::*;

        #[tokio::test]
        async fn test_check_product_not_found() {
            let conn = setup_availability_db().await;
            let fake_id = Uuid::new_v4();

            let result = AvailabilityService::check_product(&conn, fake_id, false).await;

            assert!(result.is_err());
            assert!(matches!(result, Err(AppError::NotFound(_))));
        }
    }

    /// Tests for is_back_in_stock logic
    mod back_in_stock_tests {
        use super::*;

        #[test]
        fn test_from_out_of_stock() {
            let previous = Some(AvailabilityStatus::OutOfStock);
            assert!(AvailabilityService::is_back_in_stock(
                &previous,
                &AvailabilityStatus::InStock
            ));
        }

        #[test]
        fn test_from_back_order() {
            let previous = Some(AvailabilityStatus::BackOrder);
            assert!(AvailabilityService::is_back_in_stock(
                &previous,
                &AvailabilityStatus::InStock
            ));
        }

        #[test]
        fn test_from_unknown() {
            let previous = Some(AvailabilityStatus::Unknown);
            assert!(AvailabilityService::is_back_in_stock(
                &previous,
                &AvailabilityStatus::InStock
            ));
        }

        #[test]
        fn test_already_in_stock() {
            let previous = Some(AvailabilityStatus::InStock);
            assert!(!AvailabilityService::is_back_in_stock(
                &previous,
                &AvailabilityStatus::InStock
            ));
        }

        #[test]
        fn test_still_out_of_stock() {
            let previous = Some(AvailabilityStatus::OutOfStock);
            assert!(!AvailabilityService::is_back_in_stock(
                &previous,
                &AvailabilityStatus::OutOfStock
            ));
        }

        #[test]
        fn test_no_previous() {
            let previous: Option<AvailabilityStatus> = None;
            assert!(!AvailabilityService::is_back_in_stock(
                &previous,
                &AvailabilityStatus::InStock
            ));
        }

        #[test]
        fn test_to_out_of_stock() {
            let previous = Some(AvailabilityStatus::InStock);
            assert!(!AvailabilityService::is_back_in_stock(
                &previous,
                &AvailabilityStatus::OutOfStock
            ));
        }

        #[test]
        fn test_to_back_order() {
            let previous = Some(AvailabilityStatus::InStock);
            assert!(!AvailabilityService::is_back_in_stock(
                &previous,
                &AvailabilityStatus::BackOrder
            ));
        }

        #[test]
        fn test_to_unknown() {
            let previous = Some(AvailabilityStatus::InStock);
            assert!(!AvailabilityService::is_back_in_stock(
                &previous,
                &AvailabilityStatus::Unknown
            ));
        }
    }

    /// Tests for is_price_drop logic
    mod price_drop_tests {
        use super::*;

        #[test]
        fn test_from_higher() {
            assert!(AvailabilityService::is_price_drop(Some(10000), Some(8000)));
        }

        #[test]
        fn test_same_price() {
            assert!(!AvailabilityService::is_price_drop(
                Some(10000),
                Some(10000)
            ));
        }

        #[test]
        fn test_price_increase() {
            assert!(!AvailabilityService::is_price_drop(Some(8000), Some(10000)));
        }

        #[test]
        fn test_no_previous() {
            assert!(!AvailabilityService::is_price_drop(None, Some(10000)));
        }

        #[test]
        fn test_no_new() {
            assert!(!AvailabilityService::is_price_drop(Some(10000), None));
        }

        #[test]
        fn test_both_none() {
            assert!(!AvailabilityService::is_price_drop(None, None));
        }
    }

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
                status: "in_stock".to_string(),
                raw_availability: Some("http://schema.org/InStock".to_string()),
                error_message: None,
                checked_at: chrono::Utc::now(),
                price_minor_units: Some(78900),
                price_currency: Some("USD".to_string()),
                raw_price: Some("789.00".to_string()),
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
                status: "out_of_stock".to_string(),
                raw_availability: None,
                error_message: None,
                checked_at: chrono::Utc::now(),
                price_minor_units: None,
                price_currency: None,
                raw_price: None,
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

    /// Tests for get_daily_price_comparison method
    mod daily_price_comparison_tests {
        use super::*;
        use crate::repositories::AvailabilityCheckRepository;
        use crate::test_utils::{create_test_product, setup_availability_db};

        #[tokio::test]
        async fn test_no_data() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            let comparison = AvailabilityService::get_daily_price_comparison(&conn, product_id)
                .await
                .unwrap();

            assert_eq!(comparison.today_average_minor_units, None);
            assert_eq!(comparison.yesterday_average_minor_units, None);
        }

        #[tokio::test]
        async fn test_today_only() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            // Check 6 hours ago — within "today" (last 24h)
            let check_time = chrono::Utc::now() - chrono::Duration::hours(6);
            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(15000),
                check_time,
            )
            .await;

            let comparison = AvailabilityService::get_daily_price_comparison(&conn, product_id)
                .await
                .unwrap();

            assert_eq!(comparison.today_average_minor_units, Some(15000));
            assert_eq!(comparison.yesterday_average_minor_units, None);
        }

        #[tokio::test]
        async fn test_today_and_yesterday() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            // Today: 6 hours ago
            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(15000),
                chrono::Utc::now() - chrono::Duration::hours(6),
            )
            .await;

            // Yesterday: 30 hours ago (within 24–48h window)
            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(20000),
                chrono::Utc::now() - chrono::Duration::hours(30),
            )
            .await;

            let comparison = AvailabilityService::get_daily_price_comparison(&conn, product_id)
                .await
                .unwrap();

            assert_eq!(comparison.today_average_minor_units, Some(15000));
            assert_eq!(comparison.yesterday_average_minor_units, Some(20000));
        }

        #[tokio::test]
        async fn test_yesterday_only() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            // 30 hours ago — within "yesterday" (24–48h window)
            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(20000),
                chrono::Utc::now() - chrono::Duration::hours(30),
            )
            .await;

            let comparison = AvailabilityService::get_daily_price_comparison(&conn, product_id)
                .await
                .unwrap();

            assert_eq!(comparison.today_average_minor_units, None);
            assert_eq!(comparison.yesterday_average_minor_units, Some(20000));
        }

        #[tokio::test]
        async fn test_old_data_excluded() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            // 72 hours ago — beyond both windows
            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(20000),
                chrono::Utc::now() - chrono::Duration::hours(72),
            )
            .await;

            let comparison = AvailabilityService::get_daily_price_comparison(&conn, product_id)
                .await
                .unwrap();

            assert_eq!(comparison.today_average_minor_units, None);
            assert_eq!(comparison.yesterday_average_minor_units, None);
        }
    }
}
