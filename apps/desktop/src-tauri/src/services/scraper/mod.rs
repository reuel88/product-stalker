/// Scraper service for extracting product availability from web pages.
///
/// This module is organized into focused submodules:
/// - `bot_detection`: Cloudflare and bot protection detection
/// - `chemist_warehouse`: Site-specific adapter for Chemist Warehouse
/// - `http_client`: HTTP fetching with browser-like headers and headless fallback
/// - `nextjs_data`: Next.js __NEXT_DATA__ extraction
/// - `price_parser`: Price extraction and normalization
/// - `schema_org`: JSON-LD Schema.org data parsing
/// - `shopify`: Shopify store adapter using cart API for availability
mod bot_detection;
mod chemist_warehouse;
mod http_client;
mod nextjs_data;
mod price_parser;
mod schema_org;
mod shopify;

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
    #[cfg(test)]
    pub async fn check_availability(url: &str) -> Result<ScrapingResult, AppError> {
        Self::check_availability_with_headless(url, true).await
    }

    /// Check availability with control over headless fallback
    ///
    /// This is the main orchestrator function that coordinates the scraping workflow:
    /// 1. Validate URL scheme
    /// 2. Fetch HTML (with automatic headless fallback if bot protection detected)
    /// 3. Try Schema.org extraction first
    /// 4. Try Shopify-specific extraction for Shopify stores
    /// 5. Fall back to other site-specific parsers (e.g., Next.js data)
    pub async fn check_availability_with_headless(
        url: &str,
        enable_headless: bool,
    ) -> Result<ScrapingResult, AppError> {
        // Step 1: Validate URL scheme
        Self::validate_url_scheme(url)?;

        // Step 2: Fetch HTML (tries HTTP first, falls back to headless if needed)
        let html = http_client::fetch_html_with_fallback(url, enable_headless).await?;

        // Step 3: Try Schema.org extraction first
        if let Ok(result) = Self::try_schema_org_extraction(&html, url) {
            return Ok(result);
        }

        // Step 4: Try Shopify extraction (async - uses cart API)
        if shopify::is_potential_shopify_product_url(url) {
            log::debug!(
                "URL matches Shopify pattern, trying Shopify extraction for {}",
                url
            );
            if let Ok(result) = shopify::check_shopify_availability(url, &html).await {
                return Ok(result);
            }
        }

        // Step 5: Fall back to other site-specific parsers (sync)
        Self::try_site_specific_extraction(&html, url)
    }

    /// Try to extract availability from Schema.org JSON-LD data
    fn try_schema_org_extraction(html: &str, url: &str) -> Result<ScrapingResult, AppError> {
        let variant_id = schema_org::extract_variant_id(url);
        let json_ld_blocks = schema_org::extract_json_ld_blocks(html)?;

        log::debug!(
            "Schema.org extraction: found {} JSON-LD block(s) for URL: {}",
            json_ld_blocks.len(),
            url
        );

        for (i, block) in json_ld_blocks.iter().enumerate() {
            let block_type = block
                .get("@type")
                .map(|t| t.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            log::debug!("JSON-LD block {}: @type = {}", i, block_type);

            if let Some((availability, price)) =
                schema_org::extract_availability_and_price(block, variant_id.as_deref())
            {
                log::debug!(
                    "Extracted raw availability value: '{}' -> status: {:?}",
                    availability,
                    AvailabilityStatus::from_schema_org(&availability)
                );
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

    /// Try site-specific extraction methods based on URL domain
    fn try_site_specific_extraction(html: &str, url: &str) -> Result<ScrapingResult, AppError> {
        // Chemist Warehouse: uses Next.js with product data in __NEXT_DATA__
        if chemist_warehouse::is_chemist_warehouse_url(url) {
            return Self::try_chemist_warehouse_extraction(html);
        }

        // No site-specific parser matched
        Err(AppError::Scraping(
            "No availability information found. Site does not use Schema.org or a supported data format.".to_string(),
        ))
    }

    /// Extract availability from Chemist Warehouse using Next.js data
    fn try_chemist_warehouse_extraction(html: &str) -> Result<ScrapingResult, AppError> {
        let next_data = nextjs_data::extract_next_data(html)?;
        let page_props = nextjs_data::get_page_props(&next_data)
            .ok_or_else(|| AppError::Scraping("No pageProps found in Next.js data".to_string()))?;
        chemist_warehouse::parse_chemist_warehouse_data(page_props)
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
    /// This is a convenience function for tests that already have HTML
    /// and just need to parse it. Delegates to `try_schema_org_extraction`.
    #[cfg(test)]
    pub fn parse_schema_org_with_url(html: &str, url: &str) -> Result<ScrapingResult, AppError> {
        Self::try_schema_org_extraction(html, url)
    }
}

/// Test helper module for generating HTML templates
#[cfg(test)]
mod test_html {
    /// Generate HTML with a simple Product offer
    ///
    /// # Arguments
    /// * `availability` - Schema.org availability value (e.g., "http://schema.org/InStock")
    /// * `price` - Optional price value
    /// * `currency` - Optional currency code (e.g., "USD")
    pub fn html_with_product_offer(
        availability: &str,
        price: Option<&str>,
        currency: Option<&str>,
    ) -> String {
        let price_json = match (price, currency) {
            (Some(p), Some(c)) => format!(r#""price": "{}", "priceCurrency": "{}","#, p, c),
            (Some(p), None) => format!(r#""price": "{}","#, p),
            _ => String::new(),
        };

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <script type="application/ld+json">
    {{
        "@context": "https://schema.org",
        "@type": "Product",
        "name": "Test Product",
        "offers": {{
            "@type": "Offer",
            {}
            "availability": "{}"
        }}
    }}
    </script>
</head>
<body></body>
</html>"#,
            price_json, availability
        )
    }

    /// Variant info for ProductGroup HTML generation
    pub struct VariantInfo<'a> {
        pub variant_id: &'a str,
        pub availability: &'a str,
        pub price: Option<&'a str>,
        pub currency: Option<&'a str>,
    }

    /// Generate HTML with a ProductGroup containing variants
    ///
    /// # Arguments
    /// * `variants` - Slice of variant information
    pub fn html_with_product_group(variants: &[VariantInfo]) -> String {
        let variants_json: Vec<String> = variants
            .iter()
            .map(|v| {
                let price_json = match (v.price, v.currency) {
                    (Some(p), Some(c)) => format!(r#""price": "{}", "priceCurrency": "{}","#, p, c),
                    (Some(p), None) => format!(r#""price": "{}","#, p),
                    _ => String::new(),
                };
                format!(
                    r#"{{
                "@id": "/products/test?variant={}#variant",
                "@type": "Product",
                "offers": {{
                    "@type": "Offer",
                    {}
                    "availability": "{}"
                }}
            }}"#,
                    v.variant_id, price_json, v.availability
                )
            })
            .collect();

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <script type="application/ld+json">
    {{
        "@context": "http://schema.org/",
        "@type": "ProductGroup",
        "name": "Test Product",
        "hasVariant": [
            {}
        ]
    }}
    </script>
</head>
<body></body>
</html>"#,
            variants_json.join(",\n            ")
        )
    }

    /// Generate HTML with Next.js __NEXT_DATA__ for Chemist Warehouse-style pages
    ///
    /// # Arguments
    /// * `product_json` - The product JSON to embed in pageProps
    pub fn html_with_next_data(product_json: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <script id="__NEXT_DATA__" type="application/json">
    {{
        "props": {{
            "pageProps": {{
                "product": {}
            }}
        }}
    }}
    </script>
</head>
<body></body>
</html>"#,
            product_json
        )
    }
}

#[cfg(test)]
mod tests {
    use super::test_html::{
        html_with_next_data, html_with_product_group, html_with_product_offer, VariantInfo,
    };
    use super::*;

    #[test]
    fn test_parse_schema_org_in_stock() {
        let html = html_with_product_offer("http://schema.org/InStock", Some("99.99"), None);

        let result =
            ScraperService::parse_schema_org_with_url(&html, "https://example.com").unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
        assert_eq!(
            result.raw_availability,
            Some("http://schema.org/InStock".to_string())
        );
    }

    #[test]
    fn test_parse_schema_org_out_of_stock() {
        let html = html_with_product_offer("https://schema.org/OutOfStock", None, None);

        let result =
            ScraperService::parse_schema_org_with_url(&html, "https://example.com").unwrap();
        assert_eq!(result.status, AvailabilityStatus::OutOfStock);
    }

    #[test]
    fn test_parse_schema_org_back_order() {
        let html = html_with_product_offer("http://schema.org/BackOrder", None, None);

        let result =
            ScraperService::parse_schema_org_with_url(&html, "https://example.com").unwrap();
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
        let html = html_with_product_group(&[VariantInfo {
            variant_id: "123",
            availability: "http://schema.org/BackOrder",
            price: None,
            currency: None,
        }]);

        // Without variant ID in URL, should return first variant
        let result =
            ScraperService::parse_schema_org_with_url(&html, "https://example.com/products/test")
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
        let html =
            html_with_product_offer("http://schema.org/InStock", Some("789.00"), Some("USD"));

        let result =
            ScraperService::parse_schema_org_with_url(&html, "https://example.com").unwrap();
        assert_eq!(result.price.price_cents, Some(78900));
        assert_eq!(result.price.price_currency, Some("USD".to_string()));
        assert_eq!(result.price.raw_price, Some("789.00".to_string()));
    }

    #[test]
    fn test_price_extraction_from_shopify_product_group() {
        let html = html_with_product_group(&[VariantInfo {
            variant_id: "123",
            availability: "http://schema.org/InStock",
            price: Some("1,299.00"),
            currency: Some("AUD"),
        }]);

        let result = ScraperService::parse_schema_org_with_url(
            &html,
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
        let html = html_with_product_offer("http://schema.org/InStock", None, None);

        let result =
            ScraperService::parse_schema_org_with_url(&html, "https://example.com").unwrap();
        assert_eq!(result.price.price_cents, None);
        assert_eq!(result.price.price_currency, None);
        assert_eq!(result.price.raw_price, None);
    }

    // Chemist Warehouse fallback tests

    #[test]
    fn test_chemist_warehouse_extraction_in_stock() {
        let html = html_with_next_data(
            r#"{
                "name": "Curash Simply Water Wipes 6 x 80 Pack",
                "sku": "2678514",
                "price": "23.99",
                "availability": "in-stock"
            }"#,
        );

        let result = ScraperService::try_chemist_warehouse_extraction(&html).unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
        assert_eq!(result.raw_availability, Some("in-stock".to_string()));
        assert_eq!(result.price.price_cents, Some(2399));
        assert_eq!(result.price.price_currency, Some("AUD".to_string()));
    }

    #[test]
    fn test_chemist_warehouse_extraction_out_of_stock() {
        let html = html_with_next_data(
            r#"{
                "name": "Some Product",
                "sku": "12345",
                "price": "19.99",
                "availability": "out-of-stock"
            }"#,
        );

        let result = ScraperService::try_chemist_warehouse_extraction(&html).unwrap();
        assert_eq!(result.status, AvailabilityStatus::OutOfStock);
        assert_eq!(result.raw_availability, Some("out-of-stock".to_string()));
    }

    #[test]
    fn test_site_specific_extraction_chemist_warehouse() {
        let html = html_with_next_data(
            r#"{
                "name": "Test Product",
                "price": "29.99",
                "availability": "in-stock"
            }"#,
        );

        let result = ScraperService::try_site_specific_extraction(
            &html,
            "https://www.chemistwarehouse.com.au/buy/87324/curash-simply-water-wipes",
        )
        .unwrap();
        assert_eq!(result.status, AvailabilityStatus::InStock);
        assert_eq!(result.price.price_cents, Some(2999));
    }

    #[test]
    fn test_site_specific_extraction_unsupported_site() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Random Site</title></head>
            <body></body>
            </html>
        "#;

        let result =
            ScraperService::try_site_specific_extraction(html, "https://example.com/product");
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::Scraping(msg) => {
                assert!(msg.contains("supported data format"));
            }
            _ => panic!("Expected Scraping error"),
        }
    }

    #[test]
    fn test_fallback_from_schema_org_to_chemist_warehouse() {
        // HTML with no Schema.org but has Next.js data
        let html = html_with_next_data(
            r#"{
                "name": "Baby Wipes",
                "price": "15.50",
                "availability": "in-stock"
            }"#,
        );

        // Schema.org should fail (no ld+json)
        let schema_result =
            ScraperService::try_schema_org_extraction(&html, "https://chemistwarehouse.com.au");
        assert!(schema_result.is_err());

        // But Chemist Warehouse extraction should work
        let cw_result = ScraperService::try_chemist_warehouse_extraction(&html).unwrap();
        assert_eq!(cw_result.status, AvailabilityStatus::InStock);
        assert_eq!(cw_result.price.price_cents, Some(1550));
    }

    #[test]
    fn test_schema_org_takes_priority_over_nextjs() {
        // HTML with both Schema.org and Next.js data (Schema.org should win)
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <script type="application/ld+json">
                {
                    "@type": "Product",
                    "name": "Test Product",
                    "offers": {
                        "availability": "http://schema.org/OutOfStock",
                        "price": "99.99"
                    }
                }
                </script>
                <script id="__NEXT_DATA__" type="application/json">
                {
                    "props": {
                        "pageProps": {
                            "product": {
                                "name": "Test Product",
                                "price": "50.00",
                                "availability": "in-stock"
                            }
                        }
                    }
                }
                </script>
            </head>
            <body></body>
            </html>
        "#;

        // Schema.org should succeed and take priority
        let result = ScraperService::try_schema_org_extraction(
            html,
            "https://www.chemistwarehouse.com.au/product",
        )
        .unwrap();
        assert_eq!(result.status, AvailabilityStatus::OutOfStock); // From Schema.org
        assert_eq!(result.price.price_cents, Some(9999)); // From Schema.org
    }
}
