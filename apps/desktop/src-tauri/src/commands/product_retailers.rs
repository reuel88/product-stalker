use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::DbState;
use crate::domain::entities::prelude::ProductRetailerModel;
use crate::domain::services::{AddRetailerParams, ProductRetailerService};
use crate::tauri_error::CommandError;
use crate::utils::parse_uuid;

/// Input for adding a retailer to a product
#[derive(Debug, Deserialize)]
pub struct AddRetailerInput {
    pub product_id: String,
    pub url: String,
    pub label: Option<String>,
}

/// Response DTO for product-retailer links
#[derive(Debug, Serialize)]
pub struct ProductRetailerResponse {
    pub id: String,
    pub product_id: String,
    pub retailer_id: String,
    pub url: String,
    pub label: Option<String>,
    pub created_at: String,
}

impl From<ProductRetailerModel> for ProductRetailerResponse {
    fn from(model: ProductRetailerModel) -> Self {
        Self {
            id: model.id.to_string(),
            product_id: model.product_id.to_string(),
            retailer_id: model.retailer_id.to_string(),
            url: model.url,
            label: model.label,
            created_at: model.created_at.to_rfc3339(),
        }
    }
}

/// Add a retailer URL to a product
#[tauri::command]
pub async fn add_product_retailer(
    input: AddRetailerInput,
    db: State<'_, DbState>,
) -> Result<ProductRetailerResponse, CommandError> {
    let product_id = parse_uuid(&input.product_id)?;

    let product_retailer = ProductRetailerService::add_retailer(
        db.conn(),
        AddRetailerParams {
            product_id,
            url: input.url,
            label: input.label,
        },
    )
    .await?;

    Ok(ProductRetailerResponse::from(product_retailer))
}

/// Get all retailer links for a product
#[tauri::command]
pub async fn get_product_retailers(
    product_id: String,
    db: State<'_, DbState>,
) -> Result<Vec<ProductRetailerResponse>, CommandError> {
    let uuid = parse_uuid(&product_id)?;

    let retailers = ProductRetailerService::get_retailers_for_product(db.conn(), uuid).await?;
    Ok(retailers
        .into_iter()
        .map(ProductRetailerResponse::from)
        .collect())
}

/// Remove a retailer link from a product
#[tauri::command]
pub async fn remove_product_retailer(
    id: String,
    db: State<'_, DbState>,
) -> Result<(), CommandError> {
    let uuid = parse_uuid(&id)?;

    ProductRetailerService::remove_retailer(db.conn(), uuid).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_product_retailer_response_from_model() {
        let id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let retailer_id = Uuid::new_v4();
        let now = Utc::now();

        let model = ProductRetailerModel {
            id,
            product_id,
            retailer_id,
            url: "https://amazon.com/dp/B123".to_string(),
            label: Some("64GB version".to_string()),
            created_at: now,
        };

        let response = ProductRetailerResponse::from(model);

        assert_eq!(response.id, id.to_string());
        assert_eq!(response.product_id, product_id.to_string());
        assert_eq!(response.retailer_id, retailer_id.to_string());
        assert_eq!(response.url, "https://amazon.com/dp/B123");
        assert_eq!(response.label, Some("64GB version".to_string()));
        assert!(!response.created_at.is_empty());
    }

    #[test]
    fn test_product_retailer_response_from_model_no_label() {
        let model = ProductRetailerModel {
            id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            retailer_id: Uuid::new_v4(),
            url: "https://walmart.com/item/456".to_string(),
            label: None,
            created_at: Utc::now(),
        };

        let response = ProductRetailerResponse::from(model);

        assert!(response.label.is_none());
        assert_eq!(response.url, "https://walmart.com/item/456");
    }

    #[test]
    fn test_product_retailer_response_serializes_to_json() {
        let id = Uuid::new_v4();
        let model = ProductRetailerModel {
            id,
            product_id: Uuid::new_v4(),
            retailer_id: Uuid::new_v4(),
            url: "https://bestbuy.com/product/789".to_string(),
            label: Some("Blue".to_string()),
            created_at: Utc::now(),
        };

        let response = ProductRetailerResponse::from(model);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("bestbuy.com"));
        assert!(json.contains("Blue"));
        assert!(json.contains(&id.to_string()));
    }

    #[test]
    fn test_add_retailer_input_deserializes() {
        let json = r#"{"product_id":"550e8400-e29b-41d4-a716-446655440000","url":"https://amazon.com/dp/B123","label":"64GB"}"#;
        let input: AddRetailerInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.product_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(input.url, "https://amazon.com/dp/B123");
        assert_eq!(input.label, Some("64GB".to_string()));
    }

    #[test]
    fn test_add_retailer_input_deserializes_without_label() {
        let json = r#"{"product_id":"550e8400-e29b-41d4-a716-446655440000","url":"https://amazon.com/dp/B123"}"#;
        let input: AddRetailerInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.url, "https://amazon.com/dp/B123");
        assert!(input.label.is_none());
    }
}
