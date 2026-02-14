//! Product availability checking and result processing.

use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::entities::availability_check::AvailabilityStatus;
use crate::entities::prelude::{AvailabilityCheckModel, ProductModel};
use crate::repositories::{
    AvailabilityCheckRepository, CreateCheckParams, ProductRepository, ProductRetailerRepository,
};
use crate::services::scraper::has_path_locale;
use crate::services::{NotificationService, ScraperService};
use product_stalker_core::AppError;

use super::types::{
    BulkCheckResult, CheckProcessingResult, CheckResultWithNotification, DailyPriceComparison,
    ProductCheckContext,
};
use super::AvailabilityService;

impl AvailabilityService {
    /// Build CreateCheckParams from a successful scraping result
    fn params_from_success(result: crate::services::scraper::ScrapingResult) -> CreateCheckParams {
        CreateCheckParams {
            status: result.status,
            raw_availability: result.raw_availability,
            error_message: None,
            price_minor_units: result.price.price_minor_units,
            price_currency: result.price.price_currency,
            raw_price: result.price.raw_price,
            product_retailer_id: None,
        }
    }

    /// Build CreateCheckParams from a scraping error
    fn params_from_error(error: &AppError) -> CreateCheckParams {
        CreateCheckParams {
            error_message: Some(error.to_string()),
            ..Default::default()
        }
    }

    /// Check the availability of a product by its ID using its deprecated URL field.
    ///
    /// Fetches the product's URL, scrapes the page for availability info,
    /// and stores the result in the database.
    /// Auto-sets the product's currency on first successful price scrape.
    pub async fn check_product(
        conn: &DatabaseConnection,
        product_id: Uuid,
        enable_headless: bool,
        allow_manual_verification: bool,
        session_cache_duration_days: i32,
    ) -> Result<AvailabilityCheckModel, AppError> {
        let product = ProductRepository::find_by_id(conn, product_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", product_id)))?;

        let url = product
            .url
            .as_deref()
            .ok_or_else(|| AppError::Validation("Product has no URL set".to_string()))?;

        let result = ScraperService::check_availability_with_headless(
            url,
            enable_headless,
            allow_manual_verification,
            conn,
            session_cache_duration_days,
        )
        .await;

        let params = match result {
            Ok(scraping_result) => {
                let params = Self::params_from_success(scraping_result);
                Self::auto_set_product_currency(
                    conn,
                    &product,
                    params.price_currency.as_deref(),
                    None,
                )
                .await;
                params
            }
            Err(e) => Self::params_from_error(&e),
        };

        AvailabilityCheckRepository::create(conn, Uuid::new_v4(), product_id, params).await
    }

    /// Check availability for a product-retailer link.
    ///
    /// Uses the product_retailer URL, stores results with both product_id and product_retailer_id.
    pub async fn check_product_retailer(
        conn: &DatabaseConnection,
        product_retailer_id: Uuid,
        enable_headless: bool,
        allow_manual_verification: bool,
        session_cache_duration_days: i32,
    ) -> Result<AvailabilityCheckModel, AppError> {
        let pr =
            crate::repositories::ProductRetailerRepository::find_by_id(conn, product_retailer_id)
                .await?
                .ok_or_else(|| {
                    AppError::NotFound(format!(
                        "Product retailer not found: {}",
                        product_retailer_id
                    ))
                })?;

        let product = ProductRepository::find_by_id(conn, pr.product_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", pr.product_id)))?;

        let result = ScraperService::check_availability_with_headless(
            &pr.url,
            enable_headless,
            allow_manual_verification,
            conn,
            session_cache_duration_days,
        )
        .await;

        let mut params = match result {
            Ok(scraping_result) => {
                let params = Self::params_from_success(scraping_result);
                Self::auto_set_product_currency(
                    conn,
                    &product,
                    params.price_currency.as_deref(),
                    Some(&pr.url),
                )
                .await;
                params
            }
            Err(e) => Self::params_from_error(&e),
        };

        params.product_retailer_id = Some(product_retailer_id);

        AvailabilityCheckRepository::create(conn, Uuid::new_v4(), pr.product_id, params).await
    }

    /// Auto-set product currency from scraped price data.
    ///
    /// If the product has no currency set and the scrape found one, saves it.
    /// If the product already has a different currency, checks if the URL has a path locale:
    /// - If path locale detected: Updates to the scraped currency (corrects old detection)
    /// - If no path locale: Keeps existing currency (might be user-set or genuinely ambiguous)
    async fn auto_set_product_currency(
        conn: &DatabaseConnection,
        product: &ProductModel,
        scraped_currency: Option<&str>,
        check_url: Option<&str>,
    ) {
        let Some(scraped) = scraped_currency else {
            return;
        };

        match &product.currency {
            None => {
                // No currency set - always update (existing behavior)
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
                // Currency mismatch detected

                // Special case: If the URL has a path locale pattern, the scraped
                // currency (from new logic) is more reliable than the stored one
                // (from old logic that didn't check path locales)
                if check_url
                    .or(product.url.as_deref())
                    .is_some_and(has_path_locale)
                {
                    log::info!(
                        "Correcting currency for product {} from {} to {} (path locale detected in URL)",
                        product.id,
                        existing,
                        scraped
                    );
                    let update = crate::repositories::ProductUpdateInput {
                        currency: Some(Some(scraped.to_string())),
                        ..Default::default()
                    };
                    if let Err(e) = ProductRepository::update(conn, product.clone(), update).await {
                        log::warn!("Failed to update currency: {}", e);
                    }
                } else {
                    // No path locale - keep existing (might be user-set or genuinely ambiguous)
                    log::warn!(
                        "Product {} has currency {} but scraped {}; keeping existing (no path locale found)",
                        product.id,
                        existing,
                        scraped
                    );
                }
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
        allow_manual_verification: bool,
        session_cache_duration_days: i32,
    ) -> (BulkCheckResult, CheckProcessingResult) {
        // Step 1: Get previous check context
        let context = match Self::get_product_check_context(conn, product.id).await {
            Ok(ctx) => ctx,
            Err(e) => return Self::build_context_error_result(product, e),
        };

        // Step 2: Perform the availability check
        let check_result = Self::check_product(
            conn,
            product.id,
            enable_headless,
            allow_manual_verification,
            session_cache_duration_days,
        )
        .await;

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

    /// Check a single product-retailer link and return the result.
    ///
    /// Used in bulk operations where we iterate product_retailers instead of products.
    pub async fn check_single_product_retailer(
        conn: &DatabaseConnection,
        product: &ProductModel,
        product_retailer: &crate::entities::prelude::ProductRetailerModel,
        enable_headless: bool,
        allow_manual_verification: bool,
        session_cache_duration_days: i32,
    ) -> (BulkCheckResult, CheckProcessingResult) {
        // Step 1: Get previous check context (based on product_retailer_id)
        let previous_check = AvailabilityCheckRepository::find_latest_for_product_retailer(
            conn,
            product_retailer.id,
        )
        .await;
        let context = match previous_check {
            Ok(check) => ProductCheckContext {
                previous_status: check.as_ref().map(|c| c.status_enum()),
            },
            Err(e) => return Self::build_context_error_result(product, e),
        };

        // Step 2: Perform the check via product_retailer
        let check_result = Self::check_product_retailer(
            conn,
            product_retailer.id,
            enable_headless,
            allow_manual_verification,
            session_cache_duration_days,
        )
        .await;

        // Step 3: Get daily price comparison for this product_retailer
        let daily_comparison =
            match Self::get_daily_price_comparison_for_product_retailer(conn, product_retailer.id)
                .await
            {
                Ok(dc) => dc,
                Err(e) => return Self::build_context_error_result(product, e),
            };

        // Step 4: Process result
        let result =
            Self::process_check_result(check_result, &context.previous_status, &daily_comparison);

        // Step 5: Build bulk result with retailer info
        let bulk_result =
            BulkCheckResult::from_processing_result(product, &result, &context, &daily_comparison)
                .with_retailer(product_retailer);

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

    /// Get the context needed before checking a product (previous status)
    pub async fn get_product_check_context(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<ProductCheckContext, AppError> {
        let previous_check = Self::get_latest(conn, product_id).await?;
        let previous_status = previous_check.as_ref().map(|c| c.status_enum());

        Ok(ProductCheckContext { previous_status })
    }

    /// Get the latest availability check for a product
    pub async fn get_latest(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<Option<AvailabilityCheckModel>, AppError> {
        AvailabilityCheckRepository::find_latest_for_product(conn, product_id).await
    }

    /// Get the cheapest current price across all retailers for a product
    pub async fn get_cheapest_current_price(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<Option<crate::repositories::CheapestPriceResult>, AppError> {
        AvailabilityCheckRepository::find_cheapest_current_price(conn, product_id).await
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
        enable_headless: bool,
        enable_notifications: bool,
        allow_manual_verification: bool,
        session_cache_duration_days: i32,
    ) -> Result<CheckResultWithNotification, AppError> {
        // Step 1: Get previous status before checking
        let previous_check = Self::get_latest(conn, product_id).await?;
        let previous_status = previous_check.map(|c| c.status_enum());

        // Step 2: Check retailers first, fall back to legacy product.url
        let retailers = ProductRetailerRepository::find_by_product_id(conn, product_id).await?;

        let (check, any_back_in_stock) = if retailers.is_empty() {
            // Legacy path: product has no retailer links, use product.url
            let check = Self::check_product(
                conn,
                product_id,
                enable_headless,
                allow_manual_verification,
                session_cache_duration_days,
            )
            .await?;
            let is_back = Self::is_back_in_stock(&previous_status, &check.status_enum());
            (check, is_back)
        } else {
            // Multi-retailer path: check all retailers, track back-in-stock per-retailer
            let mut last_check = None;
            let mut back_in_stock = false;
            for retailer in &retailers {
                let retailer_previous =
                    AvailabilityCheckRepository::find_latest_for_product_retailer(
                        conn,
                        retailer.id,
                    )
                    .await?
                    .map(|c| c.status_enum());

                let result = Self::check_product_retailer(
                    conn,
                    retailer.id,
                    enable_headless,
                    allow_manual_verification,
                    session_cache_duration_days,
                )
                .await?;

                if Self::is_back_in_stock(&retailer_previous, &result.status_enum()) {
                    back_in_stock = true;
                }
                last_check = Some(result);
            }
            (last_check.expect("retailers is non-empty"), back_in_stock)
        };

        // Step 3: Get daily price comparison (includes the new check in today's average)
        let daily_comparison = Self::get_daily_price_comparison(conn, product_id).await?;

        // Step 4: Determine if back in stock
        let is_back_in_stock = any_back_in_stock;

        // Step 5: Build notification if applicable (using NotificationService)
        let notification = NotificationService::build_single_notification(
            conn,
            product_id,
            enable_notifications,
            is_back_in_stock,
        )
        .await?;

        Ok(CheckResultWithNotification {
            check,
            notification,
            daily_comparison,
        })
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

            let result = AvailabilityService::check_product(&conn, fake_id, false, false, 14).await;

            assert!(result.is_err());
            assert!(matches!(result, Err(AppError::NotFound(_))));
        }
    }

    /// Tests for check_product_with_notification retailer routing
    mod check_with_notification_tests {
        use super::*;
        use crate::repositories::{
            CreateProductRepoParams, CreateProductRetailerParams, ProductRetailerRepository,
            RetailerRepository,
        };

        #[tokio::test]
        async fn test_check_product_with_notification_uses_retailers_when_present() {
            let conn = setup_availability_db().await;

            // Create a product with NO URL (post-migration state)
            let product_id = Uuid::new_v4();
            ProductRepository::create(
                &conn,
                product_id,
                CreateProductRepoParams {
                    name: "Multi-Retailer Product".to_string(),
                    url: None,
                    description: None,
                    notes: None,
                },
            )
            .await
            .unwrap();

            // Create a retailer and link it to the product
            let retailer = RetailerRepository::find_or_create_by_domain(&conn, "example.com")
                .await
                .unwrap();

            ProductRetailerRepository::create(
                &conn,
                Uuid::new_v4(),
                retailer.id,
                CreateProductRetailerParams {
                    product_id,
                    url: "https://example.com/product".to_string(),
                    label: None,
                },
            )
            .await
            .unwrap();

            // Should NOT fail with "Product has no URL set" — uses retailer path instead.
            // The scraping will fail (no network in tests), but the error is caught and
            // stored as a check result, so this should return Ok.
            let result = AvailabilityService::check_product_with_notification(
                &conn, product_id, false, false, false, 14,
            )
            .await;

            assert!(
                result.is_ok(),
                "Expected Ok but got: {:?}",
                result.unwrap_err()
            );

            // Verify a check was created (with error from failed scraping)
            let latest = AvailabilityService::get_latest(&conn, product_id)
                .await
                .unwrap();
            assert!(latest.is_some(), "A check record should have been created");
        }

        #[tokio::test]
        async fn test_multi_retailer_notification_checks_all_retailers() {
            let conn = setup_availability_db().await;

            // Create a product with NO URL
            let product_id = Uuid::new_v4();
            ProductRepository::create(
                &conn,
                product_id,
                CreateProductRepoParams {
                    name: "Multi-Retailer Product".to_string(),
                    url: None,
                    description: None,
                    notes: None,
                },
            )
            .await
            .unwrap();

            // Create two retailers linked to the product
            let retailer_a = RetailerRepository::find_or_create_by_domain(&conn, "shop-a.com")
                .await
                .unwrap();
            let retailer_b = RetailerRepository::find_or_create_by_domain(&conn, "shop-b.com")
                .await
                .unwrap();

            let pr_a_id = Uuid::new_v4();
            ProductRetailerRepository::create(
                &conn,
                pr_a_id,
                retailer_a.id,
                CreateProductRetailerParams {
                    product_id,
                    url: "https://shop-a.com/product".to_string(),
                    label: None,
                },
            )
            .await
            .unwrap();

            let pr_b_id = Uuid::new_v4();
            ProductRetailerRepository::create(
                &conn,
                pr_b_id,
                retailer_b.id,
                CreateProductRetailerParams {
                    product_id,
                    url: "https://shop-b.com/product".to_string(),
                    label: None,
                },
            )
            .await
            .unwrap();

            // Run the check — scraping fails for both, but both should get check records
            let result = AvailabilityService::check_product_with_notification(
                &conn, product_id, false, false, false, 14,
            )
            .await;

            assert!(
                result.is_ok(),
                "Expected Ok but got: {:?}",
                result.unwrap_err()
            );

            // Verify both retailers got check records
            let check_a =
                AvailabilityCheckRepository::find_latest_for_product_retailer(&conn, pr_a_id)
                    .await
                    .unwrap();
            let check_b =
                AvailabilityCheckRepository::find_latest_for_product_retailer(&conn, pr_b_id)
                    .await
                    .unwrap();

            assert!(check_a.is_some(), "Retailer A should have a check record");
            assert!(check_b.is_some(), "Retailer B should have a check record");

            // No back-in-stock notification (scraping failed → Unknown status)
            let result = result.unwrap();
            assert!(
                result.notification.is_none(),
                "No notification expected when scraping fails"
            );
        }

        #[tokio::test]
        async fn test_multi_retailer_no_false_positive_back_in_stock() {
            let conn = setup_availability_db().await;

            // Create a product with NO URL
            let product_id = Uuid::new_v4();
            ProductRepository::create(
                &conn,
                product_id,
                CreateProductRepoParams {
                    name: "Multi-Retailer Product".to_string(),
                    url: None,
                    description: None,
                    notes: None,
                },
            )
            .await
            .unwrap();

            // Create two retailers
            let retailer_a = RetailerRepository::find_or_create_by_domain(&conn, "shop-a.com")
                .await
                .unwrap();
            let retailer_b = RetailerRepository::find_or_create_by_domain(&conn, "shop-b.com")
                .await
                .unwrap();

            let pr_a_id = Uuid::new_v4();
            ProductRetailerRepository::create(
                &conn,
                pr_a_id,
                retailer_a.id,
                CreateProductRetailerParams {
                    product_id,
                    url: "https://shop-a.com/product".to_string(),
                    label: None,
                },
            )
            .await
            .unwrap();

            let pr_b_id = Uuid::new_v4();
            ProductRetailerRepository::create(
                &conn,
                pr_b_id,
                retailer_b.id,
                CreateProductRetailerParams {
                    product_id,
                    url: "https://shop-b.com/product".to_string(),
                    label: None,
                },
            )
            .await
            .unwrap();

            // Seed per-retailer previous checks with different statuses:
            // Retailer A was OutOfStock, Retailer B was InStock
            AvailabilityCheckRepository::create(
                &conn,
                Uuid::new_v4(),
                product_id,
                CreateCheckParams {
                    status: AvailabilityStatus::OutOfStock,
                    product_retailer_id: Some(pr_a_id),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

            AvailabilityCheckRepository::create(
                &conn,
                Uuid::new_v4(),
                product_id,
                CreateCheckParams {
                    status: AvailabilityStatus::InStock,
                    product_retailer_id: Some(pr_b_id),
                    ..Default::default()
                },
            )
            .await
            .unwrap();

            // Run the check — scraping fails (Unknown status), neither retailer transitions to InStock
            let result = AvailabilityService::check_product_with_notification(
                &conn, product_id, false, true, false, 14,
            )
            .await;

            assert!(
                result.is_ok(),
                "Expected Ok but got: {:?}",
                result.unwrap_err()
            );

            // No notification: retailer A was OutOfStock→Unknown (not InStock),
            // retailer B was InStock→Unknown (not a back-in-stock transition)
            let result = result.unwrap();
            assert!(
                result.notification.is_none(),
                "No notification expected when no retailer transitions to InStock"
            );
        }

        #[tokio::test]
        async fn test_check_product_with_notification_no_url_no_retailers_fails() {
            let conn = setup_availability_db().await;

            // Create a product with NO URL and NO retailers
            let product_id = Uuid::new_v4();
            ProductRepository::create(
                &conn,
                product_id,
                CreateProductRepoParams {
                    name: "No URL Product".to_string(),
                    url: None,
                    description: None,
                    notes: None,
                },
            )
            .await
            .unwrap();

            // Should fail with the legacy "Product has no URL set" validation error
            let result = AvailabilityService::check_product_with_notification(
                &conn, product_id, false, false, false, 14,
            )
            .await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(
                matches!(&err, AppError::Validation(msg) if msg.contains("no URL")),
                "Expected Validation error about missing URL, got: {err:?}"
            );
        }
    }

    /// Tests for auto_set_product_currency method
    mod auto_set_currency_tests {
        use super::*;
        use crate::repositories::{CreateProductRepoParams, ProductRepository};
        use crate::test_utils::setup_availability_db;

        async fn create_product_with_url(
            conn: &DatabaseConnection,
            name: &str,
            url: &str,
        ) -> ProductModel {
            let id = Uuid::new_v4();
            ProductRepository::create(
                conn,
                id,
                CreateProductRepoParams {
                    name: name.to_string(),
                    url: Some(url.to_string()),
                    description: None,
                    notes: None,
                },
            )
            .await
            .unwrap();
            ProductRepository::find_by_id(conn, id)
                .await
                .unwrap()
                .unwrap()
        }

        #[tokio::test]
        async fn test_sets_currency_on_first_check() {
            let conn = setup_availability_db().await;
            let product = create_product_with_url(
                &conn,
                "Test Product",
                "https://example.com/en-au/products/test",
            )
            .await;

            assert_eq!(product.currency, None);

            AvailabilityService::auto_set_product_currency(&conn, &product, Some("AUD"), None)
                .await;

            let updated = ProductRepository::find_by_id(&conn, product.id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(updated.currency, Some("AUD".to_string()));
        }

        #[tokio::test]
        async fn test_keeps_matching_currency() {
            let conn = setup_availability_db().await;
            let product = create_product_with_url(
                &conn,
                "Test Product",
                "https://example.com/en-au/products/test",
            )
            .await;

            // Set initial currency
            AvailabilityService::auto_set_product_currency(&conn, &product, Some("AUD"), None)
                .await;

            let product = ProductRepository::find_by_id(&conn, product.id)
                .await
                .unwrap()
                .unwrap();

            // Try to set the same currency again
            AvailabilityService::auto_set_product_currency(&conn, &product, Some("AUD"), None)
                .await;

            let updated = ProductRepository::find_by_id(&conn, product.id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(updated.currency, Some("AUD".to_string()));
        }

        #[tokio::test]
        async fn test_corrects_wrong_currency_when_path_locale_present() {
            let conn = setup_availability_db().await;

            // Create product with /en-au/ path
            let product = create_product_with_url(
                &conn,
                "Reyllen Backpack",
                "https://reyllen.com/en-au/products/backpack",
            )
            .await;

            // Simulate old scraper setting wrong currency (GBP instead of AUD)
            let update = crate::repositories::ProductUpdateInput {
                currency: Some(Some("GBP".to_string())),
                ..Default::default()
            };
            ProductRepository::update(&conn, product.clone(), update)
                .await
                .unwrap();

            let product = ProductRepository::find_by_id(&conn, product.id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(product.currency, Some("GBP".to_string()));

            // New scraper detects AUD - should auto-correct because path locale is present
            AvailabilityService::auto_set_product_currency(&conn, &product, Some("AUD"), None)
                .await;

            let updated = ProductRepository::find_by_id(&conn, product.id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(updated.currency, Some("AUD".to_string()));
        }

        #[tokio::test]
        async fn test_keeps_existing_currency_when_no_path_locale() {
            let conn = setup_availability_db().await;

            // Create product WITHOUT path locale
            let product =
                create_product_with_url(&conn, "Test Product", "https://example.com/products/test")
                    .await;

            // Set currency to USD
            let update = crate::repositories::ProductUpdateInput {
                currency: Some(Some("USD".to_string())),
                ..Default::default()
            };
            ProductRepository::update(&conn, product.clone(), update)
                .await
                .unwrap();

            let product = ProductRepository::find_by_id(&conn, product.id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(product.currency, Some("USD".to_string()));

            // Scraper detects GBP - should NOT override (no path locale to guide us)
            AvailabilityService::auto_set_product_currency(&conn, &product, Some("GBP"), None)
                .await;

            let updated = ProductRepository::find_by_id(&conn, product.id)
                .await
                .unwrap()
                .unwrap();
            // Should still be USD (not corrected)
            assert_eq!(updated.currency, Some("USD".to_string()));
        }

        #[tokio::test]
        async fn test_corrects_multiple_path_locales() {
            let conn = setup_availability_db().await;

            // Test /en-nz/ -> NZD correction
            let product_nz = create_product_with_url(
                &conn,
                "NZ Product",
                "https://example.com/en-nz/products/test",
            )
            .await;

            let update = crate::repositories::ProductUpdateInput {
                currency: Some(Some("GBP".to_string())),
                ..Default::default()
            };
            ProductRepository::update(&conn, product_nz.clone(), update)
                .await
                .unwrap();

            let product_nz = ProductRepository::find_by_id(&conn, product_nz.id)
                .await
                .unwrap()
                .unwrap();

            AvailabilityService::auto_set_product_currency(&conn, &product_nz, Some("NZD"), None)
                .await;

            let updated_nz = ProductRepository::find_by_id(&conn, product_nz.id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(updated_nz.currency, Some("NZD".to_string()));

            // Test /en-gb/ -> GBP correction
            let product_gb = create_product_with_url(
                &conn,
                "GB Product",
                "https://example.com/en-gb/products/test",
            )
            .await;

            let update = crate::repositories::ProductUpdateInput {
                currency: Some(Some("USD".to_string())),
                ..Default::default()
            };
            ProductRepository::update(&conn, product_gb.clone(), update)
                .await
                .unwrap();

            let product_gb = ProductRepository::find_by_id(&conn, product_gb.id)
                .await
                .unwrap()
                .unwrap();

            AvailabilityService::auto_set_product_currency(&conn, &product_gb, Some("GBP"), None)
                .await;

            let updated_gb = ProductRepository::find_by_id(&conn, product_gb.id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(updated_gb.currency, Some("GBP".to_string()));
        }

        #[tokio::test]
        async fn test_corrects_currency_via_check_url_when_product_has_no_url() {
            let conn = setup_availability_db().await;

            // Create product with NO URL (multi-retailer scenario)
            let id = Uuid::new_v4();
            ProductRepository::create(
                &conn,
                id,
                CreateProductRepoParams {
                    name: "Multi-Retailer Product".to_string(),
                    url: None,
                    description: None,
                    notes: None,
                },
            )
            .await
            .unwrap();

            let product = ProductRepository::find_by_id(&conn, id)
                .await
                .unwrap()
                .unwrap();

            // Set initial currency to GBP (simulating old detection)
            let update = crate::repositories::ProductUpdateInput {
                currency: Some(Some("GBP".to_string())),
                ..Default::default()
            };
            ProductRepository::update(&conn, product.clone(), update)
                .await
                .unwrap();

            let product = ProductRepository::find_by_id(&conn, id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(product.currency, Some("GBP".to_string()));

            // Call with check_url containing a path locale — should correct to AUD
            AvailabilityService::auto_set_product_currency(
                &conn,
                &product,
                Some("AUD"),
                Some("https://example.com/en-au/products/item"),
            )
            .await;

            let updated = ProductRepository::find_by_id(&conn, id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(updated.currency, Some("AUD".to_string()));
        }
    }
}
