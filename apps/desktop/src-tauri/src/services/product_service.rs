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
