//! HTTP client utilities for fetching web pages with browser-like headers.

use std::time::Duration;

use product_stalker_core::AppError;

use super::bot_detection::is_cloudflare_challenge;
use crate::services::HeadlessService;

/// HTTP request timeout
const TIMEOUT_SECS: u64 = 30;

/// User-Agent header mimicking Chrome browser.
///
/// Using a realistic browser User-Agent helps avoid basic bot detection
/// that blocks requests with obvious automation signatures like "curl" or "python-requests".
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// HTTP Accept header for HTML content
const ACCEPT_HEADER: &str =
    "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8";

/// Sec-Ch-Ua header for Chrome browser fingerprint
const SEC_CH_UA: &str = r#""Not_A Brand";v="8", "Chromium";v="120", "Google Chrome";v="120""#;

/// Error message shown when bot protection is detected and headless browser is disabled.
const BOT_PROTECTION_MESSAGE: &str =
    "This site has bot protection. Enable headless browser in settings to check this site.";

/// Fetch HTML content, falling back to headless browser if bot protection is detected
///
/// Tries HTTP first (fast path). If bot protection is detected (Cloudflare challenge,
/// 403/503 status), falls back to headless browser if enabled.
pub async fn fetch_html_with_fallback(
    url: &str,
    enable_headless: bool,
) -> Result<String, AppError> {
    match fetch_page(url).await {
        Ok(html) => {
            log::debug!("HTTP fetch succeeded for {}, checking for challenge", url);
            if is_cloudflare_challenge(200, &html) {
                log::info!("Detected bot protection challenge for {}", url);
                if enable_headless {
                    log::info!("Attempting headless fallback for {}", url);
                    fetch_with_headless(url).await
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
                fetch_with_headless(&failed_url).await
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
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()?;

    let response = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", ACCEPT_HEADER)
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .header("Sec-Ch-Ua", SEC_CH_UA)
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
