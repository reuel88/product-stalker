use std::time::Duration;

use sea_orm::DatabaseConnection;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::entities::availability_check::AvailabilityStatus;
use crate::entities::prelude::AvailabilityCheckModel;
use crate::error::AppError;
use crate::repositories::{AvailabilityCheckRepository, CreateCheckParams, ProductRepository};
use crate::services::notification_service::NotificationData;
use crate::services::scraper::ScrapingResult;
use crate::services::setting_service::Settings;
use crate::services::{NotificationService, ScraperService, SettingService};

/// Event name for per-product progress updates during bulk check
pub const EVENT_CHECK_PROGRESS: &str = "availability:check-progress";

/// Event name for bulk check completion
pub const EVENT_CHECK_COMPLETE: &str = "availability:check-complete";

/// Event payload for per-product progress updates
#[derive(Debug, Clone, Serialize)]
pub struct CheckProgressEvent {
    pub current_index: usize,
    pub total_count: usize,
    pub result: BulkCheckResult,
}

/// Event payload for bulk check completion
#[derive(Debug, Clone, Serialize)]
pub struct CheckCompleteEvent {
    pub summary: BulkCheckSummary,
}

/// Result of a single product availability check in a bulk operation
#[derive(Debug, Clone, Serialize)]
pub struct BulkCheckResult {
    pub product_id: String,
    pub product_name: String,
    pub status: String,
    pub previous_status: Option<String>,
    pub is_back_in_stock: bool,
    pub price_cents: Option<i64>,
    pub price_currency: Option<String>,
    pub today_average_price_cents: Option<i64>,
    pub yesterday_average_price_cents: Option<i64>,
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

/// Context for checking a single product in a bulk operation
struct ProductCheckContext {
    previous_status: Option<String>,
}

/// Accumulated counters for bulk check results
#[derive(Default)]
struct BulkCheckCounters {
    successful: usize,
    failed: usize,
    back_in_stock_count: usize,
    price_drop_count: usize,
}

/// Result of comparing today's average price vs yesterday's average price
#[derive(Debug, Clone, Default, Serialize)]
pub struct DailyPriceComparison {
    pub today_average_cents: Option<i64>,
    pub yesterday_average_cents: Option<i64>,
}

impl BulkCheckResult {
    /// Build a result from a successful processing result with daily comparison data
    fn from_processing_result(
        product: &crate::entities::prelude::ProductModel,
        result: &CheckProcessingResult,
        context: &ProductCheckContext,
        daily_comparison: &DailyPriceComparison,
    ) -> Self {
        Self {
            product_id: product.id.to_string(),
            product_name: product.name.clone(),
            status: result.status.clone(),
            previous_status: context.previous_status.clone(),
            is_back_in_stock: result.is_back_in_stock,
            price_cents: result.price_cents,
            price_currency: result.price_currency.clone(),
            today_average_price_cents: daily_comparison.today_average_cents,
            yesterday_average_price_cents: daily_comparison.yesterday_average_cents,
            is_price_drop: result.is_price_drop,
            error: result.error.clone(),
        }
    }

    /// Build an error result when context or infrastructure fails
    fn error_for_product(
        product: &crate::entities::prelude::ProductModel,
        error_message: String,
    ) -> Self {
        Self {
            product_id: product.id.to_string(),
            product_name: product.name.clone(),
            status: AvailabilityStatus::Unknown.as_str().to_string(),
            previous_status: None,
            is_back_in_stock: false,
            price_cents: None,
            price_currency: None,
            today_average_price_cents: None,
            yesterday_average_price_cents: None,
            is_price_drop: false,
            error: Some(error_message),
        }
    }
}

/// Service layer for availability checking business logic
pub struct AvailabilityService;

/// Delay between product checks to avoid overwhelming target servers.
///
/// 500ms was chosen to balance:
/// - Respectful scraping (not hammering servers)
/// - Reasonable total check time for large product lists
/// - Typical anti-bot detection thresholds
const RATE_LIMIT_DELAY_MS: u64 = 500;

impl AvailabilityService {
    /// Build CreateCheckParams from a successful scraping result
    fn params_from_success(result: ScrapingResult) -> CreateCheckParams {
        CreateCheckParams {
            status: result.status.as_str().to_string(),
            raw_availability: result.raw_availability,
            error_message: None,
            price_cents: result.price.price_cents,
            price_currency: result.price.price_currency,
            raw_price: result.price.raw_price,
        }
    }

    /// Build CreateCheckParams from a scraping error
    fn params_from_error(error: &AppError) -> CreateCheckParams {
        CreateCheckParams {
            status: AvailabilityStatus::Unknown.as_str().to_string(),
            error_message: Some(error.to_string()),
            ..Default::default()
        }
    }

    /// Check the availability of a product by its ID
    ///
    /// Fetches the product's URL, scrapes the page for availability info,
    /// and stores the result in the database.
    pub async fn check_product(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<AvailabilityCheckModel, AppError> {
        let product = ProductRepository::find_by_id(conn, product_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", product_id)))?;

        let settings = SettingService::get(conn).await?;
        let result = ScraperService::check_availability_with_headless(
            &product.url,
            settings.enable_headless_browser,
        )
        .await;

        let params = match result {
            Ok(scraping_result) => Self::params_from_success(scraping_result),
            Err(e) => Self::params_from_error(&e),
        };

        AvailabilityCheckRepository::create(conn, Uuid::new_v4(), product_id, params).await
    }

    /// Process the result of an availability check into a structured result
    fn process_check_result(
        check_result: Result<AvailabilityCheckModel, AppError>,
        previous_status: &Option<String>,
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
            status: check.status,
            price_cents: check.price_cents,
            price_currency: check.price_currency,
            error: check.error_message,
            is_back_in_stock: false,
            is_price_drop: false,
        }
    }

    /// Build result from a successful availability check
    fn result_from_successful_check(
        check: AvailabilityCheckModel,
        previous_status: &Option<String>,
        daily_comparison: &DailyPriceComparison,
    ) -> CheckProcessingResult {
        let is_back_in_stock = Self::is_back_in_stock(previous_status, &check.status);
        let is_price_drop = Self::is_price_drop(
            daily_comparison.yesterday_average_cents,
            daily_comparison.today_average_cents,
        );

        CheckProcessingResult {
            status: check.status,
            price_cents: check.price_cents,
            price_currency: check.price_currency,
            error: None,
            is_back_in_stock,
            is_price_drop,
        }
    }

    /// Build result when a database/infrastructure error occurred
    fn result_from_infrastructure_error(error: AppError) -> CheckProcessingResult {
        CheckProcessingResult {
            status: AvailabilityStatus::Unknown.as_str().to_string(),
            price_cents: None,
            price_currency: None,
            error: Some(error.to_string()),
            is_back_in_stock: false,
            is_price_drop: false,
        }
    }

    /// Check each product with rate limiting between requests.
    ///
    /// ## Orchestration Flow
    ///
    /// For each product in the list:
    /// 1. **Fetch context**: Get previous status and price from the last check
    /// 2. **Scrape product page**: Use HTTP (fast path) or headless browser (fallback)
    /// 3. **Parse Schema.org data**: Extract availability status and price
    /// 4. **Process results**: Detect back-in-stock transitions and price drops
    /// 5. **Store check record**: Save results to database
    /// 6. **Emit progress event**: Notify frontend of completion
    /// 7. **Rate limit**: Wait 500ms before checking next product
    ///
    /// The rate limiting (500ms between requests) balances:
    /// - Respectful scraping behavior (not overwhelming target servers)
    /// - Reasonable total check time for large product lists
    /// - Staying below typical anti-bot detection thresholds
    async fn check_products_with_rate_limit(
        conn: &DatabaseConnection,
        products: &[crate::entities::prelude::ProductModel],
        app: &AppHandle,
    ) -> Vec<(BulkCheckResult, CheckProcessingResult)> {
        let mut results = Vec::with_capacity(products.len());
        let total_count = products.len();

        for (index, product) in products.iter().enumerate() {
            if index > 0 {
                tokio::time::sleep(Duration::from_millis(RATE_LIMIT_DELAY_MS)).await;
            }
            let (bulk_result, processing_result) =
                Self::check_single_product_in_bulk(conn, product).await;

            // Emit progress event (ignore errors to avoid blocking)
            let _ = app.emit(
                EVENT_CHECK_PROGRESS,
                CheckProgressEvent {
                    current_index: index + 1,
                    total_count,
                    result: bulk_result.clone(),
                },
            );

            results.push((bulk_result, processing_result));
        }

        results
    }

    /// Check a single product in a bulk operation with context and error handling
    ///
    /// Combines context fetching, availability checking, and result processing
    /// into a single function to reduce call depth.
    async fn check_single_product_in_bulk(
        conn: &DatabaseConnection,
        product: &crate::entities::prelude::ProductModel,
    ) -> (BulkCheckResult, CheckProcessingResult) {
        // Step 1: Get previous check context
        let context = match Self::get_product_check_context(conn, product.id).await {
            Ok(ctx) => ctx,
            Err(e) => return Self::build_context_error_result(product, e),
        };

        // Step 2: Perform the availability check
        let check_result = Self::check_product(conn, product.id).await;

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
    fn build_context_error_result(
        product: &crate::entities::prelude::ProductModel,
        error: AppError,
    ) -> (BulkCheckResult, CheckProcessingResult) {
        let error_message = error.to_string();
        let result = CheckProcessingResult {
            status: AvailabilityStatus::Unknown.as_str().to_string(),
            price_cents: None,
            price_currency: None,
            error: Some(error_message.clone()),
            is_back_in_stock: false,
            is_price_drop: false,
        };
        let bulk_result = BulkCheckResult::error_for_product(product, error_message);
        (bulk_result, result)
    }

    /// Build summary from collected results
    fn build_summary_from_results(
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
    async fn get_product_check_context(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<ProductCheckContext, AppError> {
        let previous_check = Self::get_latest(conn, product_id).await?;
        let previous_status = previous_check.as_ref().map(|c| c.status.clone());

        Ok(ProductCheckContext { previous_status })
    }

    /// Update counters based on the check result
    fn update_counters(counters: &mut BulkCheckCounters, result: &CheckProcessingResult) {
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
    fn build_bulk_summary(
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
    pub fn is_back_in_stock(previous_status: &Option<String>, new_status: &str) -> bool {
        let in_stock = AvailabilityStatus::InStock.as_str();
        match previous_status {
            Some(prev) => prev != in_stock && new_status == in_stock,
            None => false,
        }
    }

    /// Check if the price dropped from previous check
    pub fn is_price_drop(previous_price: Option<i64>, new_price: Option<i64>) -> bool {
        match (previous_price, new_price) {
            (Some(prev), Some(new)) => new < prev,
            _ => false, // No price drop if either is None
        }
    }

    /// Get today's and yesterday's average prices for comparison
    ///
    /// Uses UTC dates for consistency. Returns a DailyPriceComparison struct
    /// containing both averages (or None if no data exists for that day).
    pub async fn get_daily_price_comparison(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<DailyPriceComparison, AppError> {
        let today = chrono::Utc::now().date_naive();
        let yesterday = today - chrono::Duration::days(1);

        let today_average =
            AvailabilityCheckRepository::get_average_price_for_date(conn, product_id, today)
                .await?;
        let yesterday_average =
            AvailabilityCheckRepository::get_average_price_for_date(conn, product_id, yesterday)
                .await?;

        Ok(DailyPriceComparison {
            today_average_cents: today_average,
            yesterday_average_cents: yesterday_average,
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
    ) -> Result<CheckResultWithNotification, AppError> {
        // Step 1: Fetch settings upfront for notification check
        let settings = SettingService::get(conn).await?;

        // Step 2: Get previous status before checking
        let previous_check = Self::get_latest(conn, product_id).await?;
        let previous_status = previous_check.map(|c| c.status);

        // Step 3: Perform the check
        let check = Self::check_product(conn, product_id).await?;

        // Step 4: Get daily price comparison (includes the new check in today's average)
        let daily_comparison = Self::get_daily_price_comparison(conn, product_id).await?;

        // Step 5: Determine if back in stock
        let is_back_in_stock = Self::is_back_in_stock(&previous_status, &check.status);

        // Step 6: Build notification if applicable (using NotificationService)
        let notification = NotificationService::build_single_notification(
            conn,
            product_id,
            &settings,
            is_back_in_stock,
        )
        .await?;

        Ok(CheckResultWithNotification {
            check,
            notification,
            daily_comparison,
        })
    }

    /// Check all products and return notification data if applicable
    ///
    /// Encapsulates all business logic for bulk checks including notification composition.
    /// This is the main orchestrator function that coordinates the entire bulk check workflow:
    ///
    /// ## Orchestration Flow
    /// 1. Fetch settings upfront (used for headless browser setting and notifications)
    /// 2. Fetch all products from database
    /// 3. Check each product with rate limiting (emitting progress events)
    /// 4. Build summary from results
    /// 5. Emit completion event
    /// 6. Compose notification if applicable
    pub async fn check_all_products_with_notification(
        conn: &DatabaseConnection,
        app: &AppHandle,
    ) -> Result<BulkCheckResultWithNotification, AppError> {
        // Step 1: Fetch settings upfront for notification check
        let settings = SettingService::get(conn).await?;

        // Step 2: Fetch all products
        let products = ProductRepository::find_all(conn).await?;

        // Step 3: Check each product with rate limiting (emitting progress events)
        let results = Self::check_products_with_rate_limit(conn, &products, app).await;

        // Step 4: Build summary from results
        let summary = Self::build_summary_from_results(products.len(), results);

        // Step 5: Emit completion event (ignore errors to avoid blocking)
        let _ = app.emit(
            EVENT_CHECK_COMPLETE,
            CheckCompleteEvent {
                summary: summary.clone(),
            },
        );

        // Step 6: Build notification if applicable (using NotificationService)
        let notification = Self::build_bulk_notification_with_settings(&settings, &summary);

        Ok(BulkCheckResultWithNotification {
            summary,
            notification,
        })
    }

    /// Build notification data for a bulk check using pre-fetched settings
    ///
    /// Delegates to NotificationService for actual notification composition.
    fn build_bulk_notification_with_settings(
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

    /// Tests for check_product method
    mod check_product_tests {
        use super::*;

        #[tokio::test]
        async fn test_check_product_not_found() {
            let conn = setup_availability_db().await;
            let fake_id = Uuid::new_v4();

            let result = AvailabilityService::check_product(&conn, fake_id).await;

            assert!(result.is_err());
            assert!(matches!(result, Err(AppError::NotFound(_))));
        }
    }

    /// Tests for is_back_in_stock logic
    mod back_in_stock_tests {
        use super::*;

        #[test]
        fn test_from_out_of_stock() {
            let previous = Some("out_of_stock".to_string());
            assert!(AvailabilityService::is_back_in_stock(&previous, "in_stock"));
        }

        #[test]
        fn test_from_back_order() {
            let previous = Some("back_order".to_string());
            assert!(AvailabilityService::is_back_in_stock(&previous, "in_stock"));
        }

        #[test]
        fn test_from_unknown() {
            let previous = Some("unknown".to_string());
            assert!(AvailabilityService::is_back_in_stock(&previous, "in_stock"));
        }

        #[test]
        fn test_already_in_stock() {
            let previous = Some("in_stock".to_string());
            assert!(!AvailabilityService::is_back_in_stock(
                &previous, "in_stock"
            ));
        }

        #[test]
        fn test_still_out_of_stock() {
            let previous = Some("out_of_stock".to_string());
            assert!(!AvailabilityService::is_back_in_stock(
                &previous,
                "out_of_stock"
            ));
        }

        #[test]
        fn test_no_previous() {
            let previous: Option<String> = None;
            assert!(!AvailabilityService::is_back_in_stock(
                &previous, "in_stock"
            ));
        }

        #[test]
        fn test_to_out_of_stock() {
            let previous = Some("in_stock".to_string());
            assert!(!AvailabilityService::is_back_in_stock(
                &previous,
                "out_of_stock"
            ));
        }

        #[test]
        fn test_to_back_order() {
            let previous = Some("in_stock".to_string());
            assert!(!AvailabilityService::is_back_in_stock(
                &previous,
                "back_order"
            ));
        }

        #[test]
        fn test_to_unknown() {
            let previous = Some("in_stock".to_string());
            assert!(!AvailabilityService::is_back_in_stock(&previous, "unknown"));
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
                status: "in_stock".to_string(),
                previous_status: Some("out_of_stock".to_string()),
                is_back_in_stock: true,
                price_cents: Some(78900),
                price_currency: Some("USD".to_string()),
                today_average_price_cents: Some(78900),
                yesterday_average_price_cents: Some(89900),
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
        fn test_with_error() {
            let result = BulkCheckResult {
                product_id: "error-id".to_string(),
                product_name: "Error Product".to_string(),
                status: "unknown".to_string(),
                previous_status: None,
                is_back_in_stock: false,
                price_cents: None,
                price_currency: None,
                today_average_price_cents: None,
                yesterday_average_price_cents: None,
                is_price_drop: false,
                error: Some("Failed to fetch".to_string()),
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
                status: "in_stock".to_string(),
                previous_status: Some("out_of_stock".to_string()),
                is_back_in_stock: true,
                price_cents: Some(78900),
                price_currency: Some("USD".to_string()),
                today_average_price_cents: None,
                yesterday_average_price_cents: None,
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
                daily_comparison: DailyPriceComparison {
                    today_average_cents: Some(78900),
                    yesterday_average_cents: Some(89900),
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
                price_cents: None,
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

    /// Tests for BulkCheckResultWithNotification struct
    mod bulk_check_result_with_notification_tests {
        use super::*;

        #[test]
        fn test_serialize() {
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
    }

    /// Tests for get_daily_price_comparison method
    mod daily_price_comparison_tests {
        use super::*;
        use crate::test_utils::{create_test_product, setup_availability_db};

        #[tokio::test]
        async fn test_no_data() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            let comparison = AvailabilityService::get_daily_price_comparison(&conn, product_id)
                .await
                .unwrap();

            assert_eq!(comparison.today_average_cents, None);
            assert_eq!(comparison.yesterday_average_cents, None);
        }

        #[tokio::test]
        async fn test_today_only() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            // Create check today
            AvailabilityCheckRepository::create(
                &conn,
                Uuid::new_v4(),
                product_id,
                CreateCheckParams {
                    status: "in_stock".to_string(),
                    price_cents: Some(15000),
                    price_currency: Some("USD".to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

            let comparison = AvailabilityService::get_daily_price_comparison(&conn, product_id)
                .await
                .unwrap();

            assert_eq!(comparison.today_average_cents, Some(15000));
            assert_eq!(comparison.yesterday_average_cents, None);
        }
    }
}
