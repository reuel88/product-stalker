use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::entities::prelude::*;
use crate::error::AppError;

/// Repository for availability check data access
pub struct AvailabilityCheckRepository;

impl AvailabilityCheckRepository {
    /// Create a new availability check record
    pub async fn create(
        conn: &DatabaseConnection,
        id: Uuid,
        product_id: Uuid,
        status: &str,
        raw_availability: Option<String>,
        error_message: Option<String>,
    ) -> Result<AvailabilityCheckModel, AppError> {
        let now = chrono::Utc::now();

        let active_model = AvailabilityCheckActiveModel {
            id: Set(id),
            product_id: Set(product_id),
            status: Set(status.to_string()),
            raw_availability: Set(raw_availability),
            error_message: Set(error_message),
            checked_at: Set(now),
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
            "in_stock",
            Some("http://schema.org/InStock".to_string()),
            None,
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
            "unknown",
            None,
            Some("Failed to fetch page".to_string()),
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
                if i == 2 { "in_stock" } else { "out_of_stock" },
                None,
                None,
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
            AvailabilityCheckRepository::create(&conn, id, product_id, "in_stock", None, None)
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
            AvailabilityCheckRepository::create(&conn, id, product_id, "in_stock", None, None)
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
            AvailabilityCheckRepository::create(&conn, id, product_id, status, None, None)
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
            let check =
                AvailabilityCheckRepository::create(&conn, id, product_id, status, None, None)
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
                "in_stock",
                None,
                None,
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
                "out_of_stock",
                None,
                None,
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
            "in_stock",
            None,
            None,
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
                "in_stock",
                Some(raw.to_string()),
                None,
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
                "in_stock",
                None,
                None,
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
}
