//! Product service for business logic around products.

use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::entities::prelude::ProductModel;
use crate::repositories::{CreateProductRepoParams, ProductRepository, ProductUpdateInput};
use product_stalker_core::AppError;

/// Parameters for creating a new product
pub struct CreateProductParams {
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
}

/// Parameters for updating an existing product (all fields optional for partial updates)
pub struct UpdateProductParams {
    pub name: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

/// Parameters for reordering products
pub struct ReorderProductsParams {
    pub updates: Vec<(Uuid, i32)>,
}

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
        params: CreateProductParams,
    ) -> Result<ProductModel, AppError> {
        Self::validate_name(&params.name)?;

        let id = Uuid::new_v4();
        ProductRepository::create(
            conn,
            id,
            CreateProductRepoParams {
                name: params.name,
                url: None,
                description: params.description,
                notes: params.notes,
            },
        )
        .await
    }

    /// Update an existing product
    pub async fn update(
        conn: &DatabaseConnection,
        id: Uuid,
        params: UpdateProductParams,
    ) -> Result<ProductModel, AppError> {
        // Validate inputs if provided
        if let Some(ref name) = params.name {
            Self::validate_name(name)?;
        }

        // Fetch existing product
        let product = Self::get_by_id(conn, id).await?;

        // Update product
        ProductRepository::update(
            conn,
            product,
            ProductUpdateInput {
                name: params.name,
                url: None,
                description: params.description.map(Some),
                notes: params.notes.map(Some),
                currency: None,
            },
        )
        .await
    }

    /// Get all products that have no associated product_retailers (legacy products)
    pub async fn get_all_without_retailers(
        conn: &DatabaseConnection,
    ) -> Result<Vec<ProductModel>, AppError> {
        ProductRepository::find_all_without_retailers(conn).await
    }

    /// Reorder products by updating their sort_order values
    pub async fn reorder(
        conn: &DatabaseConnection,
        params: ReorderProductsParams,
    ) -> Result<(), AppError> {
        for &(_, sort_order) in &params.updates {
            if sort_order < 0 {
                return Err(AppError::Validation(
                    "sort_order must be non-negative".to_string(),
                ));
            }
        }

        ProductRepository::update_sort_orders(conn, params.updates).await
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
    fn test_reorder_validates_negative_sort_order() {
        let params = ReorderProductsParams {
            updates: vec![(Uuid::new_v4(), -1)],
        };
        // Validation fails before any DB call, so Disconnected is fine
        let conn = DatabaseConnection::Disconnected;
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { ProductService::reorder(&conn, params).await });
        assert!(matches!(result, Err(AppError::Validation(_))));
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
            CreateProductParams {
                name: "".to_string(),
                description: None,
                notes: None,
            },
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_product_success() {
        let conn = setup_products_db().await;
        let result = ProductService::create(
            &conn,
            CreateProductParams {
                name: "Test Product".to_string(),
                description: Some("A description".to_string()),
                notes: Some("Some notes".to_string()),
            },
        )
        .await;

        assert!(result.is_ok());
        let product = result.unwrap();
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.description, Some("A description".to_string()));
        assert_eq!(product.notes, Some("Some notes".to_string()));
    }

    #[tokio::test]
    async fn test_create_product_minimal() {
        let conn = setup_products_db().await;
        let result = ProductService::create(
            &conn,
            CreateProductParams {
                name: "Minimal Product".to_string(),
                description: None,
                notes: None,
            },
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

    /// Helper to create a product with minimal params in tests
    fn params(name: &str) -> CreateProductParams {
        CreateProductParams {
            name: name.to_string(),
            description: None,
            notes: None,
        }
    }

    #[tokio::test]
    async fn test_get_by_id_success() {
        let conn = setup_products_db().await;
        let created = ProductService::create(&conn, params("Find Me"))
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

        ProductService::create(&conn, params("Product 1"))
            .await
            .unwrap();
        ProductService::create(&conn, params("Product 2"))
            .await
            .unwrap();
        ProductService::create(&conn, params("Product 3"))
            .await
            .unwrap();

        let result = ProductService::get_all(&conn).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_update_product_name() {
        let conn = setup_products_db().await;
        let created = ProductService::create(&conn, params("Original"))
            .await
            .unwrap();

        let updated = ProductService::update(
            &conn,
            created.id,
            UpdateProductParams {
                name: Some("Updated Name".to_string()),
                description: None,
                notes: None,
            },
        )
        .await;

        assert!(updated.is_ok());
        let product = updated.unwrap();
        assert_eq!(product.name, "Updated Name");
    }

    #[tokio::test]
    async fn test_update_product_description() {
        let conn = setup_products_db().await;
        let created = ProductService::create(&conn, params("Test")).await.unwrap();

        let updated = ProductService::update(
            &conn,
            created.id,
            UpdateProductParams {
                name: None,
                description: Some("New description".to_string()),
                notes: None,
            },
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
            UpdateProductParams {
                name: Some("Name".to_string()),
                description: None,
                notes: None,
            },
        )
        .await;

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_update_validates_empty_name() {
        let conn = setup_products_db().await;
        let created = ProductService::create(&conn, params("Test")).await.unwrap();

        let result = ProductService::update(
            &conn,
            created.id,
            UpdateProductParams {
                name: Some("".to_string()),
                description: None,
                notes: None,
            },
        )
        .await;

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
        let created = ProductService::create(&conn, params("To Delete"))
            .await
            .unwrap();

        let result = ProductService::delete(&conn, created.id).await;
        assert!(result.is_ok());

        // Verify it's actually deleted
        let find_result = ProductService::get_by_id(&conn, created.id).await;
        assert!(matches!(find_result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_reorder_products() {
        let conn = setup_products_db().await;

        let p1 = ProductService::create(&conn, params("Alpha"))
            .await
            .unwrap();
        let p2 = ProductService::create(&conn, params("Beta")).await.unwrap();
        let p3 = ProductService::create(&conn, params("Gamma"))
            .await
            .unwrap();

        // Reverse order
        ProductService::reorder(
            &conn,
            ReorderProductsParams {
                updates: vec![(p3.id, 0), (p2.id, 1), (p1.id, 2)],
            },
        )
        .await
        .unwrap();

        let products = ProductService::get_all(&conn).await.unwrap();
        assert_eq!(products[0].name, "Gamma");
        assert_eq!(products[1].name, "Beta");
        assert_eq!(products[2].name, "Alpha");
    }
}
