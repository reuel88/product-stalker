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
    url: &str,
) -> Option<(String, PriceInfo)> {
    // 1. Direct Product with offers
    if is_product_type(json) {
        if let Some(result) = get_availability_and_price_from_product(json, url) {
            return Some(result);
        }
    }

    // 2. ProductGroup with hasVariant array
    if is_product_group_type(json) {
        if let Some(result) = get_availability_and_price_from_product_group(json, variant_id, url) {
            return Some(result);
        }
    }

    // 3. @graph array containing Product or ProductGroup items
    if let Some(arr) = json.get("@graph").and_then(|g| g.as_array()) {
        if let Some(result) = find_availability_and_price_in_items(arr, variant_id, url) {
            return Some(result);
        }
    }

    // 4. Direct JSON array containing Product or ProductGroup items
    if let Some(arr) = json.as_array() {
        if let Some(result) = find_availability_and_price_in_items(arr, variant_id, url) {
            return Some(result);
        }
    }

    None
}

/// Iterate through items looking for availability and price data
fn find_availability_and_price_in_items(
    items: &[serde_json::Value],
    variant_id: Option<&str>,
    url: &str,
) -> Option<(String, PriceInfo)> {
    items.iter().find_map(|item| {
        if is_product_type(item) {
            if let Some(result) = get_availability_and_price_from_product(item, url) {
                return Some(result);
            }
        }
        if is_product_group_type(item) {
            return get_availability_and_price_from_product_group(item, variant_id, url);
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

/// Whether a JSON object lacks a usable `@type` field.
///
/// Some storefronts (notably Magento, e.g. bonds.com.au) emit Product/ProductGroup
/// JSON-LD with no `@type` at all. When `@type` is missing we fall back to
/// structural detection based on the fields that are present.
fn lacks_type(json: &serde_json::Value) -> bool {
    match json.get("@type") {
        None | Some(serde_json::Value::Null) => true,
        Some(serde_json::Value::String(s)) => s.is_empty(),
        Some(serde_json::Value::Array(arr)) => arr.is_empty(),
        _ => false,
    }
}

/// Whether the JSON object carries a non-empty `hasVariant` array.
fn has_variants(json: &serde_json::Value) -> bool {
    json.get("hasVariant")
        .and_then(|v| v.as_array())
        .is_some_and(|arr| !arr.is_empty())
}

/// Check if a JSON value represents a Product type.
///
/// Matches an explicit `@type: "Product"`, or — when `@type` is absent — a block
/// that carries `offers` but no variants (a single product, not a group). The
/// "no variants" guard ensures a typeless ProductGroup is not misread as a
/// Product (which would consume its aggregate offer instead of a variant).
fn is_product_type(json: &serde_json::Value) -> bool {
    has_schema_type(json, "Product")
        || (lacks_type(json) && json.get("offers").is_some() && !has_variants(json))
}

/// Check if a JSON value represents a ProductGroup type.
///
/// Matches an explicit `@type: "ProductGroup"`, or — when `@type` is absent — a
/// block that carries a non-empty `hasVariant` array.
fn is_product_group_type(json: &serde_json::Value) -> bool {
    has_schema_type(json, "ProductGroup") || (lacks_type(json) && has_variants(json))
}

/// Get availability and price from a ProductGroup by matching variant ID
fn get_availability_and_price_from_product_group(
    product_group: &serde_json::Value,
    variant_id: Option<&str>,
    url: &str,
) -> Option<(String, PriceInfo)> {
    let variants = product_group.get("hasVariant")?.as_array()?;

    let Some(vid) = variant_id else {
        // No variant ID specified, return first variant's availability and price
        return get_first_variant_availability(variants, url);
    };

    // Try to find the matching variant by ID
    let matched = find_variant_by_id(variants, vid, url);
    if matched.is_some() {
        return matched;
    }

    // Fallback: return first variant's availability and price
    get_first_variant_availability(variants, url)
}

/// Find a variant by its ID in the URL query parameters
fn find_variant_by_id(
    variants: &[serde_json::Value],
    vid: &str,
    url: &str,
) -> Option<(String, PriceInfo)> {
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

        if let Some(result) = get_availability_and_price_from_product(variant, url) {
            return Some(result);
        }
    }

    None
}

/// Get the first variant's availability and price
fn get_first_variant_availability(
    variants: &[serde_json::Value],
    url: &str,
) -> Option<(String, PriceInfo)> {
    variants
        .iter()
        .find_map(|v| get_availability_and_price_from_product(v, url))
}

/// Get availability and price from a Product JSON object
fn get_availability_and_price_from_product(
    product: &serde_json::Value,
    url: &str,
) -> Option<(String, PriceInfo)> {
    let offers = product.get("offers")?;

    // Single offer object
    if let Some(avail) = offers.get("availability").and_then(|a| a.as_str()) {
        let price = get_price_from_offer(offers, url);
        return Some((avail.to_string(), price));
    }

    // Array of offers - use first one with availability
    offers.as_array().and_then(|arr| {
        arr.iter().find_map(|offer| {
            let avail = offer.get("availability")?.as_str()?;
            let price = get_price_from_offer(offer, url);
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
        let result = extract_availability_and_price(&json, None, "https://example.com/product");
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
        let result =
            extract_availability_and_price(&json, Some("456"), "https://example.com/product");
        assert!(result.is_some());
        let (avail, _) = result.unwrap();
        assert_eq!(avail, "http://schema.org/InStock");

        // Without variant ID - gets first variant
        let result = extract_availability_and_price(&json, None, "https://example.com/product");
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
        let result = extract_availability_and_price(&json, None, "https://example.com/product");
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
        let result = extract_availability_and_price(&json, None, "https://example.com/product");
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
        let result = extract_availability_and_price(&json, None, "https://example.com/product");
        assert!(result.is_some());
        let (avail, price) = result.unwrap();
        // Should use first offer's availability
        assert_eq!(avail, "http://schema.org/OutOfStock");
        assert_eq!(price.price_minor_units, Some(4999));
    }

    #[test]
    fn test_extract_typeless_product_group_bonds() {
        // bonds.com.au (Magento) emits a ProductGroup with NO `@type`. Detection
        // must fall back to the `hasVariant` structure. The aggregate offer has
        // price 0, so the resolved price comes from the first variant.
        let json = serde_json::json!({
            "sku": "AVMJI_PCL",
            "brand": { "@type": "Brand", "name": "Bonds" },
            "offers": {
                "availability": "https://schema.org/InStock",
                "itemCondition": "https://schema.org/NewCondition",
                "price": 0,
                "priceCurrency": "AUD",
                "offerCount": 2,
                "lowPrice": 30,
                "highPrice": 30
            },
            "productGroupID": "AVMJI_PCL",
            "variesBy": ["https://schema.org/size"],
            "hasVariant": [
                {
                    "sku": "AVMJI_PCL-XXL",
                    "size": "XXL",
                    "brand": { "@type": "Brand" },
                    "offers": {
                        "availability": "https://schema.org/InStock",
                        "itemCondition": "https://schema.org/NewCondition",
                        "price": 30,
                        "priceCurrency": "AUD"
                    }
                },
                {
                    "sku": "AVMJI_PCL-3XL",
                    "size": "3XL",
                    "brand": { "@type": "Brand" },
                    "offers": {
                        "availability": "https://schema.org/InStock",
                        "itemCondition": "https://schema.org/NewCondition",
                        "price": 30,
                        "priceCurrency": "AUD"
                    }
                }
            ]
        });

        let result = extract_availability_and_price(
            &json,
            None,
            "https://www.bonds.com.au/originals-skinny-trackie-avmji-pcl.html",
        );
        assert!(result.is_some());
        let (avail, price) = result.unwrap();
        assert_eq!(avail, "https://schema.org/InStock");
        assert_eq!(price.price_minor_units, Some(3000));
        assert_eq!(price.price_currency, Some("AUD".to_string()));
    }

    #[test]
    fn test_extract_typeless_single_product() {
        // A single product with `offers` but no `@type` and no variants should
        // route through the Product branch.
        let json = serde_json::json!({
            "sku": "SKU123",
            "offers": {
                "availability": "https://schema.org/InStock",
                "price": 49.99,
                "priceCurrency": "AUD"
            }
        });
        let result = extract_availability_and_price(&json, None, "https://store.com.au/products/x");
        assert!(result.is_some());
        let (avail, price) = result.unwrap();
        assert_eq!(avail, "https://schema.org/InStock");
        assert_eq!(price.price_minor_units, Some(4999));
    }

    #[test]
    fn test_typeless_block_without_offers_or_variants_is_ignored() {
        // A typeless block that is neither a product nor a group (e.g. an
        // Organization-like blob) must not be misclassified.
        let json = serde_json::json!({
            "name": "Bonds Australia",
            "url": "https://www.bonds.com.au/"
        });
        let result = extract_availability_and_price(&json, None, "https://www.bonds.com.au/");
        assert!(result.is_none());
    }

    #[test]
    fn test_typed_product_group_with_aggregate_uses_variant_price() {
        // Regression guard: an explicit ProductGroup carrying an aggregate
        // `offers` (price 0) must still resolve to a variant price, not the
        // aggregate. Confirms the structural Product fallback does not hijack
        // typed ProductGroups.
        let json = serde_json::json!({
            "@type": "ProductGroup",
            "offers": {
                "availability": "https://schema.org/InStock",
                "price": 0,
                "priceCurrency": "AUD"
            },
            "hasVariant": [
                {
                    "@type": "Product",
                    "offers": {
                        "availability": "https://schema.org/InStock",
                        "price": 30,
                        "priceCurrency": "AUD"
                    }
                }
            ]
        });
        let result = extract_availability_and_price(&json, None, "https://store.com.au/x");
        assert!(result.is_some());
        let (_, price) = result.unwrap();
        assert_eq!(price.price_minor_units, Some(3000));
    }
}
