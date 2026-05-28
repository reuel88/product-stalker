//! Decathlon adapter for parsing product data from Next.js props.
//!
//! Decathlon's storefronts (decathlon.com.au, etc.) are Next.js apps that embed
//! product data in the `__NEXT_DATA__` script rather than emitting Schema.org
//! JSON-LD. The relevant slice of `pageProps` looks like:
//!
//! ```json
//! {
//!   "product": {
//!     "name": "...",
//!     "price": { "value": 19.99, "currency": "AUD" },
//!     "items": [
//!       { "itemId": "...", "currentPrice": 19.99, "currency": "AUD", "desactivated": false }
//!     ]
//!   }
//! }
//! ```
//!
//! Availability is derived from `items[].desactivated` (note: Decathlon's own
//! misspelling of "deactivated").

use serde_json::Value;

use crate::entities::availability_check::AvailabilityStatus;
use product_stalker_core::AppError;

use super::price_parser::{parse_price_to_minor_units, PriceInfo};
use super::ScrapingResult;

/// Check if the URL is for Decathlon Australia.
pub fn is_decathlon_url(url: &str) -> bool {
    url.contains("decathlon.com.au")
}

/// Parse product availability and price from Decathlon's Next.js page props.
pub fn parse_decathlon_data(page_props: &Value) -> Result<ScrapingResult, AppError> {
    let product = page_props.get("product").ok_or_else(|| {
        AppError::External("No product data found in Decathlon page props".to_string())
    })?;

    let price = extract_price_info(product);
    let (status, raw_availability) = derive_availability(product);

    Ok(ScrapingResult {
        status,
        raw_availability: Some(raw_availability),
        price,
    })
}

/// Decathlon prices live at `product.price.value` with the currency at
/// `product.price.currency`. Both URLs verified on decathlon.com.au use AUD,
/// so fall back to AUD when the currency field is absent.
fn extract_price_info(product: &Value) -> PriceInfo {
    let price_obj = product.get("price");

    let raw_price = price_obj
        .and_then(|p| p.get("value"))
        .and_then(value_as_string);

    let price_currency = price_obj
        .and_then(|p| p.get("currency"))
        .and_then(|c| c.as_str())
        .map(str::to_string)
        .or_else(|| {
            if raw_price.is_some() {
                Some("AUD".to_string())
            } else {
                None
            }
        });

    let price_minor_units = raw_price
        .as_ref()
        .and_then(|p| parse_price_to_minor_units(p, price_currency.as_deref()));

    PriceInfo {
        price_minor_units,
        price_currency,
        raw_price,
    }
}

/// Availability comes from the `items` array — each item represents a variant
/// (size) and carries a `desactivated` flag. If any item is active the product
/// is in stock; an empty/missing items array means nothing is buyable.
fn derive_availability(product: &Value) -> (AvailabilityStatus, String) {
    let Some(items) = product.get("items").and_then(|v| v.as_array()) else {
        return (AvailabilityStatus::OutOfStock, "no-items".to_string());
    };

    if items.is_empty() {
        return (AvailabilityStatus::OutOfStock, "no-items".to_string());
    }

    let any_active = items
        .iter()
        .any(|item| item.get("desactivated").and_then(|v| v.as_bool()) == Some(false));

    if any_active {
        (AvailabilityStatus::InStock, "in-stock".to_string())
    } else {
        (AvailabilityStatus::OutOfStock, "out-of-stock".to_string())
    }
}

/// Convert a JSON String or Number to its string form (same helper shape as
/// the Chemist Warehouse adapter uses for prices).
fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_decathlon_url() {
        assert!(is_decathlon_url(
            "https://www.decathlon.com.au/p/men-s-surfing-long-sleeved-uv-protection-top-t-shirt-100-grey-decathlon-8611954.html"
        ));
        assert!(is_decathlon_url(
            "https://decathlon.com.au/p/something.html"
        ));
        assert!(!is_decathlon_url("https://amazon.com.au/product"));
        assert!(!is_decathlon_url(
            "https://www.chemistwarehouse.com.au/buy/123/test"
        ));
        assert!(!is_decathlon_url("https://example.com"));
    }

    fn product_json(price: Value, items: Value) -> Value {
        serde_json::json!({
            "product": {
                "name": "Test Product",
                "price": price,
                "items": items,
            }
        })
    }

    #[test]
    fn test_parse_decathlon_in_stock() {
        let page_props = product_json(
            serde_json::json!({ "value": 19.99, "currency": "AUD" }),
            serde_json::json!([
                { "itemId": "1", "desactivated": false },
                { "itemId": "2", "desactivated": true }
            ]),
        );

        let result = parse_decathlon_data(&page_props).unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
        assert_eq!(result.raw_availability, Some("in-stock".to_string()));
        assert_eq!(result.price.price_minor_units, Some(1999));
        assert_eq!(result.price.price_currency, Some("AUD".to_string()));
        assert_eq!(result.price.raw_price, Some("19.99".to_string()));
    }

    #[test]
    fn test_parse_decathlon_out_of_stock_all_desactivated() {
        let page_props = product_json(
            serde_json::json!({ "value": 19.99, "currency": "AUD" }),
            serde_json::json!([
                { "itemId": "1", "desactivated": true },
                { "itemId": "2", "desactivated": true }
            ]),
        );

        let result = parse_decathlon_data(&page_props).unwrap();
        assert_eq!(result.status, AvailabilityStatus::OutOfStock);
        assert_eq!(result.raw_availability, Some("out-of-stock".to_string()));
    }

    #[test]
    fn test_parse_decathlon_out_of_stock_no_items() {
        let page_props = product_json(
            serde_json::json!({ "value": 19.99, "currency": "AUD" }),
            serde_json::json!([]),
        );

        let result = parse_decathlon_data(&page_props).unwrap();
        assert_eq!(result.status, AvailabilityStatus::OutOfStock);
        assert_eq!(result.raw_availability, Some("no-items".to_string()));
    }

    #[test]
    fn test_parse_decathlon_missing_items_field() {
        let page_props = serde_json::json!({
            "product": {
                "name": "Test Product",
                "price": { "value": 19.99, "currency": "AUD" }
            }
        });

        let result = parse_decathlon_data(&page_props).unwrap();
        assert_eq!(result.status, AvailabilityStatus::OutOfStock);
        assert_eq!(result.raw_availability, Some("no-items".to_string()));
    }

    #[test]
    fn test_parse_decathlon_integer_price() {
        let page_props = product_json(
            serde_json::json!({ "value": 7, "currency": "AUD" }),
            serde_json::json!([{ "itemId": "1", "desactivated": false }]),
        );

        let result = parse_decathlon_data(&page_props).unwrap();
        assert_eq!(result.price.price_minor_units, Some(700));
        assert_eq!(result.price.raw_price, Some("7".to_string()));
    }

    #[test]
    fn test_parse_decathlon_missing_currency_defaults_aud() {
        let page_props = product_json(
            serde_json::json!({ "value": 12.50 }),
            serde_json::json!([{ "itemId": "1", "desactivated": false }]),
        );

        let result = parse_decathlon_data(&page_props).unwrap();
        assert_eq!(result.price.price_currency, Some("AUD".to_string()));
        assert_eq!(result.price.price_minor_units, Some(1250));
    }

    #[test]
    fn test_parse_decathlon_no_product() {
        let page_props = serde_json::json!({ "something": "else" });

        let result = parse_decathlon_data(&page_props);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::External(msg) => assert!(msg.contains("No product data")),
            other => panic!("Expected External error, got {:?}", other),
        }
    }
}
