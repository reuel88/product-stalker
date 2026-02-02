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
    use crate::entities::availability_check::Entity as AvailabilityCheckEntity;
    use crate::entities::product::Entity as ProductEntity;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Schema};

    async fn setup_test_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(DatabaseBackend::Sqlite);

        // Create products table first (for foreign key)
        let stmt = schema.create_table_from_entity(ProductEntity);
        conn.execute(conn.get_database_backend().build(&stmt))
            .await
            .unwrap();

        // Create availability_checks table
        let stmt = schema.create_table_from_entity(AvailabilityCheckEntity);
        conn.execute(conn.get_database_backend().build(&stmt))
            .await
            .unwrap();

        conn
    }

    async fn create_test_product(conn: &DatabaseConnection) -> Uuid {
        use crate::repositories::ProductRepository;
        let id = Uuid::new_v4();
        ProductRepository::create(
            conn,
            id,
            "Test Product".to_string(),
            "https://example.com/product".to_string(),
            None,
            None,
        )
        .await
        .unwrap();
        id
    }

    #[tokio::test]
    async fn test_create_availability_check() {
        let conn = setup_test_db().await;
        let product_id = create_test_product(&conn).await;
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
        let conn = setup_test_db().await;
        let product_id = create_test_product(&conn).await;
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
        let conn = setup_test_db().await;
        let product_id = create_test_product(&conn).await;

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
        let conn = setup_test_db().await;
        let product_id = create_test_product(&conn).await;

        let latest = AvailabilityCheckRepository::find_latest_for_product(&conn, product_id)
            .await
            .unwrap();

        assert!(latest.is_none());
    }

    #[tokio::test]
    async fn test_find_all_for_product() {
        let conn = setup_test_db().await;
        let product_id = create_test_product(&conn).await;

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
        let conn = setup_test_db().await;
        let product_id = create_test_product(&conn).await;

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

}
