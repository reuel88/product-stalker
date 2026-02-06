/// Scraper service for extracting product availability from web pages.
///
/// This module is organized into focused submodules:
/// - `bot_detection`: Cloudflare and bot protection detection
/// - `http_client`: HTTP fetching with browser-like headers and headless fallback
/// - `price_parser`: Price extraction and normalization
/// - `schema_org`: JSON-LD Schema.org data parsing
mod bot_detection;
mod http_client;
mod price_parser;
mod schema_org;

use url::Url;

use crate::entities::availability_check::AvailabilityStatus;
use crate::error::AppError;

// Re-export types that are part of the public API
pub use price_parser::PriceInfo;

/// Result of a scraping operation
#[derive(Debug, Clone)]
pub struct ScrapingResult {
    pub status: AvailabilityStatus,
    pub raw_availability: Option<String>,
    pub price: PriceInfo,
}

/// Service for scraping product availability from web pages
pub struct ScraperService;

impl ScraperService {
    /// Check availability by fetching a URL and parsing Schema.org data
    ///
    /// Uses HTTP as the fast path. Falls back to headless browser if bot
    /// protection (Cloudflare, etc.) is detected and headless is enabled.
    #[allow(dead_code)]
    pub async fn check_availability(url: &str) -> Result<ScrapingResult, AppError> {
        Self::check_availability_with_headless(url, true).await
    }

    /// Check availability with control over headless fallback
    ///
    /// This is the main orchestrator function that coordinates the scraping workflow:
    /// 1. Validate URL scheme
    /// 2. Fetch HTML (with automatic headless fallback if bot protection detected)
    /// 3. Extract variant ID from URL
    /// 4. Parse JSON-LD blocks from HTML
    /// 5. Find availability and price data
    pub async fn check_availability_with_headless(
        url: &str,
        enable_headless: bool,
    ) -> Result<ScrapingResult, AppError> {
        // Step 1: Validate URL scheme
        Self::validate_url_scheme(url)?;

        // Step 2: Fetch HTML (tries HTTP first, falls back to headless if needed)
        let html = http_client::fetch_html_with_fallback(url, enable_headless).await?;

        // Step 3: Extract variant ID from URL query params
        let variant_id = schema_org::extract_variant_id(url);

        // Step 4: Parse all JSON-LD blocks from HTML
        let json_ld_blocks = schema_org::extract_json_ld_blocks(&html)?;

        // Step 5: Find first block with valid availability data
        for block in json_ld_blocks {
            if let Some((availability, price)) =
                schema_org::extract_availability_and_price(&block, variant_id.as_deref())
            {
                return Ok(ScrapingResult {
                    status: AvailabilityStatus::from_schema_org(&availability),
                    raw_availability: Some(availability),
                    price,
                });
            }
        }

        Err(AppError::Scraping(
            "No availability information found in Schema.org data".to_string(),
        ))
    }

    /// Validate that the URL uses http or https scheme
    fn validate_url_scheme(url: &str) -> Result<(), AppError> {
        let parsed =
            Url::parse(url).map_err(|e| AppError::Validation(format!("Invalid URL: {}", e)))?;

        let scheme = parsed.scheme();
        if scheme != "http" && scheme != "https" {
            return Err(AppError::Validation(format!(
                "Unsupported URL scheme '{}'. Only http and https are allowed.",
                scheme
            )));
        }
        Ok(())
    }

    /// Parse Schema.org JSON-LD data from HTML to extract availability
    /// Uses the URL to match specific product variants
    ///
    /// This is a convenience function that combines the orchestrator steps for
    /// callers who already have HTML and just need to parse it.
    #[cfg(test)]
    pub fn parse_schema_org_with_url(html: &str, url: &str) -> Result<ScrapingResult, AppError> {
        let variant_id = schema_org::extract_variant_id(url);
        let json_ld_blocks = schema_org::extract_json_ld_blocks(html)?;

        for block in json_ld_blocks {
            if let Some((availability, price)) =
                schema_org::extract_availability_and_price(&block, variant_id.as_deref())
            {
                return Ok(ScrapingResult {
                    status: AvailabilityStatus::from_schema_org(&availability),
                    raw_availability: Some(availability),
                    price,
                });
            }
        }

        Err(AppError::Scraping(
            "No availability information found in Schema.org data".to_string(),
        ))
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

    #[test]
    fn test_price_extraction_from_product() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": "Product",
                    "name": "Test Product",
                    "offers": {
                        "@type": "Offer",
                        "availability": "http://schema.org/InStock",
                        "price": "789.00",
                        "priceCurrency": "USD"
                    }
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result =
            ScraperService::parse_schema_org_with_url(html, "https://example.com").unwrap();
        assert_eq!(result.price.price_cents, Some(78900));
        assert_eq!(result.price.price_currency, Some("USD".to_string()));
        assert_eq!(result.price.raw_price, Some("789.00".to_string()));
    }

    #[test]
    fn test_price_extraction_from_shopify_product_group() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": "ProductGroup",
                    "hasVariant": [
                        {
                            "@id": "/products/test?variant=123#variant",
                            "@type": "Product",
                            "offers": {
                                "availability": "http://schema.org/InStock",
                                "price": "1,299.00",
                                "priceCurrency": "AUD"
                            }
                        }
                    ]
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result = ScraperService::parse_schema_org_with_url(
            html,
            "https://example.com/products/test?variant=123",
        )
        .unwrap();
        assert_eq!(result.price.price_cents, Some(129900));
        assert_eq!(result.price.price_currency, Some("AUD".to_string()));
    }

    #[test]
    fn test_price_extraction_with_numeric_price() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": "Product",
                    "name": "Test Product",
                    "offers": {
                        "availability": "http://schema.org/InStock",
                        "price": 49.99,
                        "priceCurrency": "EUR"
                    }
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        let result =
            ScraperService::parse_schema_org_with_url(html, "https://example.com").unwrap();
        assert_eq!(result.price.price_cents, Some(4999));
        assert_eq!(result.price.price_currency, Some("EUR".to_string()));
        assert_eq!(result.price.raw_price, Some("49.99".to_string()));
    }

    #[test]
    fn test_price_extraction_no_price() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
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
        assert_eq!(result.price.price_cents, None);
        assert_eq!(result.price.price_currency, None);
        assert_eq!(result.price.raw_price, None);
    }
}
