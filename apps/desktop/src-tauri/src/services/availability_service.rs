use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::entities::prelude::AvailabilityCheckModel;
use crate::error::AppError;
use crate::repositories::{AvailabilityCheckRepository, ProductRepository};
use crate::services::ScraperService;

/// Service layer for availability checking business logic
pub struct AvailabilityService;

impl AvailabilityService {
    /// Check the availability of a product by its ID
    ///
    /// Fetches the product's URL, scrapes the page for availability info,
    /// and stores the result in the database.
    pub async fn check_product(
        conn: &DatabaseConnection,
        product_id: Uuid,
    ) -> Result<AvailabilityCheckModel, AppError> {
        // Get the product to get its URL
        let product = ProductRepository::find_by_id(conn, product_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", product_id)))?;

        // Attempt to check availability
        let check_id = Uuid::new_v4();
        let result = ScraperService::check_availability(&product.url).await;

        match result {
            Ok(scraping_result) => {
                // Success - store the result
                AvailabilityCheckRepository::create(
                    conn,
                    check_id,
                    product_id,
                    scraping_result.status.as_str(),
                    scraping_result.raw_availability,
                    None,
                )
                .await
            }
            Err(e) => {
                // Error - store the error but still create a record
                AvailabilityCheckRepository::create(
                    conn,
                    check_id,
                    product_id,
                    "unknown",
                    None,
                    Some(e.to_string()),
                )
                .await
            }
        }
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

        let stmt = schema.create_table_from_entity(ProductEntity);
        conn.execute(conn.get_database_backend().build(&stmt))
            .await
            .unwrap();

        let stmt = schema.create_table_from_entity(AvailabilityCheckEntity);
        conn.execute(conn.get_database_backend().build(&stmt))
            .await
            .unwrap();

        conn
    }

    async fn create_test_product(conn: &DatabaseConnection, url: &str) -> Uuid {
        let id = Uuid::new_v4();
        ProductRepository::create(
            conn,
            id,
            "Test Product".to_string(),
            url.to_string(),
            None,
            None,
        )
        .await
        .unwrap();
        id
    }

    #[tokio::test]
    async fn test_get_latest_none() {
        let conn = setup_test_db().await;
        let product_id = create_test_product(&conn, "https://example.com").await;

        let latest = AvailabilityService::get_latest(&conn, product_id)
            .await
            .unwrap();

        assert!(latest.is_none());
    }

    #[tokio::test]
    async fn test_get_history_empty() {
        let conn = setup_test_db().await;
        let product_id = create_test_product(&conn, "https://example.com").await;

        let history = AvailabilityService::get_history(&conn, product_id, None)
            .await
            .unwrap();

        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_get_history_with_limit() {
        let conn = setup_test_db().await;
        let product_id = create_test_product(&conn, "https://example.com").await;

        // Create some check records directly
        for _ in 0..5 {
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

        let history = AvailabilityService::get_history(&conn, product_id, Some(3))
            .await
            .unwrap();

        assert_eq!(history.len(), 3);
    }

    #[tokio::test]
    async fn test_check_product_not_found() {
        let conn = setup_test_db().await;
        let fake_id = Uuid::new_v4();

        let result = AvailabilityService::check_product(&conn, fake_id).await;

        assert!(result.is_err());
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
