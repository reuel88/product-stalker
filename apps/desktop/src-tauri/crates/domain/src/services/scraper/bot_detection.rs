//! Bot detection and Cloudflare challenge detection utilities.
//!
//! Identifies challenge pages that require headless browser fallback.

/// Check if the response is a Cloudflare challenge page or other bot protection
///
/// Returns true if the response appears to be a bot protection challenge rather
/// than actual page content. Uses multiple detection strategies to minimize
/// false positives while catching common challenge patterns.
///
/// Detection priority (early returns for common cases):
/// 1. Pages with JSON-LD product data are real content -> false
/// 2. Strong bot protection indicators (Cloudflare markers, explicit blocks) -> true
/// 3. 403/503 with minimal content (likely challenge page) -> true
pub fn is_cloudflare_challenge(status: u16, body: &str) -> bool {
    let body_lower = body.to_lowercase();

    // Quick exit: pages with product data (JSON-LD) are real content, not challenges
    if has_product_data(&body_lower) {
        return false;
    }

    // Strong indicators: return early for Cloudflare markers or explicit bot protection
    if has_cloudflare_indicators(&body_lower) || has_explicit_bot_protection(&body_lower) {
        return true;
    }

    // For 403/503 responses: also check for suspiciously minimal content
    let is_bot_blocked_status = status == 403 || status == 503;
    is_bot_blocked_status && is_minimal_challenge_page(&body_lower, body.len())
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(is_cloudflare_challenge(403, body));
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
        assert!(is_cloudflare_challenge(403, body));
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
        assert!(is_cloudflare_challenge(503, body));
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
        assert!(is_cloudflare_challenge(403, body));
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
        assert!(!is_cloudflare_challenge(200, body));
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
        assert!(!is_cloudflare_challenge(403, body));
    }

    #[test]
    fn test_is_cloudflare_challenge_case_insensitive() {
        let body = r#"<html><body>JUST A MOMENT...</body></html>"#;
        assert!(is_cloudflare_challenge(403, body));
    }
}
