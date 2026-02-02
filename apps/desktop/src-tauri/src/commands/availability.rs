use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::db::DbState;
use crate::entities::prelude::AvailabilityCheckModel;
use crate::error::AppError;
use crate::services::AvailabilityService;

/// Response DTO for availability checks
#[derive(Debug, Serialize)]
pub struct AvailabilityCheckResponse {
    pub id: String,
    pub product_id: String,
    pub status: String,
    pub raw_availability: Option<String>,
    pub error_message: Option<String>,
    pub checked_at: String,
}

impl From<AvailabilityCheckModel> for AvailabilityCheckResponse {
    fn from(model: AvailabilityCheckModel) -> Self {
        Self {
            id: model.id.to_string(),
            product_id: model.product_id.to_string(),
            status: model.status,
            raw_availability: model.raw_availability,
            error_message: model.error_message,
            checked_at: model.checked_at.to_rfc3339(),
        }
    }
}

/// Check availability for a product
///
/// Fetches the product's URL and parses Schema.org data to determine availability.
#[tauri::command]
pub async fn check_availability(
    product_id: String,
    db: State<'_, DbState>,
) -> Result<AvailabilityCheckResponse, AppError> {
    let uuid = Uuid::parse_str(&product_id)
        .map_err(|_| AppError::Validation(format!("Invalid UUID: {}", product_id)))?;

    let check = AvailabilityService::check_product(db.conn(), uuid).await?;
    Ok(AvailabilityCheckResponse::from(check))
}

/// Get the latest availability check for a product
#[tauri::command]
pub async fn get_latest_availability(
    product_id: String,
    db: State<'_, DbState>,
) -> Result<Option<AvailabilityCheckResponse>, AppError> {
    let uuid = Uuid::parse_str(&product_id)
        .map_err(|_| AppError::Validation(format!("Invalid UUID: {}", product_id)))?;

    let check = AvailabilityService::get_latest(db.conn(), uuid).await?;
    Ok(check.map(AvailabilityCheckResponse::from))
}

/// Get availability check history for a product
#[tauri::command]
pub async fn get_availability_history(
    product_id: String,
    limit: Option<u64>,
    db: State<'_, DbState>,
) -> Result<Vec<AvailabilityCheckResponse>, AppError> {
    let uuid = Uuid::parse_str(&product_id)
        .map_err(|_| AppError::Validation(format!("Invalid UUID: {}", product_id)))?;

    let checks = AvailabilityService::get_history(db.conn(), uuid, limit).await?;
    Ok(checks
        .into_iter()
        .map(AvailabilityCheckResponse::from)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_availability_check_response_from_model() {
        let id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let now = Utc::now();

        let model = AvailabilityCheckModel {
            id,
            product_id,
            status: "in_stock".to_string(),
            raw_availability: Some("http://schema.org/InStock".to_string()),
            error_message: None,
            checked_at: now,
        };

        let response = AvailabilityCheckResponse::from(model);

        assert_eq!(response.id, id.to_string());
        assert_eq!(response.product_id, product_id.to_string());
        assert_eq!(response.status, "in_stock");
        assert_eq!(
            response.raw_availability,
            Some("http://schema.org/InStock".to_string())
        );
        assert!(response.error_message.is_none());
        assert!(!response.checked_at.is_empty());
    }

    #[test]
    fn test_availability_check_response_with_error() {
        let id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let now = Utc::now();

        let model = AvailabilityCheckModel {
            id,
            product_id,
            status: "unknown".to_string(),
            raw_availability: None,
            error_message: Some("Failed to fetch page".to_string()),
            checked_at: now,
        };

        let response = AvailabilityCheckResponse::from(model);

        assert_eq!(response.status, "unknown");
        assert!(response.raw_availability.is_none());
        assert_eq!(
            response.error_message,
            Some("Failed to fetch page".to_string())
        );
    }

    #[test]
    fn test_availability_check_response_serializes_to_json() {
        let id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let now = Utc::now();

        let model = AvailabilityCheckModel {
            id,
            product_id,
            status: "out_of_stock".to_string(),
            raw_availability: Some("http://schema.org/OutOfStock".to_string()),
            error_message: None,
            checked_at: now,
        };

        let response = AvailabilityCheckResponse::from(model);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("out_of_stock"));
        assert!(json.contains(&id.to_string()));
        assert!(json.contains(&product_id.to_string()));
    }
}
