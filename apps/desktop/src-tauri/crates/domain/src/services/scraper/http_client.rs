//! HTTP client utilities for fetching web pages with browser-like headers.

use std::time::Duration;

use product_stalker_core::AppError;
use sea_orm::DatabaseConnection;

use super::bot_detection::is_cloudflare_challenge;
use crate::services::{HeadlessService, ManualVerificationService};
use product_stalker_core::repositories::VerifiedSessionRepository;

/// HTTP request timeout
const TIMEOUT_SECS: u64 = 30;

use super::USER_AGENT;

/// HTTP Accept header for HTML content
const ACCEPT_HEADER: &str =
    "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8";

/// Sec-Ch-Ua header for Chrome browser fingerprint
const SEC_CH_UA: &str = r#""Not_A Brand";v="8", "Chromium";v="120", "Google Chrome";v="120""#;

/// Error message shown when bot protection is detected and headless browser is disabled.
const BOT_PROTECTION_MESSAGE: &str =
    "This site has bot protection. Enable headless browser in settings to check this site.";

/// Internal error type for HTTP fetch operations.
///
/// Used within the scraper module to preserve structured error data
/// (e.g., HTTP status codes) for control flow decisions before
/// converting to the generic `AppError::External` at the boundary.
enum FetchPageError {
    /// HTTP client or network error (connection refused, timeout, DNS, TLS)
    Http(String),
    /// HTTP response returned a non-success status code
    HttpStatus { status: u16, url: String },
}

/// Fetch HTML content, falling back to headless browser or manual verification if needed
///
/// Tries HTTP first (fast path). If bot protection is detected (Cloudflare challenge,
/// 403/503 status), falls back to headless browser if enabled. If headless browser
/// encounters a CAPTCHA and manual verification is allowed, launches a visible browser
/// for the user to solve the CAPTCHA manually.
pub async fn fetch_html_with_fallback(
    url: &str,
    enable_headless: bool,
    allow_manual_verification: bool,
    conn: &DatabaseConnection,
    session_cache_duration_days: i32,
) -> Result<String, AppError> {
    let needs_headless = match fetch_page(url).await {
        Ok(html) if !is_cloudflare_challenge(200, &html) => return Ok(html),
        Ok(_) => {
            log::info!("Detected bot protection challenge for {}", url);
            true
        }
        Err(FetchPageError::HttpStatus { status, .. }) if status == 403 || status == 503 => {
            log::info!("HTTP request blocked ({}) for {}", status, url);
            true
        }
        Err(FetchPageError::HttpStatus { status, url }) => {
            let msg = format!("HTTP {} for URL: {}", status, url);
            log::error!("HTTP fetch failed for {}: {}", url, msg);
            return Err(AppError::External(msg));
        }
        Err(FetchPageError::Http(msg)) => {
            log::error!("HTTP fetch failed for {}: {}", url, msg);
            return Err(AppError::External(msg));
        }
    };

    if needs_headless && enable_headless {
        log::info!("Attempting headless fallback for {}", url);
        match fetch_with_headless(url).await {
            Ok(html) => return Ok(html),
            Err(e) => {
                log::warn!("Headless browser failed for {}: {}", url, e);

                // Check if it's a CAPTCHA challenge that requires manual verification
                if let AppError::External(ref msg) = e {
                    if msg.contains("CAPTCHA") || msg.contains("challenge") {
                        if allow_manual_verification {
                            log::info!(
                                "CAPTCHA detected, attempting manual verification for {}",
                                url
                            );
                            return fetch_with_manual_verification(
                                url,
                                conn,
                                session_cache_duration_days,
                            )
                            .await;
                        } else {
                            return Err(AppError::External(
                                "This site requires manual CAPTCHA verification. Enable manual verification in settings.".to_string()
                            ));
                        }
                    }
                }

                return Err(e);
            }
        }
    }

    if allow_manual_verification {
        Err(AppError::External(
            "This site has bot protection. Manual verification is enabled but headless browser must be enabled first.".to_string()
        ))
    } else {
        Err(AppError::External(BOT_PROTECTION_MESSAGE.to_string()))
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

/// Fetch page with manual verification workflow
///
/// Checks for cached verified session first, then launches visible browser if needed.
async fn fetch_with_manual_verification(
    url: &str,
    conn: &DatabaseConnection,
    session_cache_duration_days: i32,
) -> Result<String, AppError> {
    let domain = ManualVerificationService::extract_domain(url)?;

    // Check if we have a cached session
    if let Some(_session) = VerifiedSessionRepository::find_by_domain(conn, &domain).await? {
        log::info!("Using cached verified session for {}", domain);

        // TODO: Use the cached session cookies to fetch the page
        // This would require adding cookie support to the HTTP client
        // For Phase 3 MVP, we'll just re-verify each time (noted in plan as limitation)
    }

    // Launch visible browser for manual verification
    let url_owned = url.to_string();
    let (html, cookies_json) = tokio::task::spawn_blocking(move || {
        let verification_service = ManualVerificationService::new();
        verification_service.launch_visible_browser(&url_owned)
    })
    .await
    .map_err(|e| AppError::Internal(format!("Manual verification task failed: {}", e)))??;

    // Store the verified session
    VerifiedSessionRepository::create(
        conn,
        domain,
        cookies_json,
        USER_AGENT.to_string(),
        session_cache_duration_days,
    )
    .await?;

    log::info!("Verified session stored for future use");

    Ok(html)
}

/// Fetch a page's HTML content using HTTP
async fn fetch_page(url: &str) -> Result<String, FetchPageError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUT_SECS))
        .build()
        .map_err(|e| FetchPageError::Http(e.to_string()))?;

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
        .await
        .map_err(|e| FetchPageError::Http(e.to_string()))?;

    if !response.status().is_success() {
        return Err(FetchPageError::HttpStatus {
            status: response.status().as_u16(),
            url: url.to_string(),
        });
    }

    let html = response
        .text()
        .await
        .map_err(|e| FetchPageError::Http(e.to_string()))?;
    Ok(html)
}
