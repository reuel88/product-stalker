use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;
use crate::entities::prelude::ProductModel;
use crate::error::AppError;
use crate::services::ProductService;
use crate::utils::parse_uuid;

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
    let uuid = parse_uuid(&id)?;

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
    let uuid = parse_uuid(&id)?;

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
    let uuid = parse_uuid(&id)?;

    ProductService::delete(db.conn(), uuid).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

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

    #[test]
    fn test_product_response_from_model_with_all_fields() {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let model = ProductModel {
            id,
            name: "Full Product".to_string(),
            url: "https://full.example.com".to_string(),
            description: Some("Full description".to_string()),
            notes: Some("Some notes".to_string()),
            created_at: now,
            updated_at: now,
        };

        let response = ProductResponse::from(model);

        assert_eq!(response.id, id.to_string());
        assert_eq!(response.name, "Full Product");
        assert_eq!(response.url, "https://full.example.com");
        assert_eq!(response.description, Some("Full description".to_string()));
        assert_eq!(response.notes, Some("Some notes".to_string()));
        assert!(!response.created_at.is_empty());
        assert!(!response.updated_at.is_empty());
    }

    #[test]
    fn test_product_response_from_model_with_no_optional_fields() {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let model = ProductModel {
            id,
            name: "Minimal Product".to_string(),
            url: "https://minimal.example.com".to_string(),
            description: None,
            notes: None,
            created_at: now,
            updated_at: now,
        };

        let response = ProductResponse::from(model);

        assert_eq!(response.name, "Minimal Product");
        assert!(response.description.is_none());
        assert!(response.notes.is_none());
    }

    #[test]
    fn test_product_response_serializes_to_json() {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let model = ProductModel {
            id,
            name: "JSON Test".to_string(),
            url: "https://json.test".to_string(),
            description: Some("Test desc".to_string()),
            notes: None,
            created_at: now,
            updated_at: now,
        };

        let response = ProductResponse::from(model);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("JSON Test"));
        assert!(json.contains("https://json.test"));
        assert!(json.contains(&id.to_string()));
    }

    #[test]
    fn test_create_product_input_deserializes() {
        let json =
            r#"{"name":"Test","url":"https://test.com","description":"desc","notes":"note"}"#;
        let input: CreateProductInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.name, "Test");
        assert_eq!(input.url, "https://test.com");
        assert_eq!(input.description, Some("desc".to_string()));
        assert_eq!(input.notes, Some("note".to_string()));
    }

    #[test]
    fn test_create_product_input_deserializes_minimal() {
        let json = r#"{"name":"Test","url":"https://test.com"}"#;
        let input: CreateProductInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.name, "Test");
        assert_eq!(input.url, "https://test.com");
        assert!(input.description.is_none());
        assert!(input.notes.is_none());
    }

    #[test]
    fn test_update_product_input_deserializes_partial() {
        let json = r#"{"name":"Updated Name"}"#;
        let input: UpdateProductInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.name, Some("Updated Name".to_string()));
        assert!(input.url.is_none());
        assert!(input.description.is_none());
        assert!(input.notes.is_none());
    }

    #[test]
    fn test_update_product_input_deserializes_all_fields() {
        let json =
            r#"{"name":"Name","url":"https://url.com","description":"desc","notes":"notes"}"#;
        let input: UpdateProductInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.name, Some("Name".to_string()));
        assert_eq!(input.url, Some("https://url.com".to_string()));
        assert_eq!(input.description, Some("desc".to_string()));
        assert_eq!(input.notes, Some("notes".to_string()));
    }

    #[test]
    fn test_update_product_input_deserializes_empty() {
        let json = r#"{}"#;
        let input: UpdateProductInput = serde_json::from_str(json).unwrap();

        assert!(input.name.is_none());
        assert!(input.url.is_none());
        assert!(input.description.is_none());
        assert!(input.notes.is_none());
    }

    #[test]
    fn test_product_response_timestamps_are_rfc3339() {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let model = ProductModel {
            id,
            name: "Timestamp Test".to_string(),
            url: "https://time.test".to_string(),
            description: None,
            notes: None,
            created_at: now,
            updated_at: now,
        };

        let response = ProductResponse::from(model);

        // RFC3339 format includes 'T' separator and timezone
        assert!(response.created_at.contains('T'));
        assert!(response.updated_at.contains('T'));
    }
}
