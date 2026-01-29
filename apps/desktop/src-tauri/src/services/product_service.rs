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
    use crate::entities::product::Entity as Product;
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Schema};

    async fn setup_test_db() -> DatabaseConnection {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(DatabaseBackend::Sqlite);
        let stmt = schema.create_table_from_entity(Product);
        conn.execute(conn.get_database_backend().build(&stmt))
            .await
            .unwrap();
        conn
    }

    #[tokio::test]
    async fn test_create_product_validates_name() {
        let conn = setup_test_db().await;
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
    async fn test_get_by_id_not_found() {
        let conn = setup_test_db().await;
        let result = ProductService::get_by_id(&conn, Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let conn = setup_test_db().await;
        let result = ProductService::delete(&conn, Uuid::new_v4()).await;
        assert!(matches!(result, Err(AppError::NotFound(_))));
    }
}
