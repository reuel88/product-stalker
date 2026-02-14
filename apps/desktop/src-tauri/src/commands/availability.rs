use serde::Serialize;
use tauri::State;

use crate::db::DbState;
use crate::domain::entities::prelude::AvailabilityCheckModel;
use crate::domain::services::currency;
use crate::domain::services::{AvailabilityService, BulkCheckSummary, DailyPriceComparison};
use crate::tauri_error::CommandError;
use crate::tauri_services::{send_desktop_notification, TauriAvailabilityService};
use crate::utils::parse_uuid;

/// Response DTO for availability checks
#[derive(Debug, Serialize)]
pub struct AvailabilityCheckResponse {
    pub id: String,
    pub product_id: String,
    pub product_retailer_id: Option<String>,
    pub status: String,
    pub raw_availability: Option<String>,
    pub error_message: Option<String>,
    pub checked_at: String,
    pub price_minor_units: Option<i64>,
    pub price_currency: Option<String>,
    pub raw_price: Option<String>,
    /// Currency exponent (number of decimal places: 0 for JPY, 2 for USD, 3 for KWD)
    pub currency_exponent: Option<u32>,
    /// Today's average price in minor units for daily comparison
    pub today_average_price_minor_units: Option<i64>,
    /// Yesterday's average price in minor units for daily comparison
    pub yesterday_average_price_minor_units: Option<i64>,
    /// True if today's average price is lower than yesterday's average
    pub is_price_drop: bool,
    /// Lowest current price across all retailers (minor units)
    pub lowest_price_minor_units: Option<i64>,
    /// Currency of the lowest price
    pub lowest_price_currency: Option<String>,
    /// Currency exponent for the lowest price
    pub lowest_currency_exponent: Option<u32>,
    /// Price normalized to the user's preferred currency (minor units)
    pub normalized_price_minor_units: Option<i64>,
    /// Currency code of the normalized price
    pub normalized_currency: Option<String>,
    /// Currency exponent for the normalized price
    pub normalized_currency_exponent: Option<u32>,
}

impl AvailabilityCheckResponse {
    /// Create a response from a model with daily price comparison data
    pub fn from_model_with_daily_comparison(
        model: AvailabilityCheckModel,
        daily_comparison: DailyPriceComparison,
    ) -> Self {
        let is_price_drop = AvailabilityService::is_price_drop(
            daily_comparison.yesterday_average_minor_units,
            daily_comparison.today_average_minor_units,
        );
        let currency_exponent = model
            .price_currency
            .as_deref()
            .map(currency::currency_exponent);
        let normalized_currency_exponent = model
            .normalized_currency
            .as_deref()
            .map(currency::currency_exponent);
        Self {
            id: model.id.to_string(),
            product_id: model.product_id.to_string(),
            product_retailer_id: model.product_retailer_id.map(|id| id.to_string()),
            status: model.status,
            raw_availability: model.raw_availability,
            error_message: model.error_message,
            checked_at: model.checked_at.to_rfc3339(),
            price_minor_units: model.price_minor_units,
            price_currency: model.price_currency,
            raw_price: model.raw_price,
            currency_exponent,
            today_average_price_minor_units: daily_comparison.today_average_minor_units,
            yesterday_average_price_minor_units: daily_comparison.yesterday_average_minor_units,
            is_price_drop,
            lowest_price_minor_units: None,
            lowest_price_currency: None,
            lowest_currency_exponent: None,
            normalized_price_minor_units: model.normalized_price_minor_units,
            normalized_currency: model.normalized_currency,
            normalized_currency_exponent,
        }
    }

    /// Set the lowest price fields from a cheapest price query result
    pub fn with_cheapest_price(
        mut self,
        cheapest: Option<crate::domain::repositories::CheapestPriceResult>,
    ) -> Self {
        if let Some(c) = cheapest {
            let exponent = currency::currency_exponent(&c.price_currency);
            self.lowest_price_minor_units = Some(c.price_minor_units);
            self.lowest_price_currency = Some(c.price_currency);
            self.lowest_currency_exponent = Some(exponent);
        }
        self
    }
}

impl From<AvailabilityCheckModel> for AvailabilityCheckResponse {
    fn from(model: AvailabilityCheckModel) -> Self {
        Self::from_model_with_daily_comparison(model, DailyPriceComparison::default())
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
) -> Result<AvailabilityCheckResponse, CommandError> {
    let uuid = parse_uuid(&product_id)?;

    let result = TauriAvailabilityService::check_product_with_notification(db.conn(), uuid).await?;

    if let Some(notification) = result.notification {
        send_desktop_notification(&app, &notification);
    }

    Ok(AvailabilityCheckResponse::from_model_with_daily_comparison(
        result.check,
        result.daily_comparison,
    ))
}

/// Get the latest availability check for a product
#[tauri::command]
pub async fn get_latest_availability(
    product_id: String,
    db: State<'_, DbState>,
) -> Result<Option<AvailabilityCheckResponse>, CommandError> {
    let uuid = parse_uuid(&product_id)?;

    let check = AvailabilityService::get_latest(db.conn(), uuid).await?;

    match check {
        Some(model) => {
            // Get daily price comparison for today vs yesterday
            let daily_comparison =
                AvailabilityService::get_daily_price_comparison(db.conn(), uuid).await?;
            // Get cheapest current price across all retailers
            let cheapest = AvailabilityService::get_cheapest_current_price(db.conn(), uuid).await?;
            Ok(Some(
                AvailabilityCheckResponse::from_model_with_daily_comparison(
                    model,
                    daily_comparison,
                )
                .with_cheapest_price(cheapest),
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
) -> Result<Vec<AvailabilityCheckResponse>, CommandError> {
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
) -> Result<BulkCheckSummary, CommandError> {
    let result =
        TauriAvailabilityService::check_all_products_with_notification(db.conn(), &app).await?;

    if let Some(notification) = result.notification {
        send_desktop_notification(&app, &notification);
    }

    Ok(result.summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn test_model() -> AvailabilityCheckModel {
        AvailabilityCheckModel {
            id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            product_retailer_id: None,
            status: "in_stock".to_string(),
            raw_availability: Some("http://schema.org/InStock".to_string()),
            error_message: None,
            checked_at: Utc::now(),
            price_minor_units: Some(78900),
            price_currency: Some("USD".to_string()),
            raw_price: Some("789.00".to_string()),
            normalized_price_minor_units: None,
            normalized_currency: None,
        }
    }

    #[test]
    fn test_availability_check_response_from_model() {
        let model = test_model();
        let id = model.id;
        let product_id = model.product_id;

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
        assert_eq!(response.price_minor_units, Some(78900));
        assert_eq!(response.price_currency, Some("USD".to_string()));
        assert_eq!(response.raw_price, Some("789.00".to_string()));
        assert_eq!(response.currency_exponent, Some(2));
        assert!(response.today_average_price_minor_units.is_none());
        assert!(response.yesterday_average_price_minor_units.is_none());
        assert!(!response.is_price_drop);
        assert!(response.lowest_price_minor_units.is_none());
        assert!(response.lowest_price_currency.is_none());
        assert!(response.lowest_currency_exponent.is_none());
        assert!(response.normalized_price_minor_units.is_none());
        assert!(response.normalized_currency.is_none());
        assert!(response.normalized_currency_exponent.is_none());
    }

    #[test]
    fn test_availability_check_response_with_error() {
        let model = AvailabilityCheckModel {
            status: "unknown".to_string(),
            raw_availability: None,
            error_message: Some("Failed to fetch page".to_string()),
            price_minor_units: None,
            price_currency: None,
            raw_price: None,
            ..test_model()
        };

        let response = AvailabilityCheckResponse::from(model);

        assert_eq!(response.status, "unknown");
        assert!(response.raw_availability.is_none());
        assert_eq!(
            response.error_message,
            Some("Failed to fetch page".to_string())
        );
        assert!(response.price_minor_units.is_none());
        assert!(response.currency_exponent.is_none());
        assert!(!response.is_price_drop);
        assert!(response.lowest_price_minor_units.is_none());
    }

    #[test]
    fn test_availability_check_response_serializes_to_json() {
        let model = AvailabilityCheckModel {
            status: "out_of_stock".to_string(),
            raw_availability: Some("http://schema.org/OutOfStock".to_string()),
            price_minor_units: Some(9999),
            price_currency: Some("EUR".to_string()),
            raw_price: Some("99.99".to_string()),
            ..test_model()
        };

        let id = model.id;
        let product_id = model.product_id;
        let response = AvailabilityCheckResponse::from(model);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("out_of_stock"));
        assert!(json.contains(&id.to_string()));
        assert!(json.contains(&product_id.to_string()));
        assert!(json.contains("9999"));
        assert!(json.contains("EUR"));
        assert!(json.contains("\"currency_exponent\":2"));
        assert!(json.contains("\"today_average_price_minor_units\":null"));
        assert!(json.contains("\"yesterday_average_price_minor_units\":null"));
        assert!(json.contains("\"is_price_drop\":false"));
        assert!(json.contains("\"lowest_price_minor_units\":null"));
        assert!(json.contains("\"lowest_price_currency\":null"));
        assert!(json.contains("\"lowest_currency_exponent\":null"));
        assert!(json.contains("\"normalized_price_minor_units\":null"));
        assert!(json.contains("\"normalized_currency\":null"));
        assert!(json.contains("\"normalized_currency_exponent\":null"));
    }

    #[test]
    fn test_availability_check_response_with_daily_comparison_price_drop() {
        let model = test_model();

        let daily_comparison = DailyPriceComparison {
            today_average_minor_units: Some(78900),
            yesterday_average_minor_units: Some(89900),
        };

        let response =
            AvailabilityCheckResponse::from_model_with_daily_comparison(model, daily_comparison);

        assert_eq!(response.price_minor_units, Some(78900));
        assert_eq!(response.currency_exponent, Some(2));
        assert_eq!(response.today_average_price_minor_units, Some(78900));
        assert_eq!(response.yesterday_average_price_minor_units, Some(89900));
        assert!(response.is_price_drop);
    }

    #[test]
    fn test_availability_check_response_with_daily_comparison_price_increase() {
        let model = AvailabilityCheckModel {
            raw_availability: None,
            price_minor_units: Some(99900),
            raw_price: None,
            ..test_model()
        };

        let daily_comparison = DailyPriceComparison {
            today_average_minor_units: Some(99900),
            yesterday_average_minor_units: Some(78900),
        };

        let response =
            AvailabilityCheckResponse::from_model_with_daily_comparison(model, daily_comparison);

        assert_eq!(response.price_minor_units, Some(99900));
        assert_eq!(response.currency_exponent, Some(2));
        assert_eq!(response.today_average_price_minor_units, Some(99900));
        assert_eq!(response.yesterday_average_price_minor_units, Some(78900));
        assert!(!response.is_price_drop);
    }

    #[test]
    fn test_availability_check_response_with_no_yesterday_data() {
        let model = AvailabilityCheckModel {
            raw_availability: None,
            raw_price: None,
            ..test_model()
        };

        let daily_comparison = DailyPriceComparison {
            today_average_minor_units: Some(78900),
            yesterday_average_minor_units: None,
        };

        let response =
            AvailabilityCheckResponse::from_model_with_daily_comparison(model, daily_comparison);

        assert_eq!(response.price_minor_units, Some(78900));
        assert_eq!(response.currency_exponent, Some(2));
        assert_eq!(response.today_average_price_minor_units, Some(78900));
        assert!(response.yesterday_average_price_minor_units.is_none());
        assert!(!response.is_price_drop);
    }

    #[test]
    fn test_with_cheapest_price_sets_fields() {
        use crate::domain::repositories::CheapestPriceResult;

        let response = AvailabilityCheckResponse::from(test_model());
        let cheapest = CheapestPriceResult {
            price_minor_units: 3000,
            price_currency: "AUD".to_string(),
        };

        let response = response.with_cheapest_price(Some(cheapest));

        assert_eq!(response.lowest_price_minor_units, Some(3000));
        assert_eq!(response.lowest_price_currency, Some("AUD".to_string()));
        assert_eq!(response.lowest_currency_exponent, Some(2));
    }

    #[test]
    fn test_with_cheapest_price_none_leaves_fields_null() {
        let response = AvailabilityCheckResponse::from(test_model()).with_cheapest_price(None);

        assert!(response.lowest_price_minor_units.is_none());
        assert!(response.lowest_price_currency.is_none());
        assert!(response.lowest_currency_exponent.is_none());
    }

    #[test]
    fn test_with_cheapest_price_serializes() {
        use crate::domain::repositories::CheapestPriceResult;

        let response = AvailabilityCheckResponse::from(test_model()).with_cheapest_price(Some(
            CheapestPriceResult {
                price_minor_units: 5000,
                price_currency: "JPY".to_string(),
            },
        ));

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"lowest_price_minor_units\":5000"));
        assert!(json.contains("\"lowest_price_currency\":\"JPY\""));
        assert!(json.contains("\"lowest_currency_exponent\":0"));
    }
}
