//! Uniqlo adapter for checking product availability.
//!
//! Uniqlo product pages are a client-side rendered React SPA: the raw HTML
//! contains no price or stock data, so the Schema.org / dataLayer / Next.js
//! strategies all fail. Price and stock are loaded from Uniqlo's commerce API:
//!
//! ```text
//! GET /{region}/api/commerce/v5/{lang}/products/{code}/price-groups/{pg}/l2s
//!     ?withPrices=true&withStocks=true&httpFailure=true
//! Header: x-fr-clientid: uq.{region}.web-spa
//! ```
//!
//! The approach:
//! 1. Parse region, language, product code, price group, and the
//!    color/size display codes from the product URL
//! 2. Call the commerce API with the required client-id header
//! 3. Resolve the URL's color + size display codes to an `l2Id`
//! 4. Look up that `l2Id` in the response's `prices` and `stocks` maps

use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;
use url::Url;

use crate::entities::availability_check::AvailabilityStatus;
use product_stalker_core::AppError;

use super::price_parser::{parse_price_to_minor_units, PriceInfo};
use super::ScrapingResult;
use super::USER_AGENT;

/// HTTP request timeout for Uniqlo API calls
const TIMEOUT_SECS: u64 = 15;

/// Default language segment when a URL omits it (e.g. `/au/products/...`)
const DEFAULT_LANG: &str = "en";

/// Default price group when a URL omits the trailing segment
const DEFAULT_PRICE_GROUP: &str = "00";

/// Parsed components of a Uniqlo product URL needed to query the commerce API.
struct UniqloContext {
    region: String,
    lang: String,
    product_code: String,
    price_group: String,
    color_display_code: Option<String>,
    size_display_code: Option<String>,
    /// Scheme + host (e.g. `https://www.uniqlo.com`), used to build the API URL
    base_url: String,
}

impl UniqloContext {
    /// Parse a Uniqlo product URL into the components needed for an API call.
    ///
    /// Expects a path shaped like `/{region}/{lang}/products/{code}/{priceGroup}`,
    /// e.g. `/au/en/products/E465185-000/00`. The `lang` and trailing price-group
    /// segments are optional and fall back to sensible defaults.
    fn from_url(url: &str) -> Result<Self, AppError> {
        let parsed =
            Url::parse(url).map_err(|e| AppError::Validation(format!("Invalid URL: {}", e)))?;

        let base_url = base_url(&parsed)
            .ok_or_else(|| AppError::External("Could not parse Uniqlo base URL".to_string()))?;

        let segments: Vec<&str> = parsed
            .path_segments()
            .into_iter()
            .flatten()
            .filter(|s| !s.is_empty())
            .collect();

        let products_idx = segments
            .iter()
            .position(|s| *s == "products")
            .ok_or_else(|| {
                AppError::External("URL does not contain a 'products' path segment".to_string())
            })?;

        let product_code = segments.get(products_idx + 1).ok_or_else(|| {
            AppError::External("URL is missing the product code after '/products/'".to_string())
        })?;

        // `/products/` is always preceded by region (and usually language).
        let region = segments.first().ok_or_else(|| {
            AppError::External("URL is missing the region path segment".to_string())
        })?;
        let lang = if products_idx >= 2 {
            segments[1]
        } else {
            DEFAULT_LANG
        };
        let price_group = segments
            .get(products_idx + 2)
            .copied()
            .unwrap_or(DEFAULT_PRICE_GROUP);

        let color_display_code = query_value(&parsed, "colorDisplayCode");
        let size_display_code = query_value(&parsed, "sizeDisplayCode");

        Ok(Self {
            region: region.to_string(),
            lang: lang.to_string(),
            product_code: product_code.to_string(),
            price_group: price_group.to_string(),
            color_display_code,
            size_display_code,
            base_url,
        })
    }

    /// Build the commerce API URL for this product's stock and price data.
    fn api_url(&self) -> String {
        format!(
            "{}/{}/api/commerce/v5/{}/products/{}/price-groups/{}/l2s?withPrices=true&withStocks=true&httpFailure=true",
            self.base_url, self.region, self.lang, self.product_code, self.price_group
        )
    }

    /// The `x-fr-clientid` header value Uniqlo's SPA sends for this region.
    fn client_id(&self) -> String {
        format!("uq.{}.web-spa", self.region)
    }
}

// --- API response types -----------------------------------------------------

#[derive(Debug, Deserialize)]
struct ApiResponse {
    status: String,
    result: Option<ApiResult>,
}

#[derive(Debug, Deserialize)]
struct ApiResult {
    #[serde(default)]
    l2s: Vec<L2>,
    #[serde(default)]
    prices: HashMap<String, PriceEntry>,
    #[serde(default)]
    stocks: HashMap<String, StockEntry>,
}

#[derive(Debug, Deserialize)]
struct L2 {
    color: DisplayCode,
    size: DisplayCode,
    #[serde(rename = "l2Id")]
    l2_id: String,
}

#[derive(Debug, Deserialize)]
struct DisplayCode {
    #[serde(rename = "displayCode")]
    display_code: String,
}

#[derive(Debug, Deserialize)]
struct PriceEntry {
    base: Option<PriceValue>,
    /// Promotional price; when present it's the price the customer actually pays.
    promo: Option<PriceValue>,
}

#[derive(Debug, Deserialize)]
struct PriceValue {
    currency: Currency,
    /// Kept as a JSON number to preserve the exact representation
    /// (avoids float formatting artifacts when converting to minor units).
    value: serde_json::Number,
}

#[derive(Debug, Deserialize)]
struct Currency {
    code: String,
}

#[derive(Debug, Deserialize)]
struct StockEntry {
    #[serde(rename = "statusCode")]
    status_code: String,
}

// --- Public entry point ------------------------------------------------------

/// Check if a URL is a Uniqlo product page handled by this adapter.
pub fn is_uniqlo_product_url(url: &str) -> bool {
    Url::parse(url)
        .ok()
        .and_then(|parsed| {
            let host = parsed.host_str()?.to_string();
            let is_uniqlo = host == "uniqlo.com" || host.ends_with(".uniqlo.com");
            let has_products = parsed
                .path_segments()
                .into_iter()
                .flatten()
                .any(|s| s == "products");
            Some(is_uniqlo && has_products)
        })
        .unwrap_or(false)
}

/// Check availability and price for a Uniqlo product via the commerce API.
pub async fn check_uniqlo_availability(url: &str) -> Result<ScrapingResult, AppError> {
    let context = UniqloContext::from_url(url)?;
    log::debug!(
        "Uniqlo extraction: region={}, lang={}, code={}, price_group={}, color={:?}, size={:?}",
        context.region,
        context.lang,
        context.product_code,
        context.price_group,
        context.color_display_code,
        context.size_display_code,
    );

    let client = build_http_client()?;
    let body = fetch_api_body(&client, &context.api_url(), &context.client_id()).await?;
    let result = parse_response(&body)?;

    build_result(
        &result,
        context.color_display_code.as_deref(),
        context.size_display_code.as_deref(),
    )
}

// --- Internals ---------------------------------------------------------------

fn build_http_client() -> Result<reqwest::Client, AppError> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| AppError::External(e.to_string()))
}

/// Fetch the raw commerce API response body.
async fn fetch_api_body(
    client: &reqwest::Client,
    api_url: &str,
    client_id: &str,
) -> Result<String, AppError> {
    let response = client
        .get(api_url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json")
        .header("x-fr-clientid", client_id)
        .send()
        .await
        .map_err(|e| AppError::External(e.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::External(format!(
            "Uniqlo API returned HTTP {}",
            response.status()
        )));
    }

    response
        .text()
        .await
        .map_err(|e| AppError::External(e.to_string()))
}

/// Parse and validate the commerce API response.
fn parse_response(body: &str) -> Result<ApiResult, AppError> {
    let response: ApiResponse = serde_json::from_str(body)
        .map_err(|e| AppError::External(format!("Failed to parse Uniqlo API response: {}", e)))?;

    if response.status != "ok" {
        return Err(AppError::External(format!(
            "Uniqlo API returned status '{}'",
            response.status
        )));
    }

    response
        .result
        .ok_or_else(|| AppError::External("Uniqlo API response missing 'result'".to_string()))
}

/// Resolve the target variant and build a `ScrapingResult` from its price/stock.
fn build_result(
    result: &ApiResult,
    color_display_code: Option<&str>,
    size_display_code: Option<&str>,
) -> Result<ScrapingResult, AppError> {
    let l2 = find_target_l2(&result.l2s, color_display_code, size_display_code)?;

    let status = result
        .stocks
        .get(&l2.l2_id)
        .map(|stock| map_stock_status(&stock.status_code))
        .unwrap_or(AvailabilityStatus::Unknown);
    let raw_availability = result
        .stocks
        .get(&l2.l2_id)
        .map(|stock| stock.status_code.clone());

    let price = result
        .prices
        .get(&l2.l2_id)
        .map(extract_price)
        .unwrap_or(PriceInfo {
            price_minor_units: None,
            price_currency: None,
            raw_price: None,
        });

    Ok(ScrapingResult {
        status,
        raw_availability,
        price,
    })
}

/// Find the variant matching the URL's color and size display codes.
///
/// Codes present in the URL are matched exactly; absent codes are treated as
/// wildcards. Falls back to the first variant when neither code is provided.
fn find_target_l2<'a>(
    l2s: &'a [L2],
    color_display_code: Option<&str>,
    size_display_code: Option<&str>,
) -> Result<&'a L2, AppError> {
    if l2s.is_empty() {
        return Err(AppError::External(
            "No variants found in Uniqlo API response".to_string(),
        ));
    }

    let matches = |l2: &L2| {
        color_display_code.is_none_or(|c| l2.color.display_code == c)
            && size_display_code.is_none_or(|s| l2.size.display_code == s)
    };

    l2s.iter().find(|l2| matches(l2)).ok_or_else(|| {
        AppError::External(format!(
            "No Uniqlo variant matched color={:?}, size={:?}",
            color_display_code, size_display_code
        ))
    })
}

/// Map a Uniqlo stock `statusCode` to an availability status.
fn map_stock_status(status_code: &str) -> AvailabilityStatus {
    match status_code.to_uppercase().as_str() {
        "IN_STOCK" | "LOW_STOCK" | "BACK_IN_STOCK" => AvailabilityStatus::InStock,
        "STOCK_OUT" | "SOLD_OUT" | "OUT_OF_STOCK" => AvailabilityStatus::OutOfStock,
        _ => AvailabilityStatus::Unknown,
    }
}

/// Build price info from an API price entry, preferring the promo price.
fn extract_price(entry: &PriceEntry) -> PriceInfo {
    let Some(value) = entry.promo.as_ref().or(entry.base.as_ref()) else {
        return PriceInfo {
            price_minor_units: None,
            price_currency: None,
            raw_price: None,
        };
    };

    let raw_price = value.value.to_string();
    let currency = value.currency.code.clone();
    let price_minor_units = parse_price_to_minor_units(&raw_price, Some(&currency));

    PriceInfo {
        price_minor_units,
        price_currency: Some(currency),
        raw_price: Some(raw_price),
    }
}

/// Get the scheme + host portion of a parsed URL (no path).
fn base_url(parsed: &Url) -> Option<String> {
    let host = parsed.host_str()?;
    match parsed.port() {
        Some(port) => Some(format!("{}://{}:{}", parsed.scheme(), host, port)),
        None => Some(format!("{}://{}", parsed.scheme(), host)),
    }
}

/// Get a query parameter value from a parsed URL.
fn query_value(parsed: &Url, key: &str) -> Option<String> {
    parsed
        .query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RESPONSE: &str = r#"{
        "status": "ok",
        "result": {
            "l2s": [
                {"color": {"displayCode": "07"}, "size": {"displayCode": "005"}, "l2Id": "111"},
                {"color": {"displayCode": "07"}, "size": {"displayCode": "006"}, "l2Id": "222"},
                {"color": {"displayCode": "08"}, "size": {"displayCode": "006"}, "l2Id": "333"}
            ],
            "prices": {
                "111": {"base": {"currency": {"code": "AUD", "symbol": "$"}, "value": 24.9}, "promo": null},
                "222": {"base": {"currency": {"code": "AUD", "symbol": "$"}, "value": 24.9}, "promo": null},
                "333": {"base": {"currency": {"code": "AUD", "symbol": "$"}, "value": 29.9}, "promo": {"currency": {"code": "AUD", "symbol": "$"}, "value": 19.9}}
            },
            "stocks": {
                "111": {"statusCode": "IN_STOCK"},
                "222": {"statusCode": "IN_STOCK"},
                "333": {"statusCode": "STOCK_OUT"}
            }
        }
    }"#;

    #[test]
    fn test_is_uniqlo_product_url() {
        assert!(is_uniqlo_product_url(
            "https://www.uniqlo.com/au/en/products/E465185-000/00?colorDisplayCode=07"
        ));
        assert!(is_uniqlo_product_url(
            "https://www.uniqlo.com/sg/en/products/E465185-000/00"
        ));
        // Non-product Uniqlo page
        assert!(!is_uniqlo_product_url("https://www.uniqlo.com/au/en/women"));
        // Look-alike host should not match
        assert!(!is_uniqlo_product_url(
            "https://uniqlo.com.evil.com/au/products/test"
        ));
        // Different store
        assert!(!is_uniqlo_product_url("https://example.com/products/test"));
    }

    #[test]
    fn test_context_from_url_full_path() {
        let ctx = UniqloContext::from_url(
            "https://www.uniqlo.com/au/en/products/E465185-000/00?colorDisplayCode=07&sizeDisplayCode=006",
        )
        .unwrap();
        assert_eq!(ctx.region, "au");
        assert_eq!(ctx.lang, "en");
        assert_eq!(ctx.product_code, "E465185-000");
        assert_eq!(ctx.price_group, "00");
        assert_eq!(ctx.color_display_code.as_deref(), Some("07"));
        assert_eq!(ctx.size_display_code.as_deref(), Some("006"));
        assert_eq!(
            ctx.api_url(),
            "https://www.uniqlo.com/au/api/commerce/v5/en/products/E465185-000/price-groups/00/l2s?withPrices=true&withStocks=true&httpFailure=true"
        );
        assert_eq!(ctx.client_id(), "uq.au.web-spa");
    }

    #[test]
    fn test_context_from_url_defaults() {
        // No language segment and no price group → defaults applied
        let ctx =
            UniqloContext::from_url("https://www.uniqlo.com/sg/products/E123456-000").unwrap();
        assert_eq!(ctx.region, "sg");
        assert_eq!(ctx.lang, "en");
        assert_eq!(ctx.price_group, "00");
        assert_eq!(ctx.client_id(), "uq.sg.web-spa");
    }

    #[test]
    fn test_context_from_url_missing_product_code() {
        let result = UniqloContext::from_url("https://www.uniqlo.com/au/en/women");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_result_matches_variant() {
        let result = parse_response(SAMPLE_RESPONSE).unwrap();
        let scraping = build_result(&result, Some("07"), Some("006")).unwrap();
        assert_eq!(scraping.status, AvailabilityStatus::InStock);
        assert_eq!(scraping.raw_availability, Some("IN_STOCK".to_string()));
        assert_eq!(scraping.price.price_minor_units, Some(2490));
        assert_eq!(scraping.price.price_currency, Some("AUD".to_string()));
        assert_eq!(scraping.price.raw_price, Some("24.9".to_string()));
    }

    #[test]
    fn test_build_result_out_of_stock_uses_promo_price() {
        let result = parse_response(SAMPLE_RESPONSE).unwrap();
        let scraping = build_result(&result, Some("08"), Some("006")).unwrap();
        assert_eq!(scraping.status, AvailabilityStatus::OutOfStock);
        // Promo price (19.9) preferred over base (29.9)
        assert_eq!(scraping.price.price_minor_units, Some(1990));
        assert_eq!(scraping.price.raw_price, Some("19.9".to_string()));
    }

    #[test]
    fn test_build_result_no_match() {
        let result = parse_response(SAMPLE_RESPONSE).unwrap();
        let scraping = build_result(&result, Some("99"), Some("999"));
        assert!(scraping.is_err());
    }

    #[test]
    fn test_build_result_first_variant_when_no_codes() {
        let result = parse_response(SAMPLE_RESPONSE).unwrap();
        let scraping = build_result(&result, None, None).unwrap();
        // Falls back to first variant (l2Id 111)
        assert_eq!(scraping.price.price_minor_units, Some(2490));
        assert_eq!(scraping.status, AvailabilityStatus::InStock);
    }

    #[test]
    fn test_parse_response_rejects_nok_status() {
        let body = r#"{"status": "nok", "error": {"code": 0}}"#;
        assert!(parse_response(body).is_err());
    }

    #[test]
    fn test_map_stock_status() {
        assert_eq!(map_stock_status("IN_STOCK"), AvailabilityStatus::InStock);
        assert_eq!(map_stock_status("LOW_STOCK"), AvailabilityStatus::InStock);
        assert_eq!(
            map_stock_status("STOCK_OUT"),
            AvailabilityStatus::OutOfStock
        );
        assert_eq!(map_stock_status("SOLD_OUT"), AvailabilityStatus::OutOfStock);
        assert_eq!(map_stock_status("MYSTERY"), AvailabilityStatus::Unknown);
    }
}
