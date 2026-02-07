use serde::Serialize;
use tauri::State;
use tauri_plugin_notification::NotificationExt;

use crate::db::DbState;
use crate::entities::prelude::AvailabilityCheckModel;
use crate::error::AppError;
use crate::repositories::AvailabilityCheckRepository;
use crate::services::{AvailabilityService, BulkCheckSummary, NotificationData};
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
    pub price_cents: Option<i64>,
    pub price_currency: Option<String>,
    pub raw_price: Option<String>,
    pub previous_price_cents: Option<i64>,
    pub is_price_drop: bool,
}

impl AvailabilityCheckResponse {
    /// Create a response from a model with previous price information
    pub fn from_model_with_previous_price(
        model: AvailabilityCheckModel,
        previous_price_cents: Option<i64>,
    ) -> Self {
        let is_price_drop =
            AvailabilityService::is_price_drop(previous_price_cents, model.price_cents);
        Self {
            id: model.id.to_string(),
            product_id: model.product_id.to_string(),
            status: model.status,
            raw_availability: model.raw_availability,
            error_message: model.error_message,
            checked_at: model.checked_at.to_rfc3339(),
            price_cents: model.price_cents,
            price_currency: model.price_currency,
            raw_price: model.raw_price,
            previous_price_cents,
            is_price_drop,
        }
    }
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
            price_cents: model.price_cents,
            price_currency: model.price_currency,
            raw_price: model.raw_price,
            previous_price_cents: None,
            is_price_drop: false,
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

    // Fetch previous price before the check for price drop comparison
    let previous_price_cents =
        AvailabilityCheckRepository::find_previous_price(db.conn(), uuid).await?;

    let result = AvailabilityService::check_product_with_notification(db.conn(), uuid).await?;

    if let Some(notification) = result.notification {
        send_desktop_notification(&app, &notification);
    }

    Ok(AvailabilityCheckResponse::from_model_with_previous_price(
        result.check,
        previous_price_cents,
    ))
}

/// Get the latest availability check for a product
#[tauri::command]
pub async fn get_latest_availability(
    product_id: String,
    db: State<'_, DbState>,
) -> Result<Option<AvailabilityCheckResponse>, AppError> {
    let uuid = parse_uuid(&product_id)?;

    let check = AvailabilityService::get_latest(db.conn(), uuid).await?;

    match check {
        Some(model) => {
            // Get the second-most-recent price (the one before the latest check)
            let previous_price_cents =
                AvailabilityCheckRepository::find_second_previous_price(db.conn(), uuid).await?;
            Ok(Some(
                AvailabilityCheckResponse::from_model_with_previous_price(
                    model,
                    previous_price_cents,
                ),
            ))
        }
        None => Ok(None),
    }
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
/// Emits progress events for each product checked.
#[tauri::command]
pub async fn check_all_availability(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
) -> Result<BulkCheckSummary, AppError> {
    let result = AvailabilityService::check_all_products_with_notification(db.conn(), &app).await?;

    if let Some(notification) = result.notification {
        send_desktop_notification(&app, &notification);
    }

    Ok(result.summary)
}

/// Send a desktop notification (Tauri-specific, kept in command layer)
fn send_desktop_notification(app: &tauri::AppHandle, notification: &NotificationData) {
    if let Err(e) = app
        .notification()
        .builder()
        .title(&notification.title)
        .body(&notification.body)
        .show()
    {
        log::warn!("Failed to send notification: {}", e);
    } else {
        log::info!("Sent notification: {}", notification.title);
    }
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
            price_cents: Some(78900),
            price_currency: Some("USD".to_string()),
            raw_price: Some("789.00".to_string()),
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
        assert_eq!(response.price_cents, Some(78900));
        assert_eq!(response.price_currency, Some("USD".to_string()));
        assert_eq!(response.raw_price, Some("789.00".to_string()));
        assert!(response.previous_price_cents.is_none());
        assert!(!response.is_price_drop);
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
            price_cents: None,
            price_currency: None,
            raw_price: None,
        };

        let response = AvailabilityCheckResponse::from(model);

        assert_eq!(response.status, "unknown");
        assert!(response.raw_availability.is_none());
        assert_eq!(
            response.error_message,
            Some("Failed to fetch page".to_string())
        );
        assert!(response.price_cents.is_none());
        assert!(!response.is_price_drop);
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
            price_cents: Some(9999),
            price_currency: Some("EUR".to_string()),
            raw_price: Some("99.99".to_string()),
        };

        let response = AvailabilityCheckResponse::from(model);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("out_of_stock"));
        assert!(json.contains(&id.to_string()));
        assert!(json.contains(&product_id.to_string()));
        assert!(json.contains("9999"));
        assert!(json.contains("EUR"));
        assert!(json.contains("\"previous_price_cents\":null"));
        assert!(json.contains("\"is_price_drop\":false"));
    }

    #[test]
    fn test_availability_check_response_with_previous_price() {
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
            price_cents: Some(78900),
            price_currency: Some("USD".to_string()),
            raw_price: Some("789.00".to_string()),
        };

        let response =
            AvailabilityCheckResponse::from_model_with_previous_price(model, Some(89900));

        assert_eq!(response.price_cents, Some(78900));
        assert_eq!(response.previous_price_cents, Some(89900));
        assert!(response.is_price_drop);
    }

    #[test]
    fn test_availability_check_response_price_increase() {
        let id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let now = Utc::now();

        let model = AvailabilityCheckModel {
            id,
            product_id,
            status: "in_stock".to_string(),
            raw_availability: None,
            error_message: None,
            checked_at: now,
            price_cents: Some(99900),
            price_currency: Some("USD".to_string()),
            raw_price: None,
        };

        let response =
            AvailabilityCheckResponse::from_model_with_previous_price(model, Some(78900));

        assert_eq!(response.price_cents, Some(99900));
        assert_eq!(response.previous_price_cents, Some(78900));
        assert!(!response.is_price_drop);
    }

    #[test]
    fn test_availability_check_response_no_previous_price() {
        let id = Uuid::new_v4();
        let product_id = Uuid::new_v4();
        let now = Utc::now();

        let model = AvailabilityCheckModel {
            id,
            product_id,
            status: "in_stock".to_string(),
            raw_availability: None,
            error_message: None,
            checked_at: now,
            price_cents: Some(78900),
            price_currency: Some("USD".to_string()),
            raw_price: None,
        };

        let response = AvailabilityCheckResponse::from_model_with_previous_price(model, None);

        assert_eq!(response.price_cents, Some(78900));
        assert!(response.previous_price_cents.is_none());
        assert!(!response.is_price_drop);
    }
}
