use std::time::Duration;

use scraper::{Html, Selector};
use url::Url;

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
    pub async fn check_availability(url: &str) -> Result<ScrapingResult, AppError> {
        let parsed =
            Url::parse(url).map_err(|e| AppError::Validation(format!("Invalid URL: {}", e)))?;

        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(AppError::Validation(format!(
                "Unsupported URL scheme '{}'. Only http and https are allowed.",
                scheme
            )));
        }

        let html = Self::fetch_page(url).await?;
        Self::parse_schema_org_with_url(&html, url)
    }

    /// Fetch a page's HTML content
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

    /// Extract availability from a JSON-LD value
    fn extract_availability(json: &serde_json::Value, variant_id: Option<&str>) -> Option<String> {
        // Direct Product type
        if Self::is_product_type(json) {
            if let Some(avail) = Self::get_availability_from_product(json) {
                return Some(avail);
            }
        }

        // ProductGroup with variants (Shopify style)
        if Self::is_product_group_type(json) {
            if let Some(avail) = Self::get_availability_from_product_group(json, variant_id) {
                return Some(avail);
            }
        }

        // Check @graph array
        if let Some(graph) = json.get("@graph").and_then(|g| g.as_array()) {
            for item in graph {
                if Self::is_product_type(item) {
                    if let Some(avail) = Self::get_availability_from_product(item) {
                        return Some(avail);
                    }
                }
                if Self::is_product_group_type(item) {
                    if let Some(avail) = Self::get_availability_from_product_group(item, variant_id)
                    {
                        return Some(avail);
                    }
                }
            }
        }

        // Check if it's an array of items
        if let Some(arr) = json.as_array() {
            for item in arr {
                if Self::is_product_type(item) {
                    if let Some(avail) = Self::get_availability_from_product(item) {
                        return Some(avail);
                    }
                }
                if Self::is_product_group_type(item) {
                    if let Some(avail) = Self::get_availability_from_product_group(item, variant_id)
                    {
                        return Some(avail);
                    }
                }
            }
        }

        None
    }

    /// Check if a JSON value represents a Product type
    fn is_product_type(json: &serde_json::Value) -> bool {
        json.get("@type")
            .map(|t| {
                if let Some(s) = t.as_str() {
                    s == "Product"
                } else if let Some(arr) = t.as_array() {
                    arr.iter().any(|v| v.as_str() == Some("Product"))
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }

    /// Check if a JSON value represents a ProductGroup type
    fn is_product_group_type(json: &serde_json::Value) -> bool {
        json.get("@type")
            .map(|t| {
                if let Some(s) = t.as_str() {
                    s == "ProductGroup"
                } else if let Some(arr) = t.as_array() {
                    arr.iter().any(|v| v.as_str() == Some("ProductGroup"))
                } else {
                    false
                }
            })
            .unwrap_or(false)
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
}
