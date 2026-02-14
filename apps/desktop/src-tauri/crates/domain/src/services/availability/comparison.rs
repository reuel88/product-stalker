//! Price comparison and stock transition detection.

use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::entities::availability_check::AvailabilityStatus;
use crate::repositories::AvailabilityCheckRepository;
use product_stalker_core::AppError;

use super::types::DailyPriceComparison;
use super::AvailabilityService;

impl AvailabilityService {
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

    /// Get today's and yesterday's average prices for comparison.
    ///
    /// Uses rolling 24-hour windows instead of calendar dates for timezone
    /// resilience. "Today" = last 24 hours, "Yesterday" = 24-48 hours ago.
    ///
    /// The averages include ALL checks in each window (including error checks
    /// that recorded no price, which are excluded by the SQL query). When a new
    /// check is performed, it should be called AFTER persisting the check so
    /// the new price is included in "today's" average.
    ///
    /// Returns `None` for either average if no priced checks exist in that window.
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

    /// Get daily price comparison for a specific product-retailer link.
    pub async fn get_daily_price_comparison_for_product_retailer(
        conn: &DatabaseConnection,
        product_retailer_id: Uuid,
    ) -> Result<DailyPriceComparison, AppError> {
        let now = chrono::Utc::now();
        let twenty_four_hours_ago = now - chrono::Duration::hours(24);
        let forty_eight_hours_ago = now - chrono::Duration::hours(48);

        let today_average =
            AvailabilityCheckRepository::get_average_price_for_period_by_product_retailer(
                conn,
                product_retailer_id,
                twenty_four_hours_ago,
                now,
            )
            .await?;
        let yesterday_average =
            AvailabilityCheckRepository::get_average_price_for_period_by_product_retailer(
                conn,
                product_retailer_id,
                forty_eight_hours_ago,
                twenty_four_hours_ago,
            )
            .await?;

        Ok(DailyPriceComparison {
            today_average_minor_units: today_average,
            yesterday_average_minor_units: yesterday_average,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
