//! Chemist Warehouse adapter for parsing product data from Next.js props.
//!
//! Chemist Warehouse uses Next.js and embeds product data in the __NEXT_DATA__ script
//! instead of using Schema.org markup. This module provides the adapter to extract
//! availability and price information from their specific data structure.

use serde_json::Value;

use crate::entities::availability_check::AvailabilityStatus;
use product_stalker_core::AppError;

use super::price_parser::{parse_price_to_cents, PriceInfo};
use super::ScrapingResult;

/// Check if the URL is for Chemist Warehouse
pub fn is_chemist_warehouse_url(url: &str) -> bool {
    url.contains("chemistwarehouse.com.au")
}

/// Parse product availability from Chemist Warehouse Next.js data.
///
/// The expected structure within pageProps varies, but typically includes:
/// ```json
/// {
///   "product": {
///     "name": "Product Name",
///     "sku": "123456",
///     "price": "23.99",
///     "availability": "in-stock"
///   }
/// }
/// ```
///
/// or may be nested under different paths like "productDetail" or at root level.
pub fn parse_chemist_warehouse_data(page_props: &Value) -> Result<ScrapingResult, AppError> {
    // Try different possible paths where product data might be located
    let product = find_product_data(page_props).ok_or_else(|| {
        AppError::Scraping("No product data found in Chemist Warehouse page props".to_string())
    })?;

    // Extract availability
    let availability_str = extract_availability(product).ok_or_else(|| {
        AppError::Scraping("No availability found in Chemist Warehouse product data".to_string())
    })?;

    let status = map_availability_status(&availability_str);
    let price = extract_price_info(product);

    Ok(ScrapingResult {
        status,
        raw_availability: Some(availability_str),
        price,
    })
}

/// Find the product data object from various possible locations in pageProps
fn find_product_data(page_props: &Value) -> Option<&Value> {
    // Common paths where product data might be:
    // - pageProps.product
    // - pageProps.productDetail
    // - pageProps.data.product
    // - pageProps (if product fields are at root)

    if let Some(product) = page_props.get("product") {
        if has_product_fields(product) {
            return Some(product);
        }
    }

    if let Some(product_detail) = page_props.get("productDetail") {
        if has_product_fields(product_detail) {
            return Some(product_detail);
        }
    }

    if let Some(data) = page_props.get("data") {
        if let Some(product) = data.get("product") {
            if has_product_fields(product) {
                return Some(product);
            }
        }
    }

    // Check if pageProps itself contains product fields
    if has_product_fields(page_props) {
        return Some(page_props);
    }

    None
}

/// Check if a value has typical product fields
fn has_product_fields(value: &Value) -> bool {
    // A product should have at least a name or sku, and availability or price
    let has_identifier = value.get("name").is_some()
        || value.get("sku").is_some()
        || value.get("productName").is_some();

    let has_stock_info = value.get("availability").is_some()
        || value.get("stock").is_some()
        || value.get("stockStatus").is_some()
        || value.get("inStock").is_some()
        || value.get("price").is_some();

    has_identifier && has_stock_info
}

/// Extract availability string from product data
fn extract_availability(product: &Value) -> Option<String> {
    // Try different field names
    if let Some(availability) = product.get("availability").and_then(|v| v.as_str()) {
        return Some(availability.to_string());
    }

    if let Some(stock_status) = product.get("stockStatus").and_then(|v| v.as_str()) {
        return Some(stock_status.to_string());
    }

    if let Some(stock) = product.get("stock").and_then(|v| v.as_str()) {
        return Some(stock.to_string());
    }

    // Handle boolean inStock field
    if let Some(in_stock) = product.get("inStock").and_then(|v| v.as_bool()) {
        return Some(if in_stock {
            "in-stock".to_string()
        } else {
            "out-of-stock".to_string()
        });
    }

    None
}

/// Map Chemist Warehouse availability strings to AvailabilityStatus
fn map_availability_status(availability: &str) -> AvailabilityStatus {
    let normalized = availability.to_lowercase();

    if normalized == "in-stock"
        || normalized == "instock"
        || normalized == "in stock"
        || normalized == "available"
    {
        AvailabilityStatus::InStock
    } else if normalized == "out-of-stock"
        || normalized == "outofstock"
        || normalized == "out of stock"
        || normalized == "unavailable"
        || normalized == "sold out"
        || normalized == "soldout"
    {
        AvailabilityStatus::OutOfStock
    } else if normalized == "backorder"
        || normalized == "back-order"
        || normalized == "back order"
        || normalized == "preorder"
        || normalized == "pre-order"
        || normalized == "pre order"
    {
        AvailabilityStatus::BackOrder
    } else {
        AvailabilityStatus::Unknown
    }
}

/// Extract price information from product data
fn extract_price_info(product: &Value) -> PriceInfo {
    // Try different price field names
    let raw_price = extract_price_string(product);
    let price_cents = raw_price.as_ref().and_then(|p| parse_price_to_cents(p));

    // Chemist Warehouse is Australian, so default to AUD if no currency specified
    let price_currency = product
        .get("currency")
        .and_then(|c| c.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            product
                .get("priceCurrency")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            // Default to AUD for Chemist Warehouse
            if raw_price.is_some() {
                Some("AUD".to_string())
            } else {
                None
            }
        });

    PriceInfo {
        price_cents,
        price_currency,
        raw_price,
    }
}

/// Try to extract a string representation from a JSON value (String or Number)
fn value_as_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Extract price string from various possible field names
fn extract_price_string(product: &Value) -> Option<String> {
    // Try direct price fields
    for key in ["price", "currentPrice", "salePrice"] {
        if let Some(result) = product.get(key).and_then(value_as_string) {
            return Some(result);
        }
    }

    // Try nested pricing object
    if let Some(pricing) = product.get("pricing") {
        if let Some(result) = pricing.get("price").and_then(value_as_string) {
            return Some(result);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_chemist_warehouse_url() {
        assert!(is_chemist_warehouse_url(
            "https://www.chemistwarehouse.com.au/buy/87324/curash-simply-water-wipes"
        ));
        assert!(is_chemist_warehouse_url(
            "https://chemistwarehouse.com.au/buy/12345/product"
        ));
        assert!(!is_chemist_warehouse_url("https://amazon.com.au/product"));
        assert!(!is_chemist_warehouse_url("https://example.com"));
    }

    #[test]
    fn test_map_availability_status_in_stock() {
        assert_eq!(
            map_availability_status("in-stock"),
            AvailabilityStatus::InStock
        );
        assert_eq!(
            map_availability_status("In-Stock"),
            AvailabilityStatus::InStock
        );
        assert_eq!(
            map_availability_status("IN-STOCK"),
            AvailabilityStatus::InStock
        );
        assert_eq!(
            map_availability_status("instock"),
            AvailabilityStatus::InStock
        );
        assert_eq!(
            map_availability_status("in stock"),
            AvailabilityStatus::InStock
        );
        assert_eq!(
            map_availability_status("available"),
            AvailabilityStatus::InStock
        );
    }

    #[test]
    fn test_map_availability_status_out_of_stock() {
        assert_eq!(
            map_availability_status("out-of-stock"),
            AvailabilityStatus::OutOfStock
        );
        assert_eq!(
            map_availability_status("Out-Of-Stock"),
            AvailabilityStatus::OutOfStock
        );
        assert_eq!(
            map_availability_status("outofstock"),
            AvailabilityStatus::OutOfStock
        );
        assert_eq!(
            map_availability_status("out of stock"),
            AvailabilityStatus::OutOfStock
        );
        assert_eq!(
            map_availability_status("unavailable"),
            AvailabilityStatus::OutOfStock
        );
        assert_eq!(
            map_availability_status("sold out"),
            AvailabilityStatus::OutOfStock
        );
    }

    #[test]
    fn test_map_availability_status_back_order() {
        assert_eq!(
            map_availability_status("backorder"),
            AvailabilityStatus::BackOrder
        );
        assert_eq!(
            map_availability_status("back-order"),
            AvailabilityStatus::BackOrder
        );
        assert_eq!(
            map_availability_status("preorder"),
            AvailabilityStatus::BackOrder
        );
        assert_eq!(
            map_availability_status("pre-order"),
            AvailabilityStatus::BackOrder
        );
    }

    #[test]
    fn test_map_availability_status_unknown() {
        assert_eq!(
            map_availability_status("something-else"),
            AvailabilityStatus::Unknown
        );
        assert_eq!(map_availability_status(""), AvailabilityStatus::Unknown);
    }

    #[test]
    fn test_parse_chemist_warehouse_data_standard_structure() {
        let page_props = serde_json::json!({
            "product": {
                "name": "Curash Simply Water Wipes 6 x 80 Pack",
                "sku": "2678514",
                "price": "23.99",
                "availability": "in-stock"
            }
        });

        let result = parse_chemist_warehouse_data(&page_props).unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
        assert_eq!(result.raw_availability, Some("in-stock".to_string()));
        assert_eq!(result.price.price_cents, Some(2399));
        assert_eq!(result.price.price_currency, Some("AUD".to_string()));
        assert_eq!(result.price.raw_price, Some("23.99".to_string()));
    }

    #[test]
    fn test_parse_chemist_warehouse_data_out_of_stock() {
        let page_props = serde_json::json!({
            "product": {
                "name": "Test Product",
                "sku": "12345",
                "price": "19.99",
                "availability": "out-of-stock"
            }
        });

        let result = parse_chemist_warehouse_data(&page_props).unwrap();
        assert_eq!(result.status, AvailabilityStatus::OutOfStock);
        assert_eq!(result.raw_availability, Some("out-of-stock".to_string()));
    }

    #[test]
    fn test_parse_chemist_warehouse_data_product_detail_path() {
        let page_props = serde_json::json!({
            "productDetail": {
                "name": "Another Product",
                "price": "49.99",
                "availability": "in-stock"
            }
        });

        let result = parse_chemist_warehouse_data(&page_props).unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
        assert_eq!(result.price.price_cents, Some(4999));
    }

    #[test]
    fn test_parse_chemist_warehouse_data_nested_data_path() {
        let page_props = serde_json::json!({
            "data": {
                "product": {
                    "name": "Nested Product",
                    "price": "15.00",
                    "availability": "in-stock"
                }
            }
        });

        let result = parse_chemist_warehouse_data(&page_props).unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
        assert_eq!(result.price.price_cents, Some(1500));
    }

    #[test]
    fn test_parse_chemist_warehouse_data_numeric_price() {
        let page_props = serde_json::json!({
            "product": {
                "name": "Test Product",
                "price": 29.99,
                "availability": "in-stock"
            }
        });

        let result = parse_chemist_warehouse_data(&page_props).unwrap();
        assert_eq!(result.price.price_cents, Some(2999));
        assert_eq!(result.price.raw_price, Some("29.99".to_string()));
    }

    #[test]
    fn test_parse_chemist_warehouse_data_boolean_in_stock() {
        let page_props = serde_json::json!({
            "product": {
                "name": "Test Product",
                "price": "10.00",
                "inStock": true
            }
        });

        let result = parse_chemist_warehouse_data(&page_props).unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
    }

    #[test]
    fn test_parse_chemist_warehouse_data_boolean_out_of_stock() {
        let page_props = serde_json::json!({
            "product": {
                "name": "Test Product",
                "price": "10.00",
                "inStock": false
            }
        });

        let result = parse_chemist_warehouse_data(&page_props).unwrap();
        assert_eq!(result.status, AvailabilityStatus::OutOfStock);
    }

    #[test]
    fn test_parse_chemist_warehouse_data_with_currency() {
        let page_props = serde_json::json!({
            "product": {
                "name": "Test Product",
                "price": "50.00",
                "currency": "AUD",
                "availability": "in-stock"
            }
        });

        let result = parse_chemist_warehouse_data(&page_props).unwrap();
        assert_eq!(result.price.price_currency, Some("AUD".to_string()));
    }

    #[test]
    fn test_parse_chemist_warehouse_data_no_product() {
        let page_props = serde_json::json!({
            "otherData": "not a product"
        });

        let result = parse_chemist_warehouse_data(&page_props);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::Scraping(msg) => {
                assert!(msg.contains("No product data found"));
            }
            _ => panic!("Expected Scraping error"),
        }
    }

    #[test]
    fn test_parse_chemist_warehouse_data_no_availability() {
        let page_props = serde_json::json!({
            "product": {
                "name": "Test Product",
                "price": "10.00"
            }
        });

        let result = parse_chemist_warehouse_data(&page_props);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::Scraping(msg) => {
                assert!(msg.contains("No availability found"));
            }
            _ => panic!("Expected Scraping error"),
        }
    }

    #[test]
    fn test_extract_price_from_current_price() {
        let product = serde_json::json!({
            "name": "Test",
            "currentPrice": "35.50",
            "availability": "in-stock"
        });

        let price = extract_price_info(&product);
        assert_eq!(price.price_cents, Some(3550));
        assert_eq!(price.raw_price, Some("35.50".to_string()));
    }

    #[test]
    fn test_extract_price_from_nested_pricing() {
        let product = serde_json::json!({
            "name": "Test",
            "pricing": {
                "price": "42.00"
            },
            "availability": "in-stock"
        });

        let price = extract_price_info(&product);
        assert_eq!(price.price_cents, Some(4200));
    }

    #[test]
    fn test_has_product_fields_valid() {
        let product = serde_json::json!({
            "name": "Product",
            "availability": "in-stock"
        });
        assert!(has_product_fields(&product));

        let product = serde_json::json!({
            "sku": "123",
            "price": "10.00"
        });
        assert!(has_product_fields(&product));
    }

    #[test]
    fn test_has_product_fields_invalid() {
        let not_product = serde_json::json!({
            "randomField": "value"
        });
        assert!(!has_product_fields(&not_product));

        let only_name = serde_json::json!({
            "name": "Something"
        });
        assert!(!has_product_fields(&only_name));
    }
}
