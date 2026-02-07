use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbBackend, EntityTrait, FromQueryResult,
    QueryFilter, QueryOrder, Set, Statement,
};
use serde::Serialize;
use uuid::Uuid;

use crate::entities::prelude::*;
use crate::error::AppError;

/// Result of comparing today's average price vs yesterday's average price
#[derive(Debug, Clone, Serialize, Default)]
pub struct DailyPriceComparison {
    pub today_average_cents: Option<i64>,
    pub yesterday_average_cents: Option<i64>,
}

/// Helper struct for parsing SQLite AVG query results
#[derive(Debug, FromQueryResult)]
struct AveragePriceResult {
    avg_price: Option<f64>,
}

/// Repository for availability check data access
pub struct AvailabilityCheckRepository;

/// Parameters for creating an availability check
#[derive(Default)]
pub struct CreateCheckParams {
    pub status: String,
    pub raw_availability: Option<String>,
    pub error_message: Option<String>,
    pub price_cents: Option<i64>,
    pub price_currency: Option<String>,
    pub raw_price: Option<String>,
}

impl AvailabilityCheckRepository {
    /// Create a new availability check record
    pub async fn create(
        conn: &DatabaseConnection,
        id: Uuid,
        product_id: Uuid,
        params: CreateCheckParams,
    ) -> Result<AvailabilityCheckModel, AppError> {
        let now = chrono::Utc::now();

        let active_model = AvailabilityCheckActiveModel {
            id: Set(id),
            product_id: Set(product_id),
            status: Set(params.status),
            raw_availability: Set(params.raw_availability),
            error_message: Set(params.error_message),
            checked_at: Set(now),
            price_cents: Set(params.price_cents),
            price_currency: Set(params.price_currency),
            raw_price: Set(params.raw_price),
        };

        let check = active_model.insert(conn).await?;
        Ok(check)
    }

    /// Find the previous price for a product (for price drop detection)
    /// Returns the most recent check that has a price
    pub async fn find_previous_price(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<Option<i64>, AppError> {
        let check = AvailabilityCheck::find()
            .filter(AvailabilityCheckColumn::ProductId.eq(product_id))
            .filter(AvailabilityCheckColumn::PriceCents.is_not_null())
            .order_by_desc(AvailabilityCheckColumn::CheckedAt)
            .one(conn)
            .await?;
        Ok(check.and_then(|c| c.price_cents))
    }

    /// Find the most recent availability check for a product
    pub async fn find_latest_for_product(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<Option<AvailabilityCheckModel>, AppError> {
        let check = AvailabilityCheck::find()
            .filter(AvailabilityCheckColumn::ProductId.eq(product_id))
            .order_by_desc(AvailabilityCheckColumn::CheckedAt)
            .one(conn)
            .await?;
        Ok(check)
    }

    /// Find all availability checks for a product, ordered by most recent first
    pub async fn find_all_for_product(
        conn: &DatabaseConnection,
        product_id: Uuid,
        limit: Option<u64>,
    ) -> Result<Vec<AvailabilityCheckModel>, AppError> {
        let mut query = AvailabilityCheck::find()
            .filter(AvailabilityCheckColumn::ProductId.eq(product_id))
            .order_by_desc(AvailabilityCheckColumn::CheckedAt);

        if let Some(limit) = limit {
            use sea_orm::QuerySelect;
            query = query.limit(limit);
        }

        let checks = query.all(conn).await?;
        Ok(checks)
    }

    /// Get average price for a product on a specific date (UTC)
    ///
    /// Uses SUBSTR to extract date from ISO8601 timestamp and AVG() to calculate average.
    /// Returns None if no price data exists for that date.
    pub async fn get_average_price_for_date(
        conn: &DatabaseConnection,
        product_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<i64>, AppError> {
        use sea_orm::Value;

        let date_str = date.format("%Y-%m-%d").to_string();

        // UUID is stored as raw bytes in SQLite, so we pass it as bytes
        let result = AveragePriceResult::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            r#"
                SELECT AVG(price_cents) as avg_price
                FROM availability_checks
                WHERE product_id = ?
                  AND SUBSTR(checked_at, 1, 10) = ?
                  AND price_cents IS NOT NULL
            "#,
            [
                Value::Bytes(Some(Box::new(product_id.as_bytes().to_vec()))),
                date_str.into(),
            ],
        ))
        .one(conn)
        .await?;

        Ok(result.and_then(|r| r.avg_price.map(|avg| avg.round() as i64)))
    }

    /// Get today's and yesterday's average prices for comparison
    ///
    /// Uses UTC dates for consistency. Returns a DailyPriceComparison struct
    /// containing both averages (or None if no data exists for that day).
    pub async fn get_daily_price_comparison(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<DailyPriceComparison, AppError> {
        let today = Utc::now().date_naive();
        let yesterday = today - chrono::Duration::days(1);

        let today_average = Self::get_average_price_for_date(conn, product_id, today).await?;
        let yesterday_average =
            Self::get_average_price_for_date(conn, product_id, yesterday).await?;

        Ok(DailyPriceComparison {
            today_average_cents: today_average,
            yesterday_average_cents: yesterday_average,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_test_product_default, setup_availability_db};

    #[tokio::test]
    async fn test_create_availability_check() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;
        let id = Uuid::new_v4();

        let check = AvailabilityCheckRepository::create(
            &conn,
            id,
            product_id,
            CreateCheckParams {
                status: "in_stock".to_string(),
                raw_availability: Some("http://schema.org/InStock".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(check.id, id);
        assert_eq!(check.product_id, product_id);
        assert_eq!(check.status, "in_stock");
        assert_eq!(
            check.raw_availability,
            Some("http://schema.org/InStock".to_string())
        );
        assert!(check.error_message.is_none());
    }

    #[tokio::test]
    async fn test_create_availability_check_with_error() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;
        let id = Uuid::new_v4();

        let check = AvailabilityCheckRepository::create(
            &conn,
            id,
            product_id,
            CreateCheckParams {
                status: "unknown".to_string(),
                error_message: Some("Failed to fetch page".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(check.status, "unknown");
        assert!(check.raw_availability.is_none());
        assert_eq!(
            check.error_message,
            Some("Failed to fetch page".to_string())
        );
    }

    #[tokio::test]
    async fn test_create_availability_check_with_price() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;
        let id = Uuid::new_v4();

        let check = AvailabilityCheckRepository::create(
            &conn,
            id,
            product_id,
            CreateCheckParams {
                status: "in_stock".to_string(),
                raw_availability: Some("http://schema.org/InStock".to_string()),
                price_cents: Some(78900),
                price_currency: Some("USD".to_string()),
                raw_price: Some("789.00".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(check.price_cents, Some(78900));
        assert_eq!(check.price_currency, Some("USD".to_string()));
        assert_eq!(check.raw_price, Some("789.00".to_string()));
    }

    #[tokio::test]
    async fn test_find_previous_price() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        // Create a check with price
        AvailabilityCheckRepository::create(
            &conn,
            Uuid::new_v4(),
            product_id,
            CreateCheckParams {
                status: "in_stock".to_string(),
                price_cents: Some(78900),
                price_currency: Some("USD".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let previous_price = AvailabilityCheckRepository::find_previous_price(&conn, product_id)
            .await
            .unwrap();

        assert_eq!(previous_price, Some(78900));
    }

    #[tokio::test]
    async fn test_find_previous_price_no_price() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        // Create a check without price
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

        let previous_price = AvailabilityCheckRepository::find_previous_price(&conn, product_id)
            .await
            .unwrap();

        assert_eq!(previous_price, None);
    }

    #[tokio::test]
    async fn test_find_latest_for_product() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        // Create multiple checks
        for i in 0..3 {
            let id = Uuid::new_v4();
            AvailabilityCheckRepository::create(
                &conn,
                id,
                product_id,
                CreateCheckParams {
                    status: if i == 2 {
                        "in_stock".to_string()
                    } else {
                        "out_of_stock".to_string()
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            // Small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let latest = AvailabilityCheckRepository::find_latest_for_product(&conn, product_id)
            .await
            .unwrap();

        assert!(latest.is_some());
        assert_eq!(latest.unwrap().status, "in_stock");
    }

    #[tokio::test]
    async fn test_find_latest_for_product_none() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        let latest = AvailabilityCheckRepository::find_latest_for_product(&conn, product_id)
            .await
            .unwrap();

        assert!(latest.is_none());
    }

    #[tokio::test]
    async fn test_find_all_for_product() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        // Create 5 checks
        for _ in 0..5 {
            let id = Uuid::new_v4();
            AvailabilityCheckRepository::create(
                &conn,
                id,
                product_id,
                CreateCheckParams {
                    status: "in_stock".to_string(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }

        let all = AvailabilityCheckRepository::find_all_for_product(&conn, product_id, None)
            .await
            .unwrap();

        assert_eq!(all.len(), 5);
    }

    #[tokio::test]
    async fn test_find_all_for_product_with_limit() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        // Create 5 checks
        for _ in 0..5 {
            let id = Uuid::new_v4();
            AvailabilityCheckRepository::create(
                &conn,
                id,
                product_id,
                CreateCheckParams {
                    status: "in_stock".to_string(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }

        let limited = AvailabilityCheckRepository::find_all_for_product(&conn, product_id, Some(3))
            .await
            .unwrap();

        assert_eq!(limited.len(), 3);
    }

    #[tokio::test]
    async fn test_find_all_for_product_empty() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        let all = AvailabilityCheckRepository::find_all_for_product(&conn, product_id, None)
            .await
            .unwrap();

        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn test_find_all_for_product_ordered_by_latest() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        // Create checks with different statuses to track order
        let statuses = ["out_of_stock", "back_order", "in_stock"];
        for status in statuses {
            let id = Uuid::new_v4();
            AvailabilityCheckRepository::create(
                &conn,
                id,
                product_id,
                CreateCheckParams {
                    status: status.to_string(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            // Small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let all = AvailabilityCheckRepository::find_all_for_product(&conn, product_id, None)
            .await
            .unwrap();

        assert_eq!(all.len(), 3);
        // Most recent should be first
        assert_eq!(all[0].status, "in_stock");
        assert_eq!(all[2].status, "out_of_stock");
    }

    #[tokio::test]
    async fn test_create_with_all_statuses() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        let statuses = ["in_stock", "out_of_stock", "back_order", "unknown"];
        for status in statuses {
            let id = Uuid::new_v4();
            let check = AvailabilityCheckRepository::create(
                &conn,
                id,
                product_id,
                CreateCheckParams {
                    status: status.to_string(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(check.status, status);
        }
    }

    #[tokio::test]
    async fn test_find_all_for_different_products() {
        let conn = setup_availability_db().await;

        // Create two products
        let product1_id =
            crate::test_utils::create_test_product(&conn, "https://product1.com").await;
        let product2_id =
            crate::test_utils::create_test_product(&conn, "https://product2.com").await;

        // Create checks for product1
        for _ in 0..3 {
            AvailabilityCheckRepository::create(
                &conn,
                Uuid::new_v4(),
                product1_id,
                CreateCheckParams {
                    status: "in_stock".to_string(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }

        // Create checks for product2
        for _ in 0..2 {
            AvailabilityCheckRepository::create(
                &conn,
                Uuid::new_v4(),
                product2_id,
                CreateCheckParams {
                    status: "out_of_stock".to_string(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }

        // Verify each product has correct number of checks
        let product1_checks =
            AvailabilityCheckRepository::find_all_for_product(&conn, product1_id, None)
                .await
                .unwrap();
        let product2_checks =
            AvailabilityCheckRepository::find_all_for_product(&conn, product2_id, None)
                .await
                .unwrap();

        assert_eq!(product1_checks.len(), 3);
        assert_eq!(product2_checks.len(), 2);
    }

    #[tokio::test]
    async fn test_find_latest_does_not_return_other_products_checks() {
        let conn = setup_availability_db().await;

        // Create two products
        let product1_id =
            crate::test_utils::create_test_product(&conn, "https://product1.com").await;
        let product2_id =
            crate::test_utils::create_test_product(&conn, "https://product2.com").await;

        // Create check for product1
        AvailabilityCheckRepository::create(
            &conn,
            Uuid::new_v4(),
            product1_id,
            CreateCheckParams {
                status: "in_stock".to_string(),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        // Check that product2 has no latest check
        let latest = AvailabilityCheckRepository::find_latest_for_product(&conn, product2_id)
            .await
            .unwrap();

        assert!(latest.is_none());
    }

    #[tokio::test]
    async fn test_create_with_raw_availability_variants() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        let raw_values = [
            "http://schema.org/InStock",
            "https://schema.org/OutOfStock",
            "InStock",
            "BackOrder",
        ];

        for raw in raw_values {
            let check = AvailabilityCheckRepository::create(
                &conn,
                Uuid::new_v4(),
                product_id,
                CreateCheckParams {
                    status: "in_stock".to_string(),
                    raw_availability: Some(raw.to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            assert_eq!(check.raw_availability, Some(raw.to_string()));
        }
    }

    #[tokio::test]
    async fn test_find_all_with_limit_zero() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        // Create some checks
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

        // Limit of 0 should return empty
        let limited = AvailabilityCheckRepository::find_all_for_product(&conn, product_id, Some(0))
            .await
            .unwrap();

        assert!(limited.is_empty());
    }

    #[tokio::test]
    async fn test_get_average_price_for_date_with_prices() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;
        let today = Utc::now().date_naive();

        // Create multiple checks with different prices today
        for price in [10000_i64, 20000, 30000] {
            AvailabilityCheckRepository::create(
                &conn,
                Uuid::new_v4(),
                product_id,
                CreateCheckParams {
                    status: "in_stock".to_string(),
                    price_cents: Some(price),
                    price_currency: Some("USD".to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }

        let average =
            AvailabilityCheckRepository::get_average_price_for_date(&conn, product_id, today)
                .await
                .unwrap();

        // Average of 10000, 20000, 30000 = 20000
        assert_eq!(average, Some(20000));
    }

    #[tokio::test]
    async fn test_get_average_price_for_date_no_prices() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;
        let today = Utc::now().date_naive();

        // Create check without price
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

        let average =
            AvailabilityCheckRepository::get_average_price_for_date(&conn, product_id, today)
                .await
                .unwrap();

        assert_eq!(average, None);
    }

    #[tokio::test]
    async fn test_get_average_price_for_date_different_date() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;
        let yesterday = Utc::now().date_naive() - chrono::Duration::days(1);

        // Create check today (not yesterday)
        AvailabilityCheckRepository::create(
            &conn,
            Uuid::new_v4(),
            product_id,
            CreateCheckParams {
                status: "in_stock".to_string(),
                price_cents: Some(10000),
                price_currency: Some("USD".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        // Query for yesterday should return None
        let average =
            AvailabilityCheckRepository::get_average_price_for_date(&conn, product_id, yesterday)
                .await
                .unwrap();

        assert_eq!(average, None);
    }

    #[tokio::test]
    async fn test_get_daily_price_comparison_no_data() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

        let comparison = AvailabilityCheckRepository::get_daily_price_comparison(&conn, product_id)
            .await
            .unwrap();

        assert_eq!(comparison.today_average_cents, None);
        assert_eq!(comparison.yesterday_average_cents, None);
    }

    #[tokio::test]
    async fn test_get_daily_price_comparison_today_only() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;

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

        let comparison = AvailabilityCheckRepository::get_daily_price_comparison(&conn, product_id)
            .await
            .unwrap();

        assert_eq!(comparison.today_average_cents, Some(15000));
        assert_eq!(comparison.yesterday_average_cents, None);
    }

    #[tokio::test]
    async fn test_get_average_price_rounds_correctly() {
        let conn = setup_availability_db().await;
        let product_id = create_test_product_default(&conn).await;
        let today = Utc::now().date_naive();

        // Create checks with prices that average to a non-integer
        // 10001 + 10002 = 20003 / 2 = 10001.5 -> rounds to 10002
        for price in [10001_i64, 10002] {
            AvailabilityCheckRepository::create(
                &conn,
                Uuid::new_v4(),
                product_id,
                CreateCheckParams {
                    status: "in_stock".to_string(),
                    price_cents: Some(price),
                    price_currency: Some("USD".to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        }

        let average =
            AvailabilityCheckRepository::get_average_price_for_date(&conn, product_id, today)
                .await
                .unwrap();

        // 10001.5 rounds to 10002
        assert_eq!(average, Some(10002));
    }
}
