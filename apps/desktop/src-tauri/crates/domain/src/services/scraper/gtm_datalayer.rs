//! GTM dataLayer extraction for product price and availability data.
//!
//! Parses `dataLayer.push({...})` calls commonly injected by Google Tag Manager,
//! supporting GA4 ecommerce events, Enhanced Ecommerce, and legacy fields.

use scraper::{Html, Selector};
use serde::Deserialize;

use product_stalker_core::AppError;

use super::price_parser::{parse_price_to_minor_units, PriceInfo};
use super::ScrapingResult;
use crate::entities::availability_check::AvailabilityStatus;

/// A single GA4 ecommerce item within a `dataLayer.push()` event.
#[derive(Debug, Deserialize, Default)]
struct Ga4Item {
    price: Option<serde_json::Value>,
}

/// Top-level shape of a `dataLayer.push({...})` object.
///
/// Covers GA4 ecommerce (`event`, `currency`, `value`, `items`),
/// Enhanced Ecommerce (`ecommerce.detail.products`), and legacy
/// (`ecomm_totalvalue`) formats.
#[derive(Debug, Deserialize, Default)]
struct DataLayerPush {
    event: Option<String>,
    currency: Option<String>,
    value: Option<serde_json::Value>,
    items: Option<Vec<Ga4Item>>,
    ecommerce: Option<serde_json::Value>,
    ecomm_totalvalue: Option<serde_json::Value>,
}

/// GA4 event names in priority order for price extraction.
const GA4_EVENT_PRIORITY: &[&str] = &["view_item", "add_to_cart", "purchase", "begin_checkout"];

/// Button text indicators that suggest a product is available for purchase.
const ADD_TO_CART_INDICATORS: &[&str] = &[
    "add to cart",
    "add to bag",
    "add to basket",
    "buy now",
    "purchase",
    "in den warenkorb",
    "au panier",
    "カートに入れる",
    "カートに追加",
];

/// Extract product data from GTM dataLayer pushes in the HTML.
pub fn extract_from_datalayer(html: &str) -> Result<ScrapingResult, AppError> {
    let push_objects = extract_datalayer_push_strings(html)?;

    if push_objects.is_empty() {
        return Err(AppError::External(
            "No dataLayer.push() calls found".to_string(),
        ));
    }

    let parsed: Vec<DataLayerPush> = push_objects
        .iter()
        .filter_map(|s| {
            let json_str = normalize_js_to_json(s);
            serde_json::from_str(&json_str).ok()
        })
        .collect();

    if parsed.is_empty() {
        return Err(AppError::External(
            "No parseable dataLayer.push() objects found".to_string(),
        ));
    }

    // Try GA4 ecommerce events first (by priority)
    if let Some(price) = try_ga4_extraction(&parsed) {
        return Ok(build_result(html, price));
    }

    // Try Enhanced Ecommerce format
    if let Some(price) = try_enhanced_ecommerce_extraction(&parsed) {
        return Ok(build_result(html, price));
    }

    // Try legacy ecomm_totalvalue
    if let Some(price) = try_legacy_extraction(&parsed) {
        return Ok(build_result(html, price));
    }

    Err(AppError::External(
        "No ecommerce data found in dataLayer pushes".to_string(),
    ))
}

/// Build a ScrapingResult from inferred availability and extracted price.
fn build_result(html: &str, price: PriceInfo) -> ScrapingResult {
    let status = infer_availability(html);
    let raw_availability = Some(format!("gtm_datalayer:{}", status.as_str()));
    ScrapingResult {
        status,
        raw_availability,
        price,
    }
}

/// Extract raw JS object strings from `dataLayer.push({...})` calls in `<script>` tags.
fn extract_datalayer_push_strings(html: &str) -> Result<Vec<String>, AppError> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("script:not([src])")
        .map_err(|e| AppError::External(format!("Invalid selector: {:?}", e)))?;

    let mut results = Vec::new();

    for element in document.select(&selector) {
        let text = element.inner_html();
        let mut search_from = 0;

        while let Some(push_pos) = text[search_from..].find("dataLayer.push(") {
            let abs_pos = search_from + push_pos;
            let after_push = abs_pos + "dataLayer.push(".len();

            // Find the opening brace
            let Some(brace_start) = text[after_push..].find('{') else {
                search_from = after_push;
                continue;
            };
            let brace_abs = after_push + brace_start;

            if let Some(obj_str) = extract_balanced_braces(&text[brace_abs..]) {
                results.push(obj_str);
            }

            search_from = brace_abs + 1;
        }
    }

    Ok(results)
}

/// Tracks whether the parser is inside a JS string literal.
///
/// Makes it impossible to simultaneously be in both single-quoted and
/// double-quoted strings — a state that boolean flags could allow.
#[derive(Clone, Copy, PartialEq)]
enum StringContext {
    /// Not inside any string literal
    None,
    /// Inside a `'...'` string
    SingleQuoted,
    /// Inside a `"..."` string
    DoubleQuoted,
}

/// Extract a balanced `{...}` substring, handling nested braces and string literals.
fn extract_balanced_braces(s: &str) -> Option<String> {
    let mut depth = 0i32;
    let mut result = String::new();
    let mut string_ctx = StringContext::None;
    let mut escaped = false;

    for ch in s.chars() {
        result.push(ch);

        if escaped {
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        match string_ctx {
            StringContext::SingleQuoted => {
                if ch == '\'' {
                    string_ctx = StringContext::None;
                }
                continue;
            }
            StringContext::DoubleQuoted => {
                if ch == '"' {
                    string_ctx = StringContext::None;
                }
                continue;
            }
            StringContext::None => {}
        }

        match ch {
            '\'' => string_ctx = StringContext::SingleQuoted,
            '"' => string_ctx = StringContext::DoubleQuoted,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(result);
                }
            }
            _ => {}
        }
    }

    None
}

/// Normalize a JS object literal to valid JSON.
///
/// Converts single-quoted strings to double-quoted, handles unquoted keys,
/// and removes trailing commas.
fn normalize_js_to_json(js: &str) -> String {
    let mut result = String::with_capacity(js.len());
    let mut chars = js.chars().peekable();
    let mut string_ctx = StringContext::None;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            escaped = false;
            result.push(ch);
            continue;
        }

        if ch == '\\' {
            escaped = true;
            result.push(ch);
            continue;
        }

        match string_ctx {
            StringContext::DoubleQuoted => {
                if ch == '"' {
                    string_ctx = StringContext::None;
                }
                result.push(ch);
                continue;
            }
            StringContext::SingleQuoted => {
                if ch == '\'' {
                    string_ctx = StringContext::None;
                    result.push('"');
                } else if ch == '"' {
                    // Escape double quotes inside what was a single-quoted string
                    result.push('\\');
                    result.push('"');
                } else {
                    result.push(ch);
                }
                continue;
            }
            StringContext::None => {}
        }

        match ch {
            '\'' => {
                string_ctx = StringContext::SingleQuoted;
                result.push('"');
            }
            '"' => {
                string_ctx = StringContext::DoubleQuoted;
                result.push('"');
            }
            // Remove trailing commas before } or ]
            ',' => {
                // Peek ahead past whitespace to see if next significant char is } or ]
                let rest: String = chars.clone().collect();
                let trimmed = rest.trim_start();
                if trimmed.starts_with('}') || trimmed.starts_with(']') {
                    // Skip this trailing comma
                } else {
                    result.push(',');
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

/// Try to extract price from GA4 ecommerce events, prioritizing by event type.
fn try_ga4_extraction(pushes: &[DataLayerPush]) -> Option<PriceInfo> {
    // Try events in priority order
    for event_name in GA4_EVENT_PRIORITY {
        for push in pushes {
            let matches = push.event.as_ref().is_some_and(|e| e == *event_name);
            if !matches {
                continue;
            }

            let currency = push.currency.as_deref();

            // Try items[0].price first
            if let Some(items) = &push.items {
                if let Some(item) = items.first() {
                    if let Some(price) = extract_price_from_value(&item.price, currency) {
                        return Some(price);
                    }
                }
            }

            // Fall back to top-level value
            if let Some(price) = extract_price_from_value(&push.value, currency) {
                return Some(price);
            }
        }
    }

    // Try any push with items but no recognized event name
    for push in pushes {
        if push.event.is_some() {
            continue; // Already tried above
        }
        let currency = push.currency.as_deref();
        if let Some(items) = &push.items {
            if let Some(item) = items.first() {
                if let Some(price) = extract_price_from_value(&item.price, currency) {
                    return Some(price);
                }
            }
        }
    }

    None
}

/// Try to extract price from Enhanced Ecommerce `ecommerce.detail.products[0]`.
fn try_enhanced_ecommerce_extraction(pushes: &[DataLayerPush]) -> Option<PriceInfo> {
    for push in pushes {
        let ecommerce = push.ecommerce.as_ref()?;

        // Try ecommerce.detail.products[0].price
        let products = ecommerce
            .get("detail")
            .and_then(|d| d.get("products"))
            .and_then(|p| p.as_array());

        if let Some(products) = products {
            if let Some(product) = products.first() {
                let currency_code = ecommerce
                    .get("currencyCode")
                    .and_then(|c| c.as_str())
                    .or(push.currency.as_deref());

                let price_val = product.get("price");
                if let Some(price) = extract_price_from_value(&price_val.cloned(), currency_code) {
                    return Some(price);
                }
            }
        }
    }

    None
}

/// Try to extract price from legacy `ecomm_totalvalue` field.
fn try_legacy_extraction(pushes: &[DataLayerPush]) -> Option<PriceInfo> {
    for push in pushes {
        let currency = push.currency.as_deref();
        if let Some(price) = extract_price_from_value(&push.ecomm_totalvalue, currency) {
            return Some(price);
        }
    }
    None
}

/// Convert a serde_json::Value (string or number) into a PriceInfo.
fn extract_price_from_value(
    value: &Option<serde_json::Value>,
    currency: Option<&str>,
) -> Option<PriceInfo> {
    let val = value.as_ref()?;

    let raw_price = match val {
        serde_json::Value::String(s) if !s.is_empty() => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => return None,
    };

    let minor_units = parse_price_to_minor_units(&raw_price, currency);

    // Only return if we got a valid price
    minor_units?;

    Some(PriceInfo {
        price_minor_units: minor_units,
        price_currency: currency.map(|c| c.to_string()),
        raw_price: Some(raw_price),
    })
}

/// Infer product availability from HTML by searching for add-to-cart button indicators.
fn infer_availability(html: &str) -> AvailabilityStatus {
    let lower = html.to_lowercase();
    for indicator in ADD_TO_CART_INDICATORS {
        if lower.contains(indicator) {
            return AvailabilityStatus::InStock;
        }
    }
    AvailabilityStatus::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Brace extraction tests ---

    #[test]
    fn test_extract_balanced_braces_simple() {
        let input = r#"{"event": "view_item"}"#;
        assert_eq!(extract_balanced_braces(input), Some(input.to_string()));
    }

    #[test]
    fn test_extract_balanced_braces_nested() {
        let input = r#"{"ecommerce": {"detail": {"products": [{"price": "100"}]}}}"#;
        assert_eq!(extract_balanced_braces(input), Some(input.to_string()));
    }

    #[test]
    fn test_extract_balanced_braces_strings_with_braces() {
        let input = r#"{"name": "item {special}"}"#;
        assert_eq!(extract_balanced_braces(input), Some(input.to_string()));
    }

    #[test]
    fn test_extract_balanced_braces_escaped_quotes() {
        let input = r#"{"name": "item \"quoted\""}"#;
        assert_eq!(extract_balanced_braces(input), Some(input.to_string()));
    }

    #[test]
    fn test_extract_balanced_braces_single_quoted_strings() {
        let input = "{'name': 'item {special}'}";
        assert_eq!(extract_balanced_braces(input), Some(input.to_string()));
    }

    #[test]
    fn test_extract_balanced_braces_unbalanced() {
        let input = r#"{"event": "view_item""#;
        assert_eq!(extract_balanced_braces(input), None);
    }

    // --- JS to JSON normalization tests ---

    #[test]
    fn test_normalize_single_quotes() {
        let input = "{'event': 'view_item'}";
        let result = normalize_js_to_json(input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["event"], "view_item");
    }

    #[test]
    fn test_normalize_trailing_comma() {
        let input = r#"{"event": "view_item", "currency": "JPY",}"#;
        let result = normalize_js_to_json(input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["event"], "view_item");
    }

    #[test]
    fn test_normalize_double_quotes_inside_single_quoted() {
        let input = r#"{'name': 'item "special"'}"#;
        let result = normalize_js_to_json(input);
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["name"], r#"item "special""#);
    }

    // --- GA4 extraction tests ---

    #[test]
    fn test_ga4_view_item_jpy() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({
            "event": "view_item",
            "currency": "JPY",
            "value": 69300,
            "items": [{"item_id": "105946", "item_name": "Test Bag", "price": 69300}]
        });
        </script>
        </head><body><button>カートに入れる</button></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(69300));
        assert_eq!(result.price.price_currency, Some("JPY".to_string()));
        assert_eq!(result.status, AvailabilityStatus::InStock);
    }

    #[test]
    fn test_ga4_view_item_usd_with_decimals() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({
            "event": "view_item",
            "currency": "USD",
            "value": 49.99,
            "items": [{"item_id": "ABC", "item_name": "Shirt", "price": 49.99}]
        });
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(4999));
        assert_eq!(result.price.price_currency, Some("USD".to_string()));
    }

    #[test]
    fn test_ga4_string_price() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({
            "event": "view_item",
            "currency": "EUR",
            "items": [{"item_id": "X1", "item_name": "Hat", "price": "29.90"}]
        });
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(2990));
        assert_eq!(result.price.price_currency, Some("EUR".to_string()));
    }

    #[test]
    fn test_ga4_event_priority_view_item_over_add_to_cart() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({"event": "add_to_cart", "currency": "USD", "items": [{"price": 10.00}]});
        dataLayer.push({"event": "view_item", "currency": "USD", "items": [{"price": 25.00}]});
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        // view_item should be preferred over add_to_cart
        assert_eq!(result.price.price_minor_units, Some(2500));
    }

    #[test]
    fn test_ga4_fallback_to_top_level_value() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({
            "event": "view_item",
            "currency": "GBP",
            "value": 35.50
        });
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(3550));
        assert_eq!(result.price.price_currency, Some("GBP".to_string()));
    }

    #[test]
    fn test_ga4_missing_items_uses_value() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({
            "event": "view_item",
            "currency": "AUD",
            "value": 120.00
        });
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(12000));
        assert_eq!(result.price.price_currency, Some("AUD".to_string()));
    }

    // --- Enhanced Ecommerce tests ---

    #[test]
    fn test_enhanced_ecommerce_detail_products() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({
            "ecommerce": {
                "currencyCode": "USD",
                "detail": {
                    "products": [{"name": "Widget", "id": "W1", "price": "15.99"}]
                }
            }
        });
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(1599));
        assert_eq!(result.price.price_currency, Some("USD".to_string()));
    }

    #[test]
    fn test_enhanced_ecommerce_currency_code() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({
            "ecommerce": {
                "currencyCode": "CAD",
                "detail": {
                    "products": [{"name": "Maple Syrup", "price": "12.50"}]
                }
            }
        });
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_currency, Some("CAD".to_string()));
    }

    // --- Legacy tests ---

    #[test]
    fn test_legacy_ecomm_totalvalue() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({
            "ecomm_totalvalue": 89.99,
            "currency": "USD"
        });
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(8999));
    }

    // --- Availability inference tests ---

    #[test]
    fn test_infer_availability_add_to_cart_button() {
        let html = r#"<button class="btn">Add to Cart</button>"#;
        assert_eq!(infer_availability(html), AvailabilityStatus::InStock);
    }

    #[test]
    fn test_infer_availability_add_to_bag() {
        let html = r#"<button>Add to Bag</button>"#;
        assert_eq!(infer_availability(html), AvailabilityStatus::InStock);
    }

    #[test]
    fn test_infer_availability_buy_now() {
        let html = r#"<a href="/checkout">Buy Now</a>"#;
        assert_eq!(infer_availability(html), AvailabilityStatus::InStock);
    }

    #[test]
    fn test_infer_availability_japanese_cart() {
        let html = r#"<button>カートに入れる</button>"#;
        assert_eq!(infer_availability(html), AvailabilityStatus::InStock);
    }

    #[test]
    fn test_infer_availability_no_button_returns_unknown() {
        let html = r#"<div class="product"><p>Product description</p></div>"#;
        assert_eq!(infer_availability(html), AvailabilityStatus::Unknown);
    }

    // --- Error case tests ---

    #[test]
    fn test_no_datalayer_pushes_found() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>console.log("hello");</script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No dataLayer.push()"));
    }

    #[test]
    fn test_no_ecommerce_data_in_pushes() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({"event": "page_view", "page_title": "Home"});
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No ecommerce data"));
    }

    // --- End-to-end test matching yoshidakaban page structure ---

    #[test]
    fn test_full_html_yoshidakaban_style() {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Product Page</title>
<script>
var dataLayer = dataLayer || [];
dataLayer.push({'event': 'view_item', 'currency': 'JPY', 'value': 69300, 'items': [{'item_id': '105946', 'item_name': 'TANKER SHOULDER BAG', 'price': 69300, 'item_brand': 'PORTER', 'item_category': 'SHOULDER BAG'}]});
</script>
</head>
<body>
<div class="product-detail">
    <h1>TANKER SHOULDER BAG</h1>
    <p class="price">¥69,300</p>
    <button class="btn-cart">カートに入れる</button>
</div>
</body>
</html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(69300));
        assert_eq!(result.price.price_currency, Some("JPY".to_string()));
        assert_eq!(result.price.raw_price, Some("69300".to_string()));
        assert_eq!(result.status, AvailabilityStatus::InStock);
    }

    #[test]
    fn test_single_quoted_js_object() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({'event': 'view_item', 'currency': 'JPY', 'value': 5500, 'items': [{'item_id': '001', 'price': 5500}]});
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(5500));
        assert_eq!(result.price.price_currency, Some("JPY".to_string()));
    }

    #[test]
    fn test_multiple_script_tags() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        var gtag = function() {};
        </script>
        <script>
        dataLayer.push({"event": "view_item", "currency": "USD", "items": [{"price": 42.00}]});
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(4200));
    }

    #[test]
    fn test_datalayer_push_with_trailing_comma() {
        let html = r#"<!DOCTYPE html><html><head>
        <script>
        dataLayer.push({"event": "view_item", "currency": "USD", "items": [{"price": 19.99,}],});
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(1999));
    }

    #[test]
    fn test_external_script_tags_ignored() {
        // Script tags with src attribute should be ignored
        let html = r#"<!DOCTYPE html><html><head>
        <script src="https://cdn.example.com/analytics.js"></script>
        <script>
        dataLayer.push({"event": "view_item", "currency": "USD", "items": [{"price": 30.00}]});
        </script>
        </head><body></body></html>"#;

        let result = extract_from_datalayer(html).unwrap();
        assert_eq!(result.price.price_minor_units, Some(3000));
    }
}
