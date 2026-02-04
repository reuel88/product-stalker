use std::time::Duration;

use scraper::{Html, Selector};
use url::Url;

use super::HeadlessService;
use crate::entities::availability_check::AvailabilityStatus;
use crate::error::AppError;

/// Result of a scraping operation
#[derive(Debug, Clone)]
pub struct ScrapingResult {
    pub status: AvailabilityStatus,
    pub raw_availability: Option<String>,
}

/// Service for scraping product availability from web pages
pub struct ScraperService;

impl ScraperService {
    /// HTTP request timeout
    const TIMEOUT_SECS: u64 = 30;

    /// User-Agent header to use for requests
    const USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    /// Check availability by fetching a URL and parsing Schema.org data
    ///
    /// Uses HTTP as the fast path. Falls back to headless browser if bot
    /// protection (Cloudflare, etc.) is detected and headless is enabled.
    #[allow(dead_code)]
    pub async fn check_availability(url: &str) -> Result<ScrapingResult, AppError> {
        Self::check_availability_with_headless(url, true).await
    }

    /// Check availability with control over headless fallback
    pub async fn check_availability_with_headless(
        url: &str,
        enable_headless: bool,
    ) -> Result<ScrapingResult, AppError> {
        let parsed =
            Url::parse(url).map_err(|e| AppError::Validation(format!("Invalid URL: {}", e)))?;

        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(AppError::Validation(format!(
                "Unsupported URL scheme '{}'. Only http and https are allowed.",
                scheme
            )));
        }

        // Try HTTP first (fast path)
        match Self::fetch_page(url).await {
            Ok(html) => {
                log::debug!("HTTP fetch succeeded for {}, checking for challenge", url);
                // Check for challenge in successful response (some return 200 with challenge)
                if Self::is_cloudflare_challenge(200, &html) {
                    log::info!("Detected bot protection challenge for {}", url);
                    if enable_headless {
                        log::info!("Attempting headless fallback for {}", url);
                        return Self::try_headless_fallback(url).await;
                    }
                    return Err(AppError::BotProtection(
                        "This site has bot protection. Enable headless browser in settings to check this site.".to_string()
                    ));
                }
                Self::parse_schema_org_with_url(&html, url)
            }
            Err(AppError::Scraping(msg)) if msg.contains("403") || msg.contains("503") => {
                // Likely bot protection - try headless
                log::info!(
                    "HTTP request blocked ({}) for {}, trying headless",
                    msg,
                    url
                );
                if enable_headless {
                    Self::try_headless_fallback(url).await
                } else {
                    Err(AppError::BotProtection(
                        "This site has bot protection. Enable headless browser in settings to check this site.".to_string()
                    ))
                }
            }
            Err(e) => {
                log::error!("HTTP fetch failed for {}: {}", url, e);
                Err(e)
            }
        }
    }

    /// Check if the response is a Cloudflare challenge page or other bot protection
    fn is_cloudflare_challenge(status: u16, body: &str) -> bool {
        // Check for common Cloudflare challenge indicators
        let is_challenge_status = status == 403 || status == 503;
        let body_lower = body.to_lowercase();

        // If the page has product data (JSON-LD), it's not a challenge page
        let has_product_data = body_lower.contains("application/ld+json");

        // Cloudflare-specific indicators (strong signals)
        let cloudflare_indicators = body_lower.contains("just a moment...")
            || body_lower.contains("cf-browser-verification")
            || body_lower.contains("_cf_chl_opt")
            || body_lower.contains("checking your browser")
            || body_lower.contains("ray id:")
            || body_lower.contains("cf-challenge")
            || body_lower.contains("__cf_bm")
            || body_lower.contains("cloudflare");

        // Explicit bot protection indicators (strong signals)
        let explicit_bot_protection = body_lower.contains("bot detected")
            || body_lower.contains("please verify you are a human")
            || body_lower.contains("enable javascript and cookies")
            || body_lower.contains("pardon our interruption");

        // Check for minimal HTML (likely a challenge page, not real content)
        let is_minimal_page =
            !has_product_data && (body.len() < 5000 || !body_lower.contains("<body"));

        // A 403/503 with strong challenge indicators is a challenge
        // Note: "access denied" alone is too broad - legitimate 403s often have this
        if is_challenge_status {
            // If we have product data, it's not a challenge
            if has_product_data {
                return false;
            }
            cloudflare_indicators || explicit_bot_protection || is_minimal_page
        } else {
            // For 200 responses, require strong indicators
            cloudflare_indicators || explicit_bot_protection
        }
    }

    /// Try to fetch page using headless browser
    ///
    /// Runs the blocking headless browser operations on a dedicated thread pool
    /// to avoid blocking the async runtime.
    async fn try_headless_fallback(url: &str) -> Result<ScrapingResult, AppError> {
        let url_owned = url.to_string();
        let url_for_parse = url.to_string();
        let result = tokio::task::spawn_blocking(move || {
            let mut headless = HeadlessService::new();
            headless.fetch_page(&url_owned)
        })
        .await
        .map_err(|e| AppError::Internal(format!("Headless task failed: {}", e)))?;

        match result {
            Ok(html) => Self::parse_schema_org_with_url(&html, &url_for_parse),
            Err(e) => Err(e),
        }
    }

    /// Fetch a page's HTML content using HTTP
    async fn fetch_page(url: &str) -> Result<String, AppError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(Self::TIMEOUT_SECS))
            .build()?;

        let response = client
            .get(url)
            .header("User-Agent", Self::USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Sec-Ch-Ua", "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"")
            .header("Sec-Ch-Ua-Mobile", "?0")
            .header("Sec-Ch-Ua-Platform", "\"Windows\"")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-User", "?1")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Scraping(format!(
                "HTTP {} for URL: {}",
                response.status(),
                url
            )));
        }

        let html = response.text().await?;
        Ok(html)
    }

    /// Parse Schema.org JSON-LD data from HTML to extract availability
    /// Uses the URL to match specific product variants
    pub fn parse_schema_org_with_url(html: &str, url: &str) -> Result<ScrapingResult, AppError> {
        let variant_id = Self::extract_variant_id(url);
        Self::parse_schema_org_internal(html, variant_id.as_deref())
    }

    /// Internal parsing with optional variant ID
    fn parse_schema_org_internal(
        html: &str,
        variant_id: Option<&str>,
    ) -> Result<ScrapingResult, AppError> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("script[type=\"application/ld+json\"]")
            .map_err(|e| AppError::Scraping(format!("Invalid selector: {:?}", e)))?;

        for element in document.select(&selector) {
            let json_text = element.inner_html();

            // Try to parse as JSON
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_text) {
                // Check if it's a Product, ProductGroup, or has @graph with products
                if let Some(availability) = Self::extract_availability(&json, variant_id) {
                    return Ok(ScrapingResult {
                        status: AvailabilityStatus::from_schema_org(&availability),
                        raw_availability: Some(availability),
                    });
                }
            }
        }

        // No availability found in any JSON-LD
        Err(AppError::Scraping(
            "No availability information found in Schema.org data".to_string(),
        ))
    }

    /// Extract variant ID from URL query parameters
    fn extract_variant_id(url: &str) -> Option<String> {
        Url::parse(url).ok().and_then(|parsed| {
            parsed
                .query_pairs()
                .find(|(key, _)| key == "variant")
                .map(|(_, value)| value.to_string())
        })
    }

    /// Extract availability from a JSON-LD value, trying multiple known structures
    fn extract_availability(json: &serde_json::Value, variant_id: Option<&str>) -> Option<String> {
        Self::try_extract_from_product(json)
            .or_else(|| Self::try_extract_from_product_group(json, variant_id))
            .or_else(|| Self::try_extract_from_graph(json, variant_id))
            .or_else(|| Self::try_extract_from_array(json, variant_id))
    }

    /// Try to extract availability if the JSON is a Product type
    fn try_extract_from_product(json: &serde_json::Value) -> Option<String> {
        if Self::is_product_type(json) {
            Self::get_availability_from_product(json)
        } else {
            None
        }
    }

    /// Try to extract availability if the JSON is a ProductGroup type
    fn try_extract_from_product_group(
        json: &serde_json::Value,
        variant_id: Option<&str>,
    ) -> Option<String> {
        if Self::is_product_group_type(json) {
            Self::get_availability_from_product_group(json, variant_id)
        } else {
            None
        }
    }

    /// Try to extract availability from a @graph array
    fn try_extract_from_graph(
        json: &serde_json::Value,
        variant_id: Option<&str>,
    ) -> Option<String> {
        json.get("@graph")
            .and_then(|g| g.as_array())
            .and_then(|arr| Self::find_availability_in_items(arr, variant_id))
    }

    /// Try to extract availability from a direct JSON array
    fn try_extract_from_array(
        json: &serde_json::Value,
        variant_id: Option<&str>,
    ) -> Option<String> {
        json.as_array()
            .and_then(|arr| Self::find_availability_in_items(arr, variant_id))
    }

    /// Iterate through items looking for availability data
    fn find_availability_in_items(
        items: &[serde_json::Value],
        variant_id: Option<&str>,
    ) -> Option<String> {
        items.iter().find_map(|item| {
            Self::try_extract_from_product(item)
                .or_else(|| Self::try_extract_from_product_group(item, variant_id))
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
        Self::has_schema_type(json, "Product")
    }

    /// Check if a JSON value represents a ProductGroup type
    fn is_product_group_type(json: &serde_json::Value) -> bool {
        Self::has_schema_type(json, "ProductGroup")
    }

    /// Get availability from a ProductGroup by matching variant ID
    fn get_availability_from_product_group(
        product_group: &serde_json::Value,
        variant_id: Option<&str>,
    ) -> Option<String> {
        let variants = product_group.get("hasVariant")?.as_array()?;

        // If we have a variant ID, try to find the matching variant
        if let Some(vid) = variant_id {
            // Dummy base for resolving relative URLs (host is irrelevant)
            let base = Url::parse("http://localhost").unwrap();

            for variant in variants {
                let Some(id) = variant.get("@id").and_then(|i| i.as_str()) else {
                    continue;
                };

                // Parse as absolute URL first, fall back to relative
                let Ok(parsed_url) = Url::parse(id).or_else(|_| base.join(id)) else {
                    continue;
                };

                let matches_variant = parsed_url
                    .query_pairs()
                    .any(|(key, value)| key == "variant" && value == vid);

                if !matches_variant {
                    continue;
                }

                if let Some(avail) = Self::get_availability_from_product(variant) {
                    return Some(avail);
                }
            }
        }

        // Fallback: return first variant's availability
        for variant in variants {
            if let Some(avail) = Self::get_availability_from_product(variant) {
                return Some(avail);
            }
        }

        None
    }

    /// Get availability string from a Product JSON object
    fn get_availability_from_product(product: &serde_json::Value) -> Option<String> {
        // Try offers.availability first (single offer)
        if let Some(offers) = product.get("offers") {
            // Single offer object
            if let Some(avail) = offers.get("availability").and_then(|a| a.as_str()) {
                return Some(avail.to_string());
            }

            // Array of offers - use first one
            if let Some(arr) = offers.as_array() {
                for offer in arr {
                    if let Some(avail) = offer.get("availability").and_then(|a| a.as_str()) {
                        return Some(avail.to_string());
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_schema_org_in_stock() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@context": "https://schema.org",
                    "@type": "Product",
                    "name": "Test Product",
                    "offers": {
                        "@type": "Offer",
                        "availability": "http://schema.org/InStock",
                        "price": "99.99"
                    }
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result =
            ScraperService::parse_schema_org_with_url(html, "https://example.com").unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
        assert_eq!(
            result.raw_availability,
            Some("http://schema.org/InStock".to_string())
        );
    }

    #[test]
    fn test_parse_schema_org_out_of_stock() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@context": "https://schema.org",
                    "@type": "Product",
                    "name": "Test Product",
                    "offers": {
                        "@type": "Offer",
                        "availability": "https://schema.org/OutOfStock"
                    }
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result =
            ScraperService::parse_schema_org_with_url(html, "https://example.com").unwrap();
        assert_eq!(result.status, AvailabilityStatus::OutOfStock);
    }

    #[test]
    fn test_parse_schema_org_back_order() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@context": "https://schema.org",
                    "@type": "Product",
                    "name": "Test Product",
                    "offers": {
                        "availability": "http://schema.org/BackOrder"
                    }
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result =
            ScraperService::parse_schema_org_with_url(html, "https://example.com").unwrap();
        assert_eq!(result.status, AvailabilityStatus::BackOrder);
    }

    #[test]
    fn test_parse_schema_org_with_graph() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@context": "https://schema.org",
                    "@graph": [
                        {
                            "@type": "WebSite",
                            "name": "Test Store"
                        },
                        {
                            "@type": "Product",
                            "name": "Test Product",
                            "offers": {
                                "availability": "http://schema.org/InStock"
                            }
                        }
                    ]
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result =
            ScraperService::parse_schema_org_with_url(html, "https://example.com").unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
    }

    #[test]
    fn test_parse_schema_org_array_of_offers() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": "Product",
                    "name": "Test Product",
                    "offers": [
                        {
                            "availability": "http://schema.org/OutOfStock",
                            "price": "49.99"
                        },
                        {
                            "availability": "http://schema.org/InStock",
                            "price": "99.99"
                        }
                    ]
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result =
            ScraperService::parse_schema_org_with_url(html, "https://example.com").unwrap();
        // Should use first offer's availability
        assert_eq!(result.status, AvailabilityStatus::OutOfStock);
    }

    #[test]
    fn test_parse_schema_org_no_product() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": "WebSite",
                    "name": "Test Store"
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result = ScraperService::parse_schema_org_with_url(html, "https://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_schema_org_no_json_ld() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Test Page</title>
            </head>
            <body></body>
            </html>
        "#;

        let result = ScraperService::parse_schema_org_with_url(html, "https://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_schema_org_invalid_json() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                    not valid json
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result = ScraperService::parse_schema_org_with_url(html, "https://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_schema_org_product_without_availability() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": "Product",
                    "name": "Test Product",
                    "offers": {
                        "price": "99.99"
                    }
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result = ScraperService::parse_schema_org_with_url(html, "https://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_schema_org_multiple_json_ld_blocks() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": "Organization",
                    "name": "Test Org"
                }
                </script>
                <script type="application/ld+json">
                {
                    "@type": "Product",
                    "name": "Test Product",
                    "offers": {
                        "availability": "http://schema.org/InStock"
                    }
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result =
            ScraperService::parse_schema_org_with_url(html, "https://example.com").unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
    }

    #[test]
    fn test_parse_schema_org_array_type() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": ["Product", "IndividualProduct"],
                    "name": "Test Product",
                    "offers": {
                        "availability": "http://schema.org/InStock"
                    }
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result =
            ScraperService::parse_schema_org_with_url(html, "https://example.com").unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
    }

    #[test]
    fn test_parse_schema_org_product_group_with_variant() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@context": "http://schema.org/",
                    "@type": "ProductGroup",
                    "name": "Test Product",
                    "hasVariant": [
                        {
                            "@id": "/products/test?variant=123#variant",
                            "@type": "Product",
                            "name": "Test - Silver",
                            "offers": {
                                "@type": "Offer",
                                "availability": "http://schema.org/OutOfStock"
                            }
                        },
                        {
                            "@id": "/products/test?variant=456#variant",
                            "@type": "Product",
                            "name": "Test - Black",
                            "offers": {
                                "@type": "Offer",
                                "availability": "http://schema.org/InStock"
                            }
                        }
                    ]
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        // With matching variant ID, should return that variant's availability
        let result = ScraperService::parse_schema_org_with_url(
            html,
            "https://example.com/products/test?variant=456",
        )
        .unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);

        // With different variant ID
        let result = ScraperService::parse_schema_org_with_url(
            html,
            "https://example.com/products/test?variant=123",
        )
        .unwrap();
        assert_eq!(result.status, AvailabilityStatus::OutOfStock);
    }

    #[test]
    fn test_parse_schema_org_product_group_no_variant_id() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": "ProductGroup",
                    "hasVariant": [
                        {
                            "@type": "Product",
                            "offers": {
                                "availability": "http://schema.org/BackOrder"
                            }
                        }
                    ]
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        // Without variant ID in URL, should return first variant
        let result =
            ScraperService::parse_schema_org_with_url(html, "https://example.com/products/test")
                .unwrap();
        assert_eq!(result.status, AvailabilityStatus::BackOrder);
    }

    #[test]
    fn test_extract_variant_id() {
        assert_eq!(
            ScraperService::extract_variant_id("https://example.com/products/test?variant=12345"),
            Some("12345".to_string())
        );
        assert_eq!(
            ScraperService::extract_variant_id("https://example.com/products/test"),
            None
        );
        assert_eq!(
            ScraperService::extract_variant_id(
                "https://example.com/products/test?foo=bar&variant=999&baz=qux"
            ),
            Some("999".to_string())
        );
    }

    #[test]
    fn test_parse_schema_org_shopify_style() {
        // Real-world Shopify structure
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@context": "http://schema.org/",
                    "@id": "/products/flipaction-elite-16#product",
                    "@type": "ProductGroup",
                    "brand": {
                        "@type": "Brand",
                        "name": "SOTSU"
                    },
                    "category": "Computer Monitors",
                    "hasVariant": [
                        {
                            "@id": "/products/flipaction-elite-16?variant=46314945118434#variant",
                            "@type": "Product",
                            "name": "FlipAction Elite 16\" - Silver",
                            "offers": {
                                "@id": "/products/flipaction-elite-16?variant=46314945118434#offer",
                                "@type": "Offer",
                                "availability": "http://schema.org/BackOrder",
                                "price": "789.00",
                                "priceCurrency": "USD"
                            },
                            "sku": "SFAE16PMSV"
                        },
                        {
                            "@id": "/products/flipaction-elite-16?variant=46518950953186#variant",
                            "@type": "Product",
                            "name": "FlipAction Elite 16\" - Space Black",
                            "offers": {
                                "@id": "/products/flipaction-elite-16?variant=46518950953186#offer",
                                "@type": "Offer",
                                "availability": "http://schema.org/InStock",
                                "price": "789.00",
                                "priceCurrency": "USD"
                            },
                            "sku": "SFAE16PMSPB"
                        }
                    ]
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        // Should find Silver variant (BackOrder)
        let result = ScraperService::parse_schema_org_with_url(
            html,
            "https://www.sotsu.com/products/flipaction-elite-16?variant=46314945118434",
        )
        .unwrap();
        assert_eq!(result.status, AvailabilityStatus::BackOrder);

        // Should find Space Black variant (InStock)
        let result = ScraperService::parse_schema_org_with_url(
            html,
            "https://www.sotsu.com/products/flipaction-elite-16?variant=46518950953186",
        )
        .unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
    }

    #[tokio::test]
    async fn test_check_availability_rejects_file_scheme() {
        let result = ScraperService::check_availability("file:///etc/passwd").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::Validation(msg) => {
                assert!(msg.contains("Unsupported URL scheme"));
                assert!(msg.contains("file"));
            }
            _ => panic!("Expected Validation error, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_check_availability_rejects_data_scheme() {
        let result = ScraperService::check_availability("data:text/html,<h1>Hello</h1>").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::Validation(msg) => {
                assert!(msg.contains("Unsupported URL scheme"));
                assert!(msg.contains("data"));
            }
            _ => panic!("Expected Validation error, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_check_availability_rejects_invalid_url() {
        let result = ScraperService::check_availability("not a valid url").await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::Validation(msg) => {
                assert!(msg.contains("Invalid URL"));
            }
            _ => panic!("Expected Validation error, got {:?}", err),
        }
    }

    // Cloudflare challenge detection tests

    #[test]
    fn test_is_cloudflare_challenge_detects_just_a_moment() {
        let body = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Just a moment...</title></head>
            <body>
                <div>Checking your browser before accessing the website.</div>
            </body>
            </html>
        "#;
        assert!(ScraperService::is_cloudflare_challenge(403, body));
    }

    #[test]
    fn test_is_cloudflare_challenge_detects_cf_browser_verification() {
        let body = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Loading</title></head>
            <body>
                <div id="cf-browser-verification">Please wait...</div>
            </body>
            </html>
        "#;
        assert!(ScraperService::is_cloudflare_challenge(403, body));
    }

    #[test]
    fn test_is_cloudflare_challenge_detects_cf_chl_opt() {
        let body = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script>window._cf_chl_opt = {...}</script>
            </head>
            <body></body>
            </html>
        "#;
        assert!(ScraperService::is_cloudflare_challenge(503, body));
    }

    #[test]
    fn test_is_cloudflare_challenge_detects_ray_id() {
        let body = r#"
            <!DOCTYPE html>
            <html>
            <body>
                <p>Ray ID: abc123xyz</p>
            </body>
            </html>
        "#;
        assert!(ScraperService::is_cloudflare_challenge(403, body));
    }

    #[test]
    fn test_is_cloudflare_challenge_returns_false_for_normal_page() {
        let body = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Product Page</title></head>
            <body>
                <h1>Test Product</h1>
                <script type="application/ld+json">
                {"@type": "Product", "offers": {"availability": "InStock"}}
                </script>
            </body>
            </html>
        "#;
        assert!(!ScraperService::is_cloudflare_challenge(200, body));
    }

    #[test]
    fn test_is_cloudflare_challenge_returns_false_for_normal_403() {
        // A 403 page with proper content (long body, has JSON-LD) should not be detected
        let body = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>403 Forbidden</title>
                <script type="application/ld+json">
                {"@type": "Product", "name": "Test"}
                </script>
            </head>
            <body>
                <h1>Access Denied</h1>
                <p>You don't have permission to access this resource.</p>
                <p>This is a longer page with actual content that wouldn't be a bot challenge.</p>
            </body>
            </html>
        "#;
        // Normal 403 with content should not be detected as a challenge
        assert!(!ScraperService::is_cloudflare_challenge(403, body));
    }

    #[test]
    fn test_is_cloudflare_challenge_case_insensitive() {
        let body = r#"<html><body>JUST A MOMENT...</body></html>"#;
        assert!(ScraperService::is_cloudflare_challenge(403, body));
    }
}
