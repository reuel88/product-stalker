use std::time::Duration;

use scraper::{Html, Selector};
use url::Url;

use super::HeadlessService;
use crate::entities::availability_check::AvailabilityStatus;
use crate::error::AppError;

/// Price information extracted from Schema.org data
#[derive(Debug, Clone, Default)]
pub struct PriceInfo {
    pub price_cents: Option<i64>,
    pub price_currency: Option<String>,
    pub raw_price: Option<String>,
}

/// Result of a scraping operation
#[derive(Debug, Clone)]
pub struct ScrapingResult {
    pub status: AvailabilityStatus,
    pub raw_availability: Option<String>,
    pub price: PriceInfo,
}

/// Service for scraping product availability from web pages
pub struct ScraperService;

/// Error message shown when bot protection is detected and headless browser is disabled.
const BOT_PROTECTION_MESSAGE: &str =
    "This site has bot protection. Enable headless browser in settings to check this site.";

impl ScraperService {
    /// HTTP request timeout
    const TIMEOUT_SECS: u64 = 30;

    /// User-Agent header mimicking Chrome browser.
    ///
    /// Using a realistic browser User-Agent helps avoid basic bot detection
    /// that blocks requests with obvious automation signatures like "curl" or "python-requests".
    const USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

    /// HTTP Accept header for HTML content
    const ACCEPT_HEADER: &'static str =
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8";

    /// Sec-Ch-Ua header for Chrome browser fingerprint
    const SEC_CH_UA: &'static str =
        r#""Not_A Brand";v="8", "Chromium";v="120", "Google Chrome";v="120""#;

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
        let html = Self::fetch_html_with_fallback(url, enable_headless).await?;

        // Step 3: Extract variant ID from URL query params
        let variant_id = Self::extract_variant_id(url);

        // Step 4: Parse all JSON-LD blocks from HTML
        let json_ld_blocks = Self::extract_json_ld_blocks(&html)?;

        // Step 5: Find first block with valid availability data
        for block in json_ld_blocks {
            if let Some((availability, price)) =
                Self::extract_availability_and_price(&block, variant_id.as_deref())
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

    /// Fetch HTML content, falling back to headless browser if bot protection is detected
    ///
    /// Tries HTTP first (fast path). If bot protection is detected (Cloudflare challenge,
    /// 403/503 status), falls back to headless browser if enabled.
    async fn fetch_html_with_fallback(
        url: &str,
        enable_headless: bool,
    ) -> Result<String, AppError> {
        match Self::fetch_page(url).await {
            Ok(html) => {
                log::debug!("HTTP fetch succeeded for {}, checking for challenge", url);
                if Self::is_cloudflare_challenge(200, &html) {
                    log::info!("Detected bot protection challenge for {}", url);
                    if enable_headless {
                        log::info!("Attempting headless fallback for {}", url);
                        Self::fetch_with_headless(url).await
                    } else {
                        Err(AppError::BotProtection(BOT_PROTECTION_MESSAGE.to_string()))
                    }
                } else {
                    Ok(html)
                }
            }
            Err(AppError::HttpStatus {
                status,
                url: failed_url,
            }) if status == 403 || status == 503 => {
                log::info!(
                    "HTTP request blocked ({}) for {}, trying headless",
                    status,
                    failed_url
                );
                if enable_headless {
                    log::info!("Attempting headless fallback for {}", failed_url);
                    Self::fetch_with_headless(&failed_url).await
                } else {
                    Err(AppError::BotProtection(BOT_PROTECTION_MESSAGE.to_string()))
                }
            }
            Err(e) => {
                log::error!("HTTP fetch failed for {}: {}", url, e);
                Err(e)
            }
        }
    }

    /// Extract all JSON-LD blocks from HTML
    fn extract_json_ld_blocks(html: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("script[type=\"application/ld+json\"]")
            .map_err(|e| AppError::Scraping(format!("Invalid selector: {:?}", e)))?;

        Ok(document
            .select(&selector)
            .filter_map(|el| serde_json::from_str(&el.inner_html()).ok())
            .collect())
    }

    /// Check if the response is a Cloudflare challenge page or other bot protection
    ///
    /// Returns true if the response appears to be a bot protection challenge rather
    /// than actual page content. Uses multiple detection strategies to minimize
    /// false positives while catching common challenge patterns.
    fn is_cloudflare_challenge(status: u16, body: &str) -> bool {
        let body_lower = body.to_lowercase();

        // Quick exit: pages with product data (JSON-LD) are real content, not challenges
        if Self::has_product_data(&body_lower) {
            return false;
        }

        let is_bot_blocked_status = status == 403 || status == 503;
        let has_cloudflare_markers = Self::has_cloudflare_indicators(&body_lower);
        let has_bot_protection = Self::has_explicit_bot_protection(&body_lower);
        let is_suspiciously_minimal = Self::is_minimal_challenge_page(&body_lower, body.len());

        if is_bot_blocked_status {
            // 403/503 responses: check for challenge indicators or suspiciously minimal content
            has_cloudflare_markers || has_bot_protection || is_suspiciously_minimal
        } else {
            // 200 responses: require strong indicators (avoid false positives)
            has_cloudflare_markers || has_bot_protection
        }
    }

    /// Check if the page contains JSON-LD product data (indicates real content)
    fn has_product_data(body_lower: &str) -> bool {
        body_lower.contains("application/ld+json")
    }

    /// Check for Cloudflare-specific challenge indicators.
    ///
    /// Note: We deliberately avoid matching plain "cloudflare" as it causes
    /// false positives on pages using cdnjs.cloudflare.com or other Cloudflare
    /// CDN resources that aren't actually challenge pages.
    fn has_cloudflare_indicators(body_lower: &str) -> bool {
        const CLOUDFLARE_MARKERS: &[&str] = &[
            // Challenge page title text
            "just a moment...",
            // DOM element ID for browser verification widget
            "cf-browser-verification",
            // JavaScript variable set during challenge handling
            "_cf_chl_opt",
            // User-facing message during verification
            "checking your browser",
            // Cloudflare request identifier shown on challenge pages
            "ray id:",
            // CSS class/ID prefix for challenge-related elements
            "cf-challenge",
            // Bot management cookie name
            "__cf_bm",
            // Challenge platform script path
            "/cdn-cgi/challenge-platform/",
        ];
        CLOUDFLARE_MARKERS
            .iter()
            .any(|marker| body_lower.contains(marker))
    }

    /// Check for explicit bot protection messages from various providers.
    ///
    /// These markers indicate the page is actively blocking automated access
    /// and showing a challenge or block page instead of the actual content.
    fn has_explicit_bot_protection(body_lower: &str) -> bool {
        const BOT_PROTECTION_MARKERS: &[&str] = &[
            // Generic bot detection message
            "bot detected",
            // CAPTCHA or human verification prompt
            "please verify you are a human",
            // Common requirement message on challenge pages
            "enable javascript and cookies",
            // PerimeterX/HUMAN bot protection message
            "pardon our interruption",
        ];
        BOT_PROTECTION_MARKERS
            .iter()
            .any(|marker| body_lower.contains(marker))
    }

    /// Check if the page is suspiciously minimal (likely a challenge, not real content)
    ///
    /// Challenge pages are typically very short or missing a proper body element.
    fn is_minimal_challenge_page(body_lower: &str, body_len: usize) -> bool {
        body_len < 5000 || !body_lower.contains("<body")
    }

    /// Fetch page HTML using headless browser
    ///
    /// Runs the blocking headless browser operations on a dedicated thread pool
    /// to avoid blocking the async runtime.
    async fn fetch_with_headless(url: &str) -> Result<String, AppError> {
        let url_owned = url.to_string();
        tokio::task::spawn_blocking(move || {
            let mut headless = HeadlessService::new();
            headless.fetch_page(&url_owned)
        })
        .await
        .map_err(|e| AppError::Internal(format!("Headless task failed: {}", e)))?
    }

    /// Fetch a page's HTML content using HTTP
    async fn fetch_page(url: &str) -> Result<String, AppError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(Self::TIMEOUT_SECS))
            .build()?;

        let response = client
            .get(url)
            .header("User-Agent", Self::USER_AGENT)
            .header("Accept", Self::ACCEPT_HEADER)
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .header("Sec-Ch-Ua", Self::SEC_CH_UA)
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
            return Err(AppError::HttpStatus {
                status: response.status().as_u16(),
                url: url.to_string(),
            });
        }

        let html = response.text().await?;
        Ok(html)
    }

    /// Parse Schema.org JSON-LD data from HTML to extract availability
    /// Uses the URL to match specific product variants
    ///
    /// This is a convenience function that combines the orchestrator steps for
    /// callers who already have HTML and just need to parse it.
    #[cfg(test)]
    pub fn parse_schema_org_with_url(html: &str, url: &str) -> Result<ScrapingResult, AppError> {
        let variant_id = Self::extract_variant_id(url);
        let json_ld_blocks = Self::extract_json_ld_blocks(html)?;

        for block in json_ld_blocks {
            if let Some((availability, price)) =
                Self::extract_availability_and_price(&block, variant_id.as_deref())
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

    /// Extract variant ID from URL query parameters
    fn extract_variant_id(url: &str) -> Option<String> {
        Url::parse(url).ok().and_then(|parsed| {
            parsed
                .query_pairs()
                .find(|(key, _)| key == "variant")
                .map(|(_, value)| value.to_string())
        })
    }

    /// Parse a price string to cents (smallest currency unit)
    /// Handles formats like "789.00", "1,234.56", "789", "789.9"
    pub fn parse_price_to_cents(price_str: &str) -> Option<i64> {
        // Remove currency symbols, whitespace, and thousand separators
        let cleaned: String = price_str
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();

        if cleaned.is_empty() {
            return None;
        }

        // Parse as float and convert to cents
        let price: f64 = cleaned.parse().ok()?;
        Some((price * 100.0).round() as i64)
    }

    /// Extract price info from an offer object
    fn get_price_from_offer(offer: &serde_json::Value) -> PriceInfo {
        let raw_price = offer.get("price").and_then(|p| match p {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            _ => None,
        });

        let price_currency = offer
            .get("priceCurrency")
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());

        let price_cents = raw_price
            .as_ref()
            .and_then(|p| Self::parse_price_to_cents(p));

        PriceInfo {
            price_cents,
            price_currency,
            raw_price,
        }
    }

    /// Extract availability and price from a JSON-LD value, trying multiple known structures
    fn extract_availability_and_price(
        json: &serde_json::Value,
        variant_id: Option<&str>,
    ) -> Option<(String, PriceInfo)> {
        Self::try_extract_from_product_with_price(json)
            .or_else(|| Self::try_extract_from_product_group_with_price(json, variant_id))
            .or_else(|| Self::try_extract_from_graph_with_price(json, variant_id))
            .or_else(|| Self::try_extract_from_array_with_price(json, variant_id))
    }

    /// Try to extract availability and price if the JSON is a Product type
    fn try_extract_from_product_with_price(
        json: &serde_json::Value,
    ) -> Option<(String, PriceInfo)> {
        if Self::is_product_type(json) {
            Self::get_availability_and_price_from_product(json)
        } else {
            None
        }
    }

    /// Try to extract availability and price if the JSON is a ProductGroup type
    fn try_extract_from_product_group_with_price(
        json: &serde_json::Value,
        variant_id: Option<&str>,
    ) -> Option<(String, PriceInfo)> {
        if Self::is_product_group_type(json) {
            Self::get_availability_and_price_from_product_group(json, variant_id)
        } else {
            None
        }
    }

    /// Try to extract availability and price from a @graph array
    fn try_extract_from_graph_with_price(
        json: &serde_json::Value,
        variant_id: Option<&str>,
    ) -> Option<(String, PriceInfo)> {
        json.get("@graph")
            .and_then(|g| g.as_array())
            .and_then(|arr| Self::find_availability_and_price_in_items(arr, variant_id))
    }

    /// Try to extract availability and price from a direct JSON array
    fn try_extract_from_array_with_price(
        json: &serde_json::Value,
        variant_id: Option<&str>,
    ) -> Option<(String, PriceInfo)> {
        json.as_array()
            .and_then(|arr| Self::find_availability_and_price_in_items(arr, variant_id))
    }

    /// Iterate through items looking for availability and price data
    fn find_availability_and_price_in_items(
        items: &[serde_json::Value],
        variant_id: Option<&str>,
    ) -> Option<(String, PriceInfo)> {
        items.iter().find_map(|item| {
            Self::try_extract_from_product_with_price(item)
                .or_else(|| Self::try_extract_from_product_group_with_price(item, variant_id))
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

    /// Get availability and price from a ProductGroup by matching variant ID
    fn get_availability_and_price_from_product_group(
        product_group: &serde_json::Value,
        variant_id: Option<&str>,
    ) -> Option<(String, PriceInfo)> {
        let variants = product_group.get("hasVariant")?.as_array()?;

        let Some(vid) = variant_id else {
            // No variant ID specified, return first variant's availability and price
            return Self::get_first_variant_availability(variants);
        };

        // Try to find the matching variant by ID
        let matched = Self::find_variant_by_id(variants, vid);
        if matched.is_some() {
            return matched;
        }

        // Fallback: return first variant's availability and price
        Self::get_first_variant_availability(variants)
    }

    /// Find a variant by its ID in the URL query parameters
    fn find_variant_by_id(
        variants: &[serde_json::Value],
        vid: &str,
    ) -> Option<(String, PriceInfo)> {
        // Dummy base for resolving relative URLs (host is irrelevant)
        let base = Url::parse("http://localhost").unwrap();

        for variant in variants {
            let id = variant.get("@id").and_then(|i| i.as_str())?;
            let parsed_url = Url::parse(id).or_else(|_| base.join(id)).ok()?;

            let matches_variant = parsed_url
                .query_pairs()
                .any(|(key, value)| key == "variant" && value == vid);

            if !matches_variant {
                continue;
            }

            if let Some(result) = Self::get_availability_and_price_from_product(variant) {
                return Some(result);
            }
        }

        None
    }

    /// Get the first variant's availability and price
    fn get_first_variant_availability(
        variants: &[serde_json::Value],
    ) -> Option<(String, PriceInfo)> {
        variants
            .iter()
            .find_map(Self::get_availability_and_price_from_product)
    }

    /// Get availability and price from a Product JSON object
    fn get_availability_and_price_from_product(
        product: &serde_json::Value,
    ) -> Option<(String, PriceInfo)> {
        let offers = product.get("offers")?;

        // Single offer object
        if let Some(avail) = offers.get("availability").and_then(|a| a.as_str()) {
            let price = Self::get_price_from_offer(offers);
            return Some((avail.to_string(), price));
        }

        // Array of offers - use first one with availability
        offers.as_array().and_then(|arr| {
            arr.iter().find_map(|offer| {
                let avail = offer.get("availability")?.as_str()?;
                let price = Self::get_price_from_offer(offer);
                Some((avail.to_string(), price))
            })
        })
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

    // Price parsing tests

    #[test]
    fn test_parse_price_to_cents_simple() {
        assert_eq!(ScraperService::parse_price_to_cents("789.00"), Some(78900));
        assert_eq!(ScraperService::parse_price_to_cents("99.99"), Some(9999));
        assert_eq!(ScraperService::parse_price_to_cents("49.99"), Some(4999));
    }

    #[test]
    fn test_parse_price_to_cents_with_thousands() {
        assert_eq!(
            ScraperService::parse_price_to_cents("1,234.56"),
            Some(123456)
        );
        assert_eq!(
            ScraperService::parse_price_to_cents("10,000.00"),
            Some(1000000)
        );
    }

    #[test]
    fn test_parse_price_to_cents_no_decimals() {
        assert_eq!(ScraperService::parse_price_to_cents("789"), Some(78900));
        assert_eq!(ScraperService::parse_price_to_cents("100"), Some(10000));
    }

    #[test]
    fn test_parse_price_to_cents_single_decimal() {
        assert_eq!(ScraperService::parse_price_to_cents("789.9"), Some(78990));
        assert_eq!(ScraperService::parse_price_to_cents("99.5"), Some(9950));
    }

    #[test]
    fn test_parse_price_to_cents_with_currency_symbol() {
        assert_eq!(ScraperService::parse_price_to_cents("$789.00"), Some(78900));
        assert_eq!(ScraperService::parse_price_to_cents("€99.99"), Some(9999));
    }

    #[test]
    fn test_parse_price_to_cents_empty() {
        assert_eq!(ScraperService::parse_price_to_cents(""), None);
        assert_eq!(ScraperService::parse_price_to_cents("   "), None);
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
