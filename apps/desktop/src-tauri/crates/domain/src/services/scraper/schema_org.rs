//! Schema.org JSON-LD parsing for extracting product availability and price data.

use scraper::{Html, Selector};
use url::Url;

use product_stalker_core::AppError;

use super::price_parser::{get_price_from_offer, PriceInfo};

/// Extract all JSON-LD blocks from HTML
pub fn extract_json_ld_blocks(html: &str) -> Result<Vec<serde_json::Value>, AppError> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("script[type=\"application/ld+json\"]")
        .map_err(|e| AppError::External(format!("Invalid selector: {:?}", e)))?;

    Ok(document
        .select(&selector)
        .filter_map(|el| serde_json::from_str(&el.inner_html()).ok())
        .collect())
}

/// Extract variant ID from URL query parameters
pub fn extract_variant_id(url: &str) -> Option<String> {
    Url::parse(url).ok().and_then(|parsed| {
        parsed
            .query_pairs()
            .find(|(key, _)| key == "variant")
            .map(|(_, value)| value.to_string())
    })
}

/// Extract availability and price from a JSON-LD value, trying multiple known structures.
///
/// Attempts extraction in the following priority order:
/// 1. **Direct Product** - JSON with `@type: "Product"` and `offers` containing availability
/// 2. **ProductGroup** - JSON with `@type: "ProductGroup"` and `hasVariant` array;
///    matches by `variant_id` if provided, otherwise uses the first variant
/// 3. **@graph array** - JSON with `@graph` array containing Product or ProductGroup items
/// 4. **Direct JSON array** - Top-level array containing Product or ProductGroup items
///
/// Returns `None` if no availability data is found in any of these structures.
pub fn extract_availability_and_price(
    json: &serde_json::Value,
    variant_id: Option<&str>,
) -> Option<(String, PriceInfo)> {
    // 1. Direct Product with offers
    if is_product_type(json) {
        if let Some(result) = get_availability_and_price_from_product(json) {
            return Some(result);
        }
    }

    // 2. ProductGroup with hasVariant array
    if is_product_group_type(json) {
        if let Some(result) = get_availability_and_price_from_product_group(json, variant_id) {
            return Some(result);
        }
    }

    // 3. @graph array containing Product or ProductGroup items
    if let Some(arr) = json.get("@graph").and_then(|g| g.as_array()) {
        if let Some(result) = find_availability_and_price_in_items(arr, variant_id) {
            return Some(result);
        }
    }

    // 4. Direct JSON array containing Product or ProductGroup items
    if let Some(arr) = json.as_array() {
        if let Some(result) = find_availability_and_price_in_items(arr, variant_id) {
            return Some(result);
        }
    }

    None
}

/// Iterate through items looking for availability and price data
fn find_availability_and_price_in_items(
    items: &[serde_json::Value],
    variant_id: Option<&str>,
) -> Option<(String, PriceInfo)> {
    items.iter().find_map(|item| {
        if is_product_type(item) {
            if let Some(result) = get_availability_and_price_from_product(item) {
                return Some(result);
            }
        }
        if is_product_group_type(item) {
            return get_availability_and_price_from_product_group(item, variant_id);
        }
        None
    })
}

/// Check if a JSON @type field matches the expected type
fn has_schema_type(json: &serde_json::Value, expected_type: &str) -> bool {
    let Some(type_value) = json.get("@type") else {
        return false;
    };

    match type_value {
        serde_json::Value::String(s) => s == expected_type,
        serde_json::Value::Array(arr) => arr.iter().any(|v| v.as_str() == Some(expected_type)),
        _ => false,
    }
}

/// Check if a JSON value represents a Product type
fn is_product_type(json: &serde_json::Value) -> bool {
    has_schema_type(json, "Product")
}

/// Check if a JSON value represents a ProductGroup type
fn is_product_group_type(json: &serde_json::Value) -> bool {
    has_schema_type(json, "ProductGroup")
}

/// Get availability and price from a ProductGroup by matching variant ID
fn get_availability_and_price_from_product_group(
    product_group: &serde_json::Value,
    variant_id: Option<&str>,
) -> Option<(String, PriceInfo)> {
    let variants = product_group.get("hasVariant")?.as_array()?;

    let Some(vid) = variant_id else {
        // No variant ID specified, return first variant's availability and price
        return get_first_variant_availability(variants);
    };

    // Try to find the matching variant by ID
    let matched = find_variant_by_id(variants, vid);
    if matched.is_some() {
        return matched;
    }

    // Fallback: return first variant's availability and price
    get_first_variant_availability(variants)
}

/// Find a variant by its ID in the URL query parameters
fn find_variant_by_id(variants: &[serde_json::Value], vid: &str) -> Option<(String, PriceInfo)> {
    // Dummy base for resolving relative URLs (host is irrelevant)
    let base = Url::parse("http://localhost").unwrap();

    for variant in variants {
        let Some(id) = variant.get("@id").and_then(|i| i.as_str()) else {
            continue;
        };
        let Some(parsed_url) = Url::parse(id).or_else(|_| base.join(id)).ok() else {
            continue;
        };

        let matches_variant = parsed_url
            .query_pairs()
            .any(|(key, value)| key == "variant" && value == vid);

        if !matches_variant {
            continue;
        }

        if let Some(result) = get_availability_and_price_from_product(variant) {
            return Some(result);
        }
    }

    None
}

/// Get the first variant's availability and price
fn get_first_variant_availability(variants: &[serde_json::Value]) -> Option<(String, PriceInfo)> {
    variants
        .iter()
        .find_map(get_availability_and_price_from_product)
}

/// Get availability and price from a Product JSON object
fn get_availability_and_price_from_product(
    product: &serde_json::Value,
) -> Option<(String, PriceInfo)> {
    let offers = product.get("offers")?;

    // Single offer object
    if let Some(avail) = offers.get("availability").and_then(|a| a.as_str()) {
        let price = get_price_from_offer(offers);
        return Some((avail.to_string(), price));
    }

    // Array of offers - use first one with availability
    offers.as_array().and_then(|arr| {
        arr.iter().find_map(|offer| {
            let avail = offer.get("availability")?.as_str()?;
            let price = get_price_from_offer(offer);
            Some((avail.to_string(), price))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_variant_id() {
        assert_eq!(
            extract_variant_id("https://example.com/products/test?variant=12345"),
            Some("12345".to_string())
        );
        assert_eq!(
            extract_variant_id("https://example.com/products/test"),
            None
        );
        assert_eq!(
            extract_variant_id("https://example.com/products/test?foo=bar&variant=999&baz=qux"),
            Some("999".to_string())
        );
    }

    #[test]
    fn test_extract_json_ld_blocks() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {"@type": "Product", "name": "Test"}
                </script>
            </head>
            <body></body>
            </html>
        "#;
        let blocks = extract_json_ld_blocks(html).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["@type"], "Product");
    }

    #[test]
    fn test_extract_availability_from_product() {
        let json = serde_json::json!({
            "@type": "Product",
            "name": "Test",
            "offers": {
                "availability": "http://schema.org/InStock",
                "price": "99.99",
                "priceCurrency": "USD"
            }
        });
        let result = extract_availability_and_price(&json, None);
        assert!(result.is_some());
        let (avail, price) = result.unwrap();
        assert_eq!(avail, "http://schema.org/InStock");
        assert_eq!(price.price_minor_units, Some(9999));
    }

    #[test]
    fn test_extract_availability_from_product_group() {
        let json = serde_json::json!({
            "@type": "ProductGroup",
            "hasVariant": [
                {
                    "@id": "/products/test?variant=123#variant",
                    "@type": "Product",
                    "offers": {
                        "availability": "http://schema.org/OutOfStock"
                    }
                },
                {
                    "@id": "/products/test?variant=456#variant",
                    "@type": "Product",
                    "offers": {
                        "availability": "http://schema.org/InStock"
                    }
                }
            ]
        });

        // With matching variant ID
        let result = extract_availability_and_price(&json, Some("456"));
        assert!(result.is_some());
        let (avail, _) = result.unwrap();
        assert_eq!(avail, "http://schema.org/InStock");

        // Without variant ID - gets first variant
        let result = extract_availability_and_price(&json, None);
        assert!(result.is_some());
        let (avail, _) = result.unwrap();
        assert_eq!(avail, "http://schema.org/OutOfStock");
    }

    #[test]
    fn test_extract_availability_from_graph() {
        let json = serde_json::json!({
            "@graph": [
                {"@type": "WebSite", "name": "Test"},
                {
                    "@type": "Product",
                    "name": "Test Product",
                    "offers": {
                        "availability": "http://schema.org/InStock"
                    }
                }
            ]
        });
        let result = extract_availability_and_price(&json, None);
        assert!(result.is_some());
        let (avail, _) = result.unwrap();
        assert_eq!(avail, "http://schema.org/InStock");
    }

    #[test]
    fn test_extract_availability_from_array() {
        let json = serde_json::json!([
            {"@type": "Organization", "name": "Test"},
            {
                "@type": "Product",
                "name": "Test Product",
                "offers": {
                    "availability": "http://schema.org/BackOrder"
                }
            }
        ]);
        let result = extract_availability_and_price(&json, None);
        assert!(result.is_some());
        let (avail, _) = result.unwrap();
        assert_eq!(avail, "http://schema.org/BackOrder");
    }

    #[test]
    fn test_extract_availability_array_of_offers() {
        let json = serde_json::json!({
            "@type": "Product",
            "offers": [
                {"availability": "http://schema.org/OutOfStock", "price": "49.99"},
                {"availability": "http://schema.org/InStock", "price": "99.99"}
            ]
        });
        let result = extract_availability_and_price(&json, None);
        assert!(result.is_some());
        let (avail, price) = result.unwrap();
        // Should use first offer's availability
        assert_eq!(avail, "http://schema.org/OutOfStock");
        assert_eq!(price.price_minor_units, Some(4999));
    }
}
