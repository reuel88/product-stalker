use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::entities::prelude::ProductModel;
use crate::error::AppError;
use crate::repositories::ProductRepository;

/// Service layer for product business logic
///
/// This layer validates input and orchestrates repository calls.
/// It keeps business rules separate from data access and presentation.
pub struct ProductService;

impl ProductService {
    /// Get all products
    pub async fn get_all(conn: &DatabaseConnection) -> Result<Vec<ProductModel>, AppError> {
        ProductRepository::find_all(conn).await
    }

    /// Get a product by ID
    pub async fn get_by_id(conn: &DatabaseConnection, id: Uuid) -> Result<ProductModel, AppError> {
        ProductRepository::find_by_id(conn, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Product not found: {}", id)))
    }

    /// Create a new product
    pub async fn create(
        conn: &DatabaseConnection,
        name: String,
        url: String,
        description: Option<String>,
        notes: Option<String>,
    ) -> Result<ProductModel, AppError> {
        // Validate inputs
        Self::validate_name(&name)?;
        Self::validate_url(&url)?;

        let id = Uuid::new_v4();
        ProductRepository::create(conn, id, name, url, description, notes).await
    }

    /// Update an existing product
    pub async fn update(
        conn: &DatabaseConnection,
        id: Uuid,
        name: Option<String>,
        url: Option<String>,
        description: Option<String>,
        notes: Option<String>,
    ) -> Result<ProductModel, AppError> {
        // Validate inputs if provided
        if let Some(ref name) = name {
            Self::validate_name(name)?;
        }
        if let Some(ref url) = url {
            Self::validate_url(url)?;
        }

        // Fetch existing product
        let product = Self::get_by_id(conn, id).await?;

        // Update product
        ProductRepository::update(
            conn,
            product,
            name,
            url,
            description.map(Some),
            notes.map(Some),
        )
        .await
    }

    /// Delete a product
    pub async fn delete(conn: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
        let rows_affected = ProductRepository::delete_by_id(conn, id).await?;

        if rows_affected == 0 {
            return Err(AppError::NotFound(format!("Product not found: {}", id)));
        }

        Ok(())
    }

    // Private validation helpers

    fn validate_name(name: &str) -> Result<(), AppError> {
        if name.trim().is_empty() {
            return Err(AppError::Validation("Name cannot be empty".to_string()));
        }
        Ok(())
    }

    fn validate_url(url: &str) -> Result<(), AppError> {
        if url.trim().is_empty() {
            return Err(AppError::Validation("URL cannot be empty".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_empty() {
        assert!(ProductService::validate_name("").is_err());
    }

    #[test]
    fn test_validate_name_whitespace() {
        assert!(ProductService::validate_name("   ").is_err());
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(ProductService::validate_name("My Product").is_ok());
    }

    #[test]
    fn test_validate_url_empty() {
        assert!(ProductService::validate_url("").is_err());
    }

    #[test]
    fn test_validate_url_whitespace() {
        assert!(ProductService::validate_url("   ").is_err());
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(ProductService::validate_url("https://example.com").is_ok());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::test_utils::setup_products_db;

    #[tokio::test]
    async fn test_create_product_validates_name() {
        let conn = setup_products_db().await;
        let result = ProductService::create(
            &conn,
            "".to_string(),
            "https://test.com".to_string(),
            None,
            None,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_product_validates_url() {
        let conn = setup_products_db().await;
        let result =
            ProductService::create(&conn, "Valid Name".to_string(), "".to_string(), None, None)
                .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_product_success() {
        let conn = setup_products_db().await;
        let result = ProductService::create(
            &conn,
            "Test Product".to_string(),
            "https://test.com".to_string(),
            Some("A description".to_string()),
            Some("Some notes".to_string()),
        )
        .await;

        assert!(result.is_ok());
        let product = result.unwrap();
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.url, "https://test.com");
        assert_eq!(product.description, Some("A description".to_string()));
        assert_eq!(product.notes, Some("Some notes".to_string()));
    }

    #[tokio::test]
    async fn test_create_product_minimal() {
        let conn = setup_products_db().await;
        let result = ProductService::create(
            &conn,
            "Minimal Product".to_string(),
            "https://minimal.com".to_string(),
            None,
            None,
        )
        .await;

        assert!(result.is_ok());
        let product = result.unwrap();
        assert_eq!(product.name, "Minimal Product");
        assert!(product.description.is_none());
        assert!(product.notes.is_none());
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let conn = setup_products_db().await;
        let result = ProductService::get_by_id(&conn, Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_get_by_id_success() {
        let conn = setup_products_db().await;
        let created = ProductService::create(
            &conn,
            "Find Me".to_string(),
            "https://findme.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

        let found = ProductService::get_by_id(&conn, created.id).await;
        assert!(found.is_ok());
        assert_eq!(found.unwrap().name, "Find Me");
    }

    #[tokio::test]
    async fn test_get_all_empty() {
        let conn = setup_products_db().await;
        let result = ProductService::get_all(&conn).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_all_multiple() {
        let conn = setup_products_db().await;

        ProductService::create(
            &conn,
            "Product 1".to_string(),
            "https://p1.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();
        ProductService::create(
            &conn,
            "Product 2".to_string(),
            "https://p2.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();
        ProductService::create(
            &conn,
            "Product 3".to_string(),
            "https://p3.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

        let result = ProductService::get_all(&conn).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_update_product_name() {
        let conn = setup_products_db().await;
        let created = ProductService::create(
            &conn,
            "Original".to_string(),
            "https://original.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

        let updated = ProductService::update(
            &conn,
            created.id,
            Some("Updated Name".to_string()),
            None,
            None,
            None,
        )
        .await;

        assert!(updated.is_ok());
        let product = updated.unwrap();
        assert_eq!(product.name, "Updated Name");
        assert_eq!(product.url, "https://original.com");
    }

    #[tokio::test]
    async fn test_update_product_url() {
        let conn = setup_products_db().await;
        let created = ProductService::create(
            &conn,
            "Test".to_string(),
            "https://old.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

        let updated = ProductService::update(
            &conn,
            created.id,
            None,
            Some("https://new.com".to_string()),
            None,
            None,
        )
        .await;

        assert!(updated.is_ok());
        let product = updated.unwrap();
        assert_eq!(product.name, "Test");
        assert_eq!(product.url, "https://new.com");
    }

    #[tokio::test]
    async fn test_update_product_description() {
        let conn = setup_products_db().await;
        let created = ProductService::create(
            &conn,
            "Test".to_string(),
            "https://test.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

        let updated = ProductService::update(
            &conn,
            created.id,
            None,
            None,
            Some("New description".to_string()),
            None,
        )
        .await;

        assert!(updated.is_ok());
        assert_eq!(
            updated.unwrap().description,
            Some("New description".to_string())
        );
    }

    #[tokio::test]
    async fn test_update_product_not_found() {
        let conn = setup_products_db().await;
        let result = ProductService::update(
            &conn,
            Uuid::new_v4(),
            Some("Name".to_string()),
            None,
            None,
            None,
        )
        .await;

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_update_validates_empty_name() {
        let conn = setup_products_db().await;
        let created = ProductService::create(
            &conn,
            "Test".to_string(),
            "https://test.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

        let result =
            ProductService::update(&conn, created.id, Some("".to_string()), None, None, None).await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn test_update_validates_empty_url() {
        let conn = setup_products_db().await;
        let created = ProductService::create(
            &conn,
            "Test".to_string(),
            "https://test.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

        let result =
            ProductService::update(&conn, created.id, None, Some("".to_string()), None, None).await;

        assert!(matches!(result, Err(AppError::Validation(_))));
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let conn = setup_products_db().await;
        let result = ProductService::delete(&conn, Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_success() {
        let conn = setup_products_db().await;
        let created = ProductService::create(
            &conn,
            "To Delete".to_string(),
            "https://delete.com".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

        let result = ProductService::delete(&conn, created.id).await;
        assert!(result.is_ok());

        // Verify it's actually deleted
        let find_result = ProductService::get_by_id(&conn, created.id).await;
        assert!(matches!(find_result, Err(AppError::NotFound(_))));
    }
}
