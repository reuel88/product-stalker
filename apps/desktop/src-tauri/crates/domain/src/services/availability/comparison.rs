//! Price comparison and stock transition detection.

use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::entities::availability_check::AvailabilityStatus;
use crate::repositories::{AvailabilityCheckRepository, CurrencyAverageResult};
use product_stalker_core::AppError;

use super::types::DailyPriceComparison;
use super::AvailabilityService;

/// Rolling 24-hour time windows for daily price comparison.
/// "Today" = last 24 hours, "Yesterday" = 24-48 hours ago.
fn daily_time_windows() -> (DateTime<Utc>, DateTime<Utc>, DateTime<Utc>) {
    let now = Utc::now();
    let twenty_four_hours_ago = now - chrono::Duration::hours(24);
    let forty_eight_hours_ago = now - chrono::Duration::hours(48);
    (now, twenty_four_hours_ago, forty_eight_hours_ago)
}

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

    /// Re-normalize per-currency average prices to the preferred currency.
    ///
    /// Takes per-currency averages (from `get_original_averages_by_currency_for_period`)
    /// and converts each to the preferred currency using **today's** exchange rates,
    /// then computes a weighted average (by check count).
    ///
    /// This ensures both today and yesterday windows use the same exchange rate,
    /// eliminating percentage noise from rate fluctuations.
    async fn renormalize_averages(
        conn: &DatabaseConnection,
        averages: &[CurrencyAverageResult],
        preferred_currency: &str,
    ) -> Result<Option<i64>, AppError> {
        if averages.is_empty() {
            return Ok(None);
        }

        let mut total_normalized = 0.0;
        let mut total_count = 0i64;

        for avg in averages {
            let rate = match product_stalker_core::services::ExchangeRateService::get_rate(
                conn,
                &avg.price_currency,
                preferred_currency,
            )
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    log::warn!(
                        "Skipping currency {} in daily comparison: {}",
                        avg.price_currency,
                        e
                    );
                    continue;
                }
            };

            let from_exp = crate::services::currency::currency_exponent(&avg.price_currency) as i32;
            let to_exp = crate::services::currency::currency_exponent(preferred_currency) as i32;

            // Convert average from original minor units to preferred minor units
            let avg_in_major = avg.avg_price / 10_f64.powi(from_exp);
            let converted_major = avg_in_major * rate;
            let converted_minor = converted_major * 10_f64.powi(to_exp);

            total_normalized += converted_minor * avg.check_count as f64;
            total_count += avg.check_count;
        }

        if total_count == 0 {
            return Ok(None);
        }

        Ok(Some((total_normalized / total_count as f64).round() as i64))
    }

    /// Get today's and yesterday's average prices for comparison.
    ///
    /// Uses rolling 24-hour windows instead of calendar dates for timezone
    /// resilience. "Today" = last 24 hours, "Yesterday" = 24-48 hours ago.
    ///
    /// Both windows are re-normalized to the preferred currency using **today's**
    /// exchange rates, so the resulting percentage reflects actual price changes
    /// and not exchange rate fluctuations.
    ///
    /// Returns `None` for either average if no priced checks exist in that window.
    pub async fn get_daily_price_comparison(
        conn: &DatabaseConnection,
        product_id: Uuid,
        preferred_currency: &str,
    ) -> Result<DailyPriceComparison, AppError> {
        let (now, yesterday_start, day_before_start) = daily_time_windows();

        // Get per-currency original averages for both windows
        let today_averages =
            AvailabilityCheckRepository::get_original_averages_by_currency_for_period(
                conn,
                product_id,
                yesterday_start,
                now,
            )
            .await?;
        let yesterday_averages =
            AvailabilityCheckRepository::get_original_averages_by_currency_for_period(
                conn,
                product_id,
                day_before_start,
                yesterday_start,
            )
            .await?;

        // Re-normalize both using today's exchange rates
        let today_average =
            Self::renormalize_averages(conn, &today_averages, preferred_currency).await?;
        let yesterday_average =
            Self::renormalize_averages(conn, &yesterday_averages, preferred_currency).await?;

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
        let (now, yesterday_start, day_before_start) = daily_time_windows();

        let today_average =
            AvailabilityCheckRepository::get_average_price_for_period_by_product_retailer(
                conn,
                product_retailer_id,
                yesterday_start,
                now,
            )
            .await?;
        let yesterday_average =
            AvailabilityCheckRepository::get_average_price_for_period_by_product_retailer(
                conn,
                product_retailer_id,
                day_before_start,
                yesterday_start,
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

        /// Checks are created with "USD" currency by default, so using "USD"
        /// as preferred_currency gives identity-rate (1.0) without needing
        /// exchange rates in the test DB.
        const PREFERRED: &str = "USD";

        #[tokio::test]
        async fn test_no_data() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            let comparison =
                AvailabilityService::get_daily_price_comparison(&conn, product_id, PREFERRED)
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

            let comparison =
                AvailabilityService::get_daily_price_comparison(&conn, product_id, PREFERRED)
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

            let comparison =
                AvailabilityService::get_daily_price_comparison(&conn, product_id, PREFERRED)
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

            let comparison =
                AvailabilityService::get_daily_price_comparison(&conn, product_id, PREFERRED)
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

            let comparison =
                AvailabilityService::get_daily_price_comparison(&conn, product_id, PREFERRED)
                    .await
                    .unwrap();

            assert_eq!(comparison.today_average_minor_units, None);
            assert_eq!(comparison.yesterday_average_minor_units, None);
        }
    }

    /// Tests for re-normalization logic in daily price comparison.
    ///
    /// These verify that exchange rate fluctuations don't produce false
    /// percentage changes when actual prices haven't changed.
    mod renormalization_tests {
        use super::*;
        use crate::repositories::AvailabilityCheckRepository;
        use crate::test_utils::{create_test_product, setup_availability_db_with_exchange_rates};
        use product_stalker_core::repositories::ExchangeRateRepository;

        /// Helper: insert a check with a specific currency and original price
        async fn insert_check(
            conn: &sea_orm::DatabaseConnection,
            product_id: uuid::Uuid,
            price: i64,
            currency: &str,
            hours_ago: i64,
        ) {
            use sea_orm::{ActiveModelTrait, Set};
            let model = crate::entities::prelude::AvailabilityCheckActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                product_id: Set(product_id),
                product_retailer_id: Set(None),
                status: Set("in_stock".to_string()),
                raw_availability: Set(None),
                error_message: Set(None),
                checked_at: Set(chrono::Utc::now() - chrono::Duration::hours(hours_ago)),
                price_minor_units: Set(Some(price)),
                price_currency: Set(Some(currency.to_string())),
                raw_price: Set(None),
                normalized_price_minor_units: Set(None),
                normalized_currency: Set(None),
            };
            model.insert(conn).await.unwrap();
        }

        #[tokio::test]
        async fn test_same_price_different_rates_yields_zero_change() {
            // Scenario: EUR product costs 10000 (EUR 100.00) both days.
            // Exchange rate EUR->AUD doesn't matter because both windows
            // use the SAME rate, so percentage should be 0%.
            let conn = setup_availability_db_with_exchange_rates().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            // Set up exchange rate: EUR->AUD = 1.67
            ExchangeRateRepository::upsert_rate(&conn, "EUR", "AUD", 1.67, "api")
                .await
                .unwrap();

            // Today: EUR 100.00 (6 hours ago)
            insert_check(&conn, product_id, 10000, "EUR", 6).await;
            // Yesterday: EUR 100.00 (30 hours ago)
            insert_check(&conn, product_id, 10000, "EUR", 30).await;

            let comparison =
                AvailabilityService::get_daily_price_comparison(&conn, product_id, "AUD")
                    .await
                    .unwrap();

            // Both should be converted at the same rate (1.67):
            // 10000 EUR cents = EUR 100.00 * 1.67 = AUD 167.00 = 16700 AUD cents
            assert_eq!(comparison.today_average_minor_units, Some(16700));
            assert_eq!(comparison.yesterday_average_minor_units, Some(16700));
        }

        #[tokio::test]
        async fn test_real_price_change_reflected_correctly() {
            // Scenario: EUR product was EUR 100.00 yesterday, EUR 110.00 today.
            // This is a real 10% increase that should be visible.
            let conn = setup_availability_db_with_exchange_rates().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            ExchangeRateRepository::upsert_rate(&conn, "EUR", "AUD", 1.67, "api")
                .await
                .unwrap();

            // Today: EUR 110.00
            insert_check(&conn, product_id, 11000, "EUR", 6).await;
            // Yesterday: EUR 100.00
            insert_check(&conn, product_id, 10000, "EUR", 30).await;

            let comparison =
                AvailabilityService::get_daily_price_comparison(&conn, product_id, "AUD")
                    .await
                    .unwrap();

            // Today: 11000 EUR cents = EUR 110.00 * 1.67 = AUD 183.70 = 18370
            assert_eq!(comparison.today_average_minor_units, Some(18370));
            // Yesterday: 10000 EUR cents = EUR 100.00 * 1.67 = AUD 167.00 = 16700
            assert_eq!(comparison.yesterday_average_minor_units, Some(16700));
        }

        #[tokio::test]
        async fn test_mixed_currencies_normalized_correctly() {
            // Scenario: Two retailers — one EUR, one USD. Preferred = AUD.
            // Both use the same exchange rate for today AND yesterday.
            let conn = setup_availability_db_with_exchange_rates().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            ExchangeRateRepository::upsert_rate(&conn, "EUR", "AUD", 1.67, "api")
                .await
                .unwrap();
            ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.58, "api")
                .await
                .unwrap();

            // Today: EUR 100.00 and USD 90.00 (6 hours ago each)
            insert_check(&conn, product_id, 10000, "EUR", 6).await;
            insert_check(&conn, product_id, 9000, "USD", 6).await;

            // Yesterday: same prices
            insert_check(&conn, product_id, 10000, "EUR", 30).await;
            insert_check(&conn, product_id, 9000, "USD", 30).await;

            let comparison =
                AvailabilityService::get_daily_price_comparison(&conn, product_id, "AUD")
                    .await
                    .unwrap();

            // Both days should produce the same weighted average since prices
            // and rates are identical. The important thing is today == yesterday.
            assert_eq!(
                comparison.today_average_minor_units,
                comparison.yesterday_average_minor_units
            );
        }

        #[tokio::test]
        async fn test_preferred_currency_same_as_original_no_rates_needed() {
            // Scenario: All checks are in AUD, preferred is AUD.
            // Identity rate (1.0) is used, no exchange_rates row needed.
            let conn = setup_availability_db_with_exchange_rates().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            insert_check(&conn, product_id, 15000, "AUD", 6).await;
            insert_check(&conn, product_id, 20000, "AUD", 30).await;

            let comparison =
                AvailabilityService::get_daily_price_comparison(&conn, product_id, "AUD")
                    .await
                    .unwrap();

            assert_eq!(comparison.today_average_minor_units, Some(15000));
            assert_eq!(comparison.yesterday_average_minor_units, Some(20000));
        }

        #[tokio::test]
        async fn test_unknown_currency_skipped_gracefully() {
            // Scenario: One check in an unknown currency (no rate) — should be skipped.
            let conn = setup_availability_db_with_exchange_rates().await;
            let product_id = create_test_product(&conn, "https://example.com").await;

            // AUD check (will resolve with identity rate)
            insert_check(&conn, product_id, 15000, "AUD", 6).await;
            // XYZ check (no rate available — should be skipped)
            insert_check(&conn, product_id, 99999, "XYZ", 6).await;

            let comparison =
                AvailabilityService::get_daily_price_comparison(&conn, product_id, "AUD")
                    .await
                    .unwrap();

            // Only the AUD check should contribute
            assert_eq!(comparison.today_average_minor_units, Some(15000));
        }
    }
}
