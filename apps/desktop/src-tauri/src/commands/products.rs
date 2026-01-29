use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;
use crate::entities::prelude::ProductModel;
use crate::error::AppError;
use crate::services::ProductService;

/// Input for creating a product
#[derive(Debug, Deserialize)]
pub struct CreateProductInput {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub notes: Option<String>,
}

/// Input for updating a product
#[derive(Debug, Deserialize)]
pub struct UpdateProductInput {
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

/// Response DTO for products
#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ProductModel> for ProductResponse {
    fn from(model: ProductModel) -> Self {
        Self {
            id: model.id.to_string(),
            name: model.name,
            url: model.url,
            description: model.description,
            notes: model.notes,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

/// Get all products
#[tauri::command]
pub async fn get_products(db: State<'_, DbState>) -> Result<Vec<ProductResponse>, AppError> {
    let products = ProductService::get_all(db.conn()).await?;
    Ok(products.into_iter().map(ProductResponse::from).collect())
}

/// Get a single product by ID
#[tauri::command]
pub async fn get_product(id: String, db: State<'_, DbState>) -> Result<ProductResponse, AppError> {
    let uuid =
        Uuid::parse_str(&id).map_err(|_| AppError::Validation(format!("Invalid UUID: {}", id)))?;

    let product = ProductService::get_by_id(db.conn(), uuid).await?;
    Ok(ProductResponse::from(product))
}

/// Create a new product
#[tauri::command]
pub async fn create_product(
    input: CreateProductInput,
    db: State<'_, DbState>,
) -> Result<ProductResponse, AppError> {
    let product = ProductService::create(
        db.conn(),
        input.name,
        input.url,
        input.description,
        input.notes,
    )
    .await?;

    Ok(ProductResponse::from(product))
}

/// Update an existing product
#[tauri::command]
pub async fn update_product(
    id: String,
    input: UpdateProductInput,
    db: State<'_, DbState>,
) -> Result<ProductResponse, AppError> {
    let uuid =
        Uuid::parse_str(&id).map_err(|_| AppError::Validation(format!("Invalid UUID: {}", id)))?;

    let product = ProductService::update(
        db.conn(),
        uuid,
        input.name,
        input.url,
        input.description,
        input.notes,
    )
    .await?;

    Ok(ProductResponse::from(product))
}

/// Delete a product
#[tauri::command]
pub async fn delete_product(id: String, db: State<'_, DbState>) -> Result<(), AppError> {
    let uuid =
        Uuid::parse_str(&id).map_err(|_| AppError::Validation(format!("Invalid UUID: {}", id)))?;

    ProductService::delete(db.conn(), uuid).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_product_response_from_model() {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let model = ProductModel {
            id,
            name: "Test Product".to_string(),
            url: "https://example.com".to_string(),
            description: Some("A description".to_string()),
            notes: None,
            created_at: now,
            updated_at: now,
        };

        let response = ProductResponse::from(model);

        assert_eq!(response.id, id.to_string());
        assert_eq!(response.name, "Test Product");
        assert_eq!(response.url, "https://example.com");
        assert_eq!(response.description, Some("A description".to_string()));
        assert!(response.notes.is_none());
    }
}
