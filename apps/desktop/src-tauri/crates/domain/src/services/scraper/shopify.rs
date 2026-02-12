//! Shopify adapter for checking product availability.
//!
//! Many Shopify stores don't include Schema.org JSON-LD data or don't expose
//! availability in their product.json API. This adapter uses the cart API
//! to determine availability by attempting to add the product to the cart.
//!
//! The approach:
//! 1. Extract variant ID from URL (required for Shopify products)
//! 2. Fetch product.json for price and product details
//! 3. Use cart/add.js API to check availability

use std::time::Duration;

use serde::Deserialize;
use url::Url;

use crate::entities::availability_check::AvailabilityStatus;
use product_stalker_core::AppError;

use super::price_parser::{parse_price_to_minor_units, PriceInfo};
use super::ScrapingResult;

/// HTTP request timeout for Shopify API calls
const TIMEOUT_SECS: u64 = 15;

use super::USER_AGENT;

/// Error phrases from the Shopify cart API that indicate a product is out of stock
const CART_ERROR_OUT_OF_STOCK_PHRASES: &[&str] = &[
    "sold out",
    "not available",
    "out of stock",
    "no longer available",
    "all items are out of stock",
    "insufficient inventory",
];

/// Stores that use /products/ URL pattern but are NOT Shopify stores
/// These stores have their own specialized adapters
const NON_SHOPIFY_STORES: &[&str] = &["chemistwarehouse"];

/// Raw availability value constants for consistent formatting
/// Format: "source:status" or "source:status:details"
const RAW_AVAILABILITY_PRODUCT_JSON_AVAILABLE: &str = "product_json:available";
const RAW_AVAILABILITY_PRODUCT_JSON_UNAVAILABLE: &str = "product_json:unavailable";
const RAW_AVAILABILITY_CART_API_IN_STOCK: &str = "cart_api:in_stock";
/// Base value for out-of-stock; error message is appended after a colon
const RAW_AVAILABILITY_CART_API_OUT_OF_STOCK: &str = "cart_api:out_of_stock";

/// Domain suffix to currency code mappings
/// Within each inner slice, more specific suffixes (e.g., ".com.au") must come before
/// generic ones (e.g., ".au") so that `ends_with` matches the longest suffix first.
const DOMAIN_CURRENCY_MAP: &[(&[&str], &str)] = &[
    (&[".com.au", ".au"], "AUD"),
    (&[".co.uk", ".uk"], "GBP"),
    (&[".ca"], "CAD"),
    (&[".co.nz", ".nz"], "NZD"),
    (&[".eu"], "EUR"),
    (&[".com", ".us"], "USD"),
];

/// Shopify product.json response structure
#[derive(Debug, Deserialize)]
struct ShopifyProductResponse {
    product: ShopifyProduct,
}

#[derive(Debug, Deserialize)]
struct ShopifyProduct {
    #[serde(default)]
    variants: Vec<ShopifyVariant>,
}

#[derive(Debug, Deserialize)]
struct ShopifyVariant {
    id: i64,
    #[serde(default)]
    price: String,
    #[serde(default)]
    available: Option<bool>,
    #[serde(default)]
    price_currency: Option<String>,
}

/// Shopify cart error response - product is out of stock or unavailable
#[derive(Debug, Deserialize)]
struct ShopifyCartError {
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

/// Context for a Shopify product URL containing parsed components
struct ShopifyContext {
    base_url: String,
    handle: String,
    variant_id: Option<i64>,
    url: String,
}

impl ShopifyContext {
    /// Parse a Shopify product URL into its components
    fn from_url(url: &str) -> Result<Self, AppError> {
        let base_url = get_base_url(url)
            .ok_or_else(|| AppError::External("Could not parse base URL".to_string()))?;

        let handle = extract_product_handle(url).ok_or_else(|| {
            AppError::External("Could not extract product handle from URL".to_string())
        })?;

        let variant_id = extract_variant_id(url);

        log::debug!(
            "Shopify extraction: base={}, handle={}, variant_id={:?}",
            base_url,
            handle,
            variant_id
        );

        Ok(Self {
            base_url,
            handle,
            variant_id,
            url: url.to_string(),
        })
    }

    /// Get the product.json URL for this context
    fn product_json_url(&self) -> String {
        format!("{}/products/{}.json", self.base_url, self.handle)
    }
}

/// Check if a URL has an exact "products" path segment (common in Shopify URLs)
fn has_products_path_segment(url: &str) -> bool {
    Url::parse(url)
        .ok()
        .map(|parsed| {
            parsed
                .path_segments()
                .into_iter()
                .flatten()
                .any(|s| s == "products")
        })
        .unwrap_or(false)
}

/// Check if a URL belongs to a known non-Shopify store that uses /products/ pattern
fn is_known_non_shopify_store(url: &str) -> bool {
    NON_SHOPIFY_STORES.iter().any(|store| url.contains(store))
}

/// Check if a URL is potentially a Shopify product page
///
/// Note: This checks URL patterns only. The URL must also pass the `is_shopify_store`
/// HTML check to confirm it's actually a Shopify store, since some non-Shopify stores
/// (like Chemist Warehouse) also use the /products/ URL pattern.
///
/// We detect potential Shopify stores by:
/// 1. URL contains `/products/` path segment (standard Shopify product URL pattern)
/// 2. URL is NOT from a known non-Shopify store (these have dedicated adapters)
pub fn is_potential_shopify_product_url(url: &str) -> bool {
    has_products_path_segment(url) && !is_known_non_shopify_store(url)
}

/// Check if HTML contains Shopify-specific markers
pub fn is_shopify_store(html: &str) -> bool {
    html.contains("Shopify.shop")
        || html.contains("cdn.shopify.com")
        || html.contains("shopify-section")
}

/// Extract the product handle from a Shopify URL
/// e.g., https://store.com/products/my-product -> "my-product"
fn extract_product_handle(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let path = parsed.path();

    // Find /products/ segment and extract the handle
    let parts: Vec<&str> = path.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "products" && i + 1 < parts.len() {
            let handle = parts[i + 1];
            // Handle might have query params stripped already, but make sure
            return Some(handle.split('?').next().unwrap_or(handle).to_string());
        }
    }
    None
}

/// Extract variant ID from URL query parameter
pub fn extract_variant_id(url: &str) -> Option<i64> {
    Url::parse(url).ok().and_then(|parsed| {
        parsed
            .query_pairs()
            .find(|(key, _)| key == "variant")
            .and_then(|(_, value)| value.parse().ok())
    })
}

/// Get the base URL (scheme + host + port) from a full URL
///
/// Includes the port if present (e.g., `http://localhost:3000` preserves `:3000`).
fn get_base_url(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let host = parsed.host_str()?;
    match parsed.port() {
        Some(port) => Some(format!("{}://{}:{}", parsed.scheme(), host, port)),
        None => Some(format!("{}://{}", parsed.scheme(), host)),
    }
}

/// Build a configured HTTP client for Shopify API requests
fn build_http_client() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| AppError::External(e.to_string()))
}

/// Infer currency from a store's domain TLD
///
/// Returns a currency code based on the domain's top-level domain.
/// Returns None if the domain doesn't match a known pattern.
fn infer_currency_from_domain(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let host = parsed.host_str()?;

    DOMAIN_CURRENCY_MAP
        .iter()
        .find(|(suffixes, _)| suffixes.iter().any(|s| host.ends_with(s)))
        .map(|(_, currency)| (*currency).to_string())
}

/// Build a ScrapingResult from product.json availability data
fn build_product_json_result(
    available: bool,
    variant: &ShopifyVariant,
    url: &str,
) -> ScrapingResult {
    let (status, raw_availability) = if available {
        (
            AvailabilityStatus::InStock,
            RAW_AVAILABILITY_PRODUCT_JSON_AVAILABLE,
        )
    } else {
        (
            AvailabilityStatus::OutOfStock,
            RAW_AVAILABILITY_PRODUCT_JSON_UNAVAILABLE,
        )
    };

    ScrapingResult {
        status,
        raw_availability: Some(raw_availability.to_string()),
        price: extract_price_from_variant(variant, url),
    }
}

/// Check availability for a Shopify product
///
/// This is an async function that makes HTTP requests to:
/// 1. product.json - to get price and variant info
/// 2. cart/add.js - to verify availability
pub async fn check_shopify_availability(url: &str, html: &str) -> Result<ScrapingResult, AppError> {
    if !is_shopify_store(html) {
        return Err(AppError::External("Not a Shopify store".to_string()));
    }

    let client = build_http_client()?;
    let context = ShopifyContext::from_url(url)?;
    let product = fetch_product_json(&client, &context.product_json_url()).await?;
    let target_variant = find_target_variant(&product.variants, context.variant_id)?;

    // Use product.json availability if present, otherwise fall back to cart API
    if let Some(available) = target_variant.available {
        log::debug!("Shopify product.json includes availability: {}", available);
        return Ok(build_product_json_result(
            available,
            target_variant,
            &context.url,
        ));
    }

    log::debug!(
        "Shopify product.json lacks availability, using cart API for variant {}",
        target_variant.id
    );
    let cart_result =
        check_cart_availability(&client, &context.base_url, target_variant.id).await?;

    Ok(ScrapingResult {
        status: cart_result.status,
        raw_availability: Some(cart_result.raw_availability),
        price: extract_price_from_variant(target_variant, &context.url),
    })
}

/// Fetch and parse product.json from Shopify store
async fn fetch_product_json(
    client: &reqwest::Client,
    url: &str,
) -> Result<ShopifyProduct, AppError> {
    let response = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| AppError::External(e.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::External(format!(
            "Failed to fetch product.json: HTTP {}",
            response.status()
        )));
    }

    let product_response: ShopifyProductResponse = response
        .json()
        .await
        .map_err(|e| AppError::External(format!("Failed to parse product.json: {}", e)))?;

    Ok(product_response.product)
}

/// Find the target variant by ID, or return the first variant if no ID specified
fn find_target_variant(
    variants: &[ShopifyVariant],
    variant_id: Option<i64>,
) -> Result<&ShopifyVariant, AppError> {
    if variants.is_empty() {
        return Err(AppError::External(
            "No variants found in product.json".to_string(),
        ));
    }

    if let Some(vid) = variant_id {
        variants.iter().find(|v| v.id == vid).ok_or_else(|| {
            AppError::External(format!(
                "Variant {} not found. Available: {:?}",
                vid,
                variants.iter().map(|v| v.id).collect::<Vec<_>>()
            ))
        })
    } else {
        // Return first variant
        Ok(&variants[0])
    }
}

/// Cart availability check result
struct CartAvailabilityResult {
    status: AvailabilityStatus,
    raw_availability: String,
}

/// Check availability using the Shopify cart/add.js API
///
/// If we can successfully add to cart, the product is in stock.
/// If we get an error, check the error message to determine status.
async fn check_cart_availability(
    client: &reqwest::Client,
    base_url: &str,
    variant_id: i64,
) -> Result<CartAvailabilityResult, AppError> {
    let response = send_cart_add_request(client, base_url, variant_id).await?;
    let status_code = response.status();
    let body = response.text().await.unwrap_or_default();

    log::debug!(
        "Shopify cart API response: status={}, body_preview={}",
        status_code,
        &body[..body.len().min(200)]
    );

    if status_code.is_success() {
        // Successfully added to cart - product is in stock
        // Try to clear the cart item we just added
        clear_cart(client, base_url).await;

        return Ok(CartAvailabilityResult {
            status: AvailabilityStatus::InStock,
            raw_availability: RAW_AVAILABILITY_CART_API_IN_STOCK.to_string(),
        });
    }

    parse_cart_error_body(&body, status_code)
}

/// Send a request to add an item to the Shopify cart
async fn send_cart_add_request(
    client: &reqwest::Client,
    base_url: &str,
    variant_id: i64,
) -> Result<reqwest::Response, AppError> {
    let cart_url = format!("{}/cart/add.js", base_url);
    let payload = serde_json::json!({
        "items": [{
            "id": variant_id,
            "quantity": 1
        }]
    });

    client
        .post(&cart_url)
        .header("User-Agent", USER_AGENT)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::External(e.to_string()))
}

/// Parse an error body from the cart API to determine if out of stock
fn parse_cart_error_body(
    body: &str,
    status_code: reqwest::StatusCode,
) -> Result<CartAvailabilityResult, AppError> {
    if let Ok(error) = serde_json::from_str::<ShopifyCartError>(body) {
        let message = error
            .message
            .or(error.description)
            .unwrap_or_default()
            .to_lowercase();

        if is_cart_error_out_of_stock(&message) {
            return Ok(CartAvailabilityResult {
                status: AvailabilityStatus::OutOfStock,
                raw_availability: format!("{}:{}", RAW_AVAILABILITY_CART_API_OUT_OF_STOCK, message),
            });
        }
    }

    // Unknown error - could be rate limiting, etc.
    Err(AppError::External(format!(
        "Cart API returned unexpected response: HTTP {} - {}",
        status_code,
        &body[..body.len().min(100)]
    )))
}

/// Check if a cart API error message indicates the product is out of stock
fn is_cart_error_out_of_stock(message: &str) -> bool {
    CART_ERROR_OUT_OF_STOCK_PHRASES
        .iter()
        .any(|phrase| message.contains(phrase))
}

/// Clear the cart after checking availability
async fn clear_cart(client: &reqwest::Client, base_url: &str) {
    let clear_url = format!("{}/cart/clear.js", base_url);

    if let Err(e) = client
        .post(&clear_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
    {
        log::debug!("Failed to clear Shopify cart (non-critical): {}", e);
    }
}

/// Extract price info from a Shopify variant
///
/// Currency is determined in order of precedence:
/// 1. Currency from the variant data (if available)
/// 2. Inferred from the store's domain TLD
/// 3. None if neither is available
fn extract_price_from_variant(variant: &ShopifyVariant, url: &str) -> PriceInfo {
    let raw_price = if variant.price.is_empty() {
        None
    } else {
        Some(variant.price.clone())
    };

    let price_currency = variant.price_currency.clone().or_else(|| {
        if raw_price.is_some() {
            infer_currency_from_domain(url)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_potential_shopify_product_url() {
        // Standard Shopify URLs
        assert!(is_potential_shopify_product_url(
            "https://rushfaster.com.au/products/able-carry-thirteen-daybag-x-pac"
        ));
        assert!(is_potential_shopify_product_url(
            "https://store.myshopify.com/products/test-product?variant=123"
        ));
        assert!(is_potential_shopify_product_url(
            "https://example.com/collections/all/products/my-product"
        ));

        // Not Shopify URLs (missing /products/ path)
        assert!(!is_potential_shopify_product_url(
            "https://amazon.com/dp/B12345"
        ));
        assert!(!is_potential_shopify_product_url(
            "https://example.com/product/123"
        ));

        // Known non-Shopify stores that use /products/ pattern
        assert!(!is_potential_shopify_product_url(
            "https://chemistwarehouse.com.au/products/test"
        ));
    }

    #[test]
    fn test_has_products_path_segment() {
        assert!(has_products_path_segment(
            "https://example.com/products/test"
        ));
        assert!(has_products_path_segment(
            "https://example.com/collections/all/products/test"
        ));
        assert!(!has_products_path_segment(
            "https://example.com/product/test"
        ));
        assert!(!has_products_path_segment("https://example.com/items/test"));
        // Should not match "byproducts" or other words containing "products"
        assert!(!has_products_path_segment(
            "https://example.com/byproducts/test"
        ));
    }

    #[test]
    fn test_is_known_non_shopify_store() {
        assert!(is_known_non_shopify_store(
            "https://chemistwarehouse.com.au/products/test"
        ));
        assert!(is_known_non_shopify_store(
            "https://www.chemistwarehouse.com.au/buy/123"
        ));
        assert!(!is_known_non_shopify_store(
            "https://rushfaster.com.au/products/test"
        ));
        assert!(!is_known_non_shopify_store(
            "https://example.com/products/test"
        ));
    }

    #[test]
    fn test_is_shopify_store() {
        assert!(is_shopify_store(
            r#"<html><script>Shopify.shop = "mystore"</script></html>"#
        ));
        assert!(is_shopify_store(
            r#"<html><link href="https://cdn.shopify.com/s/files/1/123/style.css"></html>"#
        ));
        assert!(is_shopify_store(
            r#"<html><div class="shopify-section"></div></html>"#
        ));

        assert!(!is_shopify_store("<html><body>Normal page</body></html>"));
    }

    #[test]
    fn test_extract_product_handle() {
        assert_eq!(
            extract_product_handle("https://store.com/products/my-product"),
            Some("my-product".to_string())
        );
        assert_eq!(
            extract_product_handle("https://store.com/products/my-product?variant=123"),
            Some("my-product".to_string())
        );
        assert_eq!(
            extract_product_handle("https://store.com/collections/all/products/test"),
            Some("test".to_string())
        );
        assert_eq!(extract_product_handle("https://store.com/cart"), None);
    }

    #[test]
    fn test_extract_variant_id() {
        assert_eq!(
            extract_variant_id("https://store.com/products/test?variant=45237546025126"),
            Some(45237546025126)
        );
        assert_eq!(extract_variant_id("https://store.com/products/test"), None);
        assert_eq!(
            extract_variant_id("https://store.com/products/test?color=blue"),
            None
        );
    }

    #[test]
    fn test_get_base_url() {
        assert_eq!(
            get_base_url("https://rushfaster.com.au/products/test?variant=123"),
            Some("https://rushfaster.com.au".to_string())
        );
        // Port should be preserved
        assert_eq!(
            get_base_url("http://localhost:3000/products/test"),
            Some("http://localhost:3000".to_string())
        );
        // Standard ports (80/443) are not included by url crate's port() method
        assert_eq!(
            get_base_url("https://example.com:443/products/test"),
            Some("https://example.com".to_string())
        );
    }

    #[test]
    fn test_find_target_variant() {
        let variants = vec![
            ShopifyVariant {
                id: 100,
                price: "10.00".to_string(),
                available: Some(true),
                price_currency: None,
            },
            ShopifyVariant {
                id: 200,
                price: "20.00".to_string(),
                available: Some(false),
                price_currency: None,
            },
        ];

        // With specific variant ID
        let result = find_target_variant(&variants, Some(200)).unwrap();
        assert_eq!(result.id, 200);

        // Without variant ID - returns first
        let result = find_target_variant(&variants, None).unwrap();
        assert_eq!(result.id, 100);

        // Non-existent variant
        let result = find_target_variant(&variants, Some(999));
        assert!(result.is_err());

        // Empty variants
        let result = find_target_variant(&[], None);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_price_from_variant() {
        let variant = ShopifyVariant {
            id: 123,
            price: "330.00".to_string(),
            available: Some(true),
            price_currency: Some("AUD".to_string()),
        };

        let price = extract_price_from_variant(&variant, "https://store.com.au/products/test");
        assert_eq!(price.price_minor_units, Some(33000));
        assert_eq!(price.price_currency, Some("AUD".to_string()));
        assert_eq!(price.raw_price, Some("330.00".to_string()));
    }

    #[test]
    fn test_extract_price_from_variant_empty_price() {
        let variant = ShopifyVariant {
            id: 123,
            price: "".to_string(),
            available: None,
            price_currency: None,
        };

        let price = extract_price_from_variant(&variant, "https://store.com/products/test");
        assert_eq!(price.price_minor_units, None);
        assert_eq!(price.price_currency, None);
        assert_eq!(price.raw_price, None);
    }

    #[test]
    fn test_extract_price_infers_currency_from_domain() {
        let variant = ShopifyVariant {
            id: 123,
            price: "50.00".to_string(),
            available: Some(true),
            price_currency: None, // No currency in variant data
        };

        // Australian domain
        let price = extract_price_from_variant(&variant, "https://store.com.au/products/test");
        assert_eq!(price.price_currency, Some("AUD".to_string()));

        // UK domain
        let price = extract_price_from_variant(&variant, "https://store.co.uk/products/test");
        assert_eq!(price.price_currency, Some("GBP".to_string()));

        // US domain
        let price = extract_price_from_variant(&variant, "https://store.com/products/test");
        assert_eq!(price.price_currency, Some("USD".to_string()));

        // Canadian domain
        let price = extract_price_from_variant(&variant, "https://store.ca/products/test");
        assert_eq!(price.price_currency, Some("CAD".to_string()));

        // NZ domain
        let price = extract_price_from_variant(&variant, "https://store.co.nz/products/test");
        assert_eq!(price.price_currency, Some("NZD".to_string()));
    }

    #[test]
    fn test_extract_price_variant_currency_takes_precedence() {
        let variant = ShopifyVariant {
            id: 123,
            price: "50.00".to_string(),
            available: Some(true),
            price_currency: Some("EUR".to_string()), // Explicit currency
        };

        // Even though domain is .au, variant currency should take precedence
        let price = extract_price_from_variant(&variant, "https://store.com.au/products/test");
        assert_eq!(price.price_currency, Some("EUR".to_string()));
    }

    #[test]
    fn test_infer_currency_from_domain() {
        assert_eq!(
            infer_currency_from_domain("https://store.com.au/products/test"),
            Some("AUD".to_string())
        );
        assert_eq!(
            infer_currency_from_domain("https://store.co.uk/products/test"),
            Some("GBP".to_string())
        );
        assert_eq!(
            infer_currency_from_domain("https://store.com/products/test"),
            Some("USD".to_string())
        );
        assert_eq!(
            infer_currency_from_domain("https://store.ca/products/test"),
            Some("CAD".to_string())
        );
        assert_eq!(
            infer_currency_from_domain("https://store.co.nz/products/test"),
            Some("NZD".to_string())
        );
        assert_eq!(
            infer_currency_from_domain("https://store.eu/products/test"),
            Some("EUR".to_string())
        );
        // Unknown TLD
        assert_eq!(
            infer_currency_from_domain("https://store.xyz/products/test"),
            None
        );
    }

    #[test]
    fn test_is_cart_error_out_of_stock() {
        assert!(is_cart_error_out_of_stock("this product is sold out"));
        assert!(is_cart_error_out_of_stock("item not available"));
        assert!(is_cart_error_out_of_stock("out of stock"));
        assert!(is_cart_error_out_of_stock("no longer available"));
        assert!(is_cart_error_out_of_stock("insufficient inventory"));
        assert!(is_cart_error_out_of_stock("all items are out of stock"));

        assert!(!is_cart_error_out_of_stock("added to cart"));
        assert!(!is_cart_error_out_of_stock("success"));
        // "inventory" alone should not match - too generic
        assert!(!is_cart_error_out_of_stock("inventory updated"));
    }
}
