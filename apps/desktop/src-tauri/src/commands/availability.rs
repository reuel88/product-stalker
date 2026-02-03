use serde::Serialize;
use tauri::State;
use tauri_plugin_notification::NotificationExt;

use crate::commands::should_send_notification;
use crate::db::DbState;
use crate::entities::prelude::AvailabilityCheckModel;
use crate::error::AppError;
use crate::repositories::ProductRepository;
use crate::services::{AvailabilityService, BulkCheckSummary};
use crate::utils::parse_uuid;

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
/// Sends a desktop notification if the product is back in stock.
#[tauri::command]
pub async fn check_availability(
    app: tauri::AppHandle,
    product_id: String,
    db: State<'_, DbState>,
) -> Result<AvailabilityCheckResponse, AppError> {
    let uuid = parse_uuid(&product_id)?;

    // Get previous status before checking
    let previous_check = AvailabilityService::get_latest(db.conn(), uuid).await?;
    let previous_status = previous_check.map(|c| c.status);

    // Perform the check
    let check = AvailabilityService::check_product(db.conn(), uuid).await?;

    // Check if product is back in stock and send notification
    if AvailabilityService::is_back_in_stock(&previous_status, &check.status)
        && should_send_notification(db.conn()).await?
    {
        // Get product name for the notification
        if let Ok(Some(product)) = ProductRepository::find_by_id(db.conn(), uuid).await {
            if let Err(e) = app
                .notification()
                .builder()
                .title("Product Back in Stock!")
                .body(format!("{} is now available!", product.name))
                .show()
            {
                log::warn!("Failed to send back-in-stock notification: {}", e);
            } else {
                log::info!("Sent back-in-stock notification for: {}", product.name);
            }
        }
    }

    Ok(AvailabilityCheckResponse::from(check))
}

/// Get the latest availability check for a product
#[tauri::command]
pub async fn get_latest_availability(
    product_id: String,
    db: State<'_, DbState>,
) -> Result<Option<AvailabilityCheckResponse>, AppError> {
    let uuid = parse_uuid(&product_id)?;

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
    let uuid = parse_uuid(&product_id)?;

    let checks = AvailabilityService::get_history(db.conn(), uuid, limit).await?;
    Ok(checks
        .into_iter()
        .map(AvailabilityCheckResponse::from)
        .collect())
}

/// Check availability for all products
///
/// Performs a bulk availability check on all products with rate limiting.
/// Sends desktop notifications for products that are back in stock.
#[tauri::command]
pub async fn check_all_availability(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
) -> Result<BulkCheckSummary, AppError> {
    let summary = AvailabilityService::check_all_products(db.conn()).await?;

    // Send notifications for products that are back in stock
    if summary.back_in_stock_count > 0 && should_send_notification(db.conn()).await? {
        let back_in_stock_products: Vec<&str> = summary
            .results
            .iter()
            .filter(|r| r.is_back_in_stock)
            .map(|r| r.product_name.as_str())
            .collect();

        let notification_body = if back_in_stock_products.len() == 1 {
            format!("{} is back in stock!", back_in_stock_products[0])
        } else {
            format!(
                "{} products are back in stock: {}",
                back_in_stock_products.len(),
                back_in_stock_products.join(", ")
            )
        };

        if let Err(e) = app
            .notification()
            .builder()
            .title("Products Back in Stock!")
            .body(&notification_body)
            .show()
        {
            log::warn!("Failed to send back-in-stock notification: {}", e);
        } else {
            log::info!(
                "Sent back-in-stock notification for {} product(s)",
                back_in_stock_products.len()
            );
        }
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

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
