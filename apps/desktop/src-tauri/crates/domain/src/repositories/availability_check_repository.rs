use chrono::{DateTime, Utc};
use product_stalker_core::AppError;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbBackend, EntityTrait, FromQueryResult,
    QueryFilter, QueryOrder, Set, Statement,
};
use uuid::Uuid;

use crate::entities::availability_check::AvailabilityStatus;
use crate::entities::prelude::*;

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
    pub status: AvailabilityStatus,
    pub raw_availability: Option<String>,
    pub error_message: Option<String>,
    pub price_minor_units: Option<i64>,
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
            status: Set(params.status.as_str().to_string()),
            raw_availability: Set(params.raw_availability),
            error_message: Set(params.error_message),
            checked_at: Set(now),
            price_minor_units: Set(params.price_minor_units),
            price_currency: Set(params.price_currency),
            raw_price: Set(params.raw_price),
        };

        let check = active_model.insert(conn).await?;
        Ok(check)
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

    /// Get average price for a product within a time period [from, to)
    ///
    /// Uses a rolling time window instead of calendar dates, making it
    /// timezone-agnostic. Returns None if no price data exists in the period.
    pub async fn get_average_price_for_period(
        conn: &DatabaseConnection,
        product_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Option<i64>, AppError> {
        use sea_orm::Value;

        let result = AveragePriceResult::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Sqlite,
            r#"
                SELECT AVG(price_minor_units) as avg_price
                FROM availability_checks
                WHERE product_id = ?
                  AND checked_at >= ?
                  AND checked_at < ?
                  AND price_minor_units IS NOT NULL
            "#,
            [
                Value::Uuid(Some(Box::new(product_id))),
                from.into(),
                to.into(),
            ],
        ))
        .one(conn)
        .await?;

        Ok(result.and_then(|r| r.avg_price.map(|avg| avg.round() as i64)))
    }
}

#[cfg(test)]
impl AvailabilityCheckRepository {
    /// Test helper: create an availability check with a specific timestamp
    pub async fn create_with_timestamp(
        conn: &DatabaseConnection,
        product_id: Uuid,
        price_minor_units: Option<i64>,
        checked_at: DateTime<Utc>,
    ) -> AvailabilityCheckModel {
        let active_model = AvailabilityCheckActiveModel {
            id: Set(Uuid::new_v4()),
            product_id: Set(product_id),
            status: Set("in_stock".to_string()),
            raw_availability: Set(None),
            error_message: Set(None),
            checked_at: Set(checked_at),
            price_minor_units: Set(price_minor_units),
            price_currency: Set(Some("USD".to_string())),
            raw_price: Set(None),
        };
        active_model.insert(conn).await.unwrap()
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
                status: AvailabilityStatus::InStock,
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
                status: AvailabilityStatus::Unknown,
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
                status: AvailabilityStatus::InStock,
                raw_availability: Some("http://schema.org/InStock".to_string()),
                price_minor_units: Some(78900),
                price_currency: Some("USD".to_string()),
                raw_price: Some("789.00".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(check.price_minor_units, Some(78900));
        assert_eq!(check.price_currency, Some("USD".to_string()));
        assert_eq!(check.raw_price, Some("789.00".to_string()));
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
                        AvailabilityStatus::InStock
                    } else {
                        AvailabilityStatus::OutOfStock
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
                    status: AvailabilityStatus::InStock,
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
                    status: AvailabilityStatus::InStock,
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

    mod average_price_period_tests {
        use super::*;
        use chrono::Duration;

        #[tokio::test]
        async fn test_no_data_returns_none() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product_default(&conn).await;
            let now = Utc::now();
            let from = now - Duration::hours(24);

            let result = AvailabilityCheckRepository::get_average_price_for_period(
                &conn, product_id, from, now,
            )
            .await
            .unwrap();

            assert_eq!(result, None);
        }

        #[tokio::test]
        async fn test_single_check_returns_price() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product_default(&conn).await;
            let now = Utc::now();
            let from = now - Duration::hours(24);
            let check_time = now - Duration::hours(12);

            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(10000),
                check_time,
            )
            .await;

            let result = AvailabilityCheckRepository::get_average_price_for_period(
                &conn, product_id, from, now,
            )
            .await
            .unwrap();

            assert_eq!(result, Some(10000));
        }

        #[tokio::test]
        async fn test_multiple_checks_returns_average() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product_default(&conn).await;
            let now = Utc::now();
            let from = now - Duration::hours(24);

            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(10000),
                now - Duration::hours(12),
            )
            .await;
            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(20000),
                now - Duration::hours(6),
            )
            .await;

            let result = AvailabilityCheckRepository::get_average_price_for_period(
                &conn, product_id, from, now,
            )
            .await
            .unwrap();

            assert_eq!(result, Some(15000));
        }

        #[tokio::test]
        async fn test_excludes_outside_range() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product_default(&conn).await;
            let now = Utc::now();
            let from = now - Duration::hours(24);

            // Inside range
            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(10000),
                now - Duration::hours(12),
            )
            .await;
            // Outside range (before)
            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(50000),
                now - Duration::hours(30),
            )
            .await;

            let result = AvailabilityCheckRepository::get_average_price_for_period(
                &conn, product_id, from, now,
            )
            .await
            .unwrap();

            assert_eq!(result, Some(10000));
        }

        #[tokio::test]
        async fn test_excludes_null_prices() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product_default(&conn).await;
            let now = Utc::now();
            let from = now - Duration::hours(24);

            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(10000),
                now - Duration::hours(12),
            )
            .await;
            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                None,
                now - Duration::hours(6),
            )
            .await;

            let result = AvailabilityCheckRepository::get_average_price_for_period(
                &conn, product_id, from, now,
            )
            .await
            .unwrap();

            assert_eq!(result, Some(10000));
        }

        #[tokio::test]
        async fn test_boundary_from_inclusive() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product_default(&conn).await;
            let now = Utc::now();
            let from = now - Duration::hours(24);

            AvailabilityCheckRepository::create_with_timestamp(
                &conn,
                product_id,
                Some(10000),
                from,
            )
            .await;

            let result = AvailabilityCheckRepository::get_average_price_for_period(
                &conn, product_id, from, now,
            )
            .await
            .unwrap();

            assert_eq!(result, Some(10000));
        }

        #[tokio::test]
        async fn test_boundary_to_exclusive() {
            let conn = setup_availability_db().await;
            let product_id = create_test_product_default(&conn).await;
            let now = Utc::now();
            let from = now - Duration::hours(24);

            // Check at exact "to" boundary should be excluded
            AvailabilityCheckRepository::create_with_timestamp(&conn, product_id, Some(10000), now)
                .await;

            let result = AvailabilityCheckRepository::get_average_price_for_period(
                &conn, product_id, from, now,
            )
            .await
            .unwrap();

            assert_eq!(result, None);
        }
    }
}
