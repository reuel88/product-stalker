use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use headless_chrome::{Browser, LaunchOptions};

use crate::error::AppError;

/// Service for headless browser automation
///
/// Used as a fallback when HTTP requests are blocked by bot protection
/// (Cloudflare, etc.). Requires Chrome/Chromium to be installed.
pub struct HeadlessService {
    browser: Option<Arc<Browser>>,
}

impl HeadlessService {
    /// Page load timeout for headless browser (longer than HTTP due to JS execution)
    const PAGE_TIMEOUT_SECS: u64 = 60;

    /// Create a new headless service instance
    pub fn new() -> Self {
        Self { browser: None }
    }

    /// Fetch a page using headless Chrome
    ///
    /// Lazily initializes the browser on first use. Falls back to clear
    /// error messages if Chrome is not found.
    pub fn fetch_page(&mut self, url: &str) -> Result<String, AppError> {
        log::info!("Headless: starting fetch for {}", url);

        // Initialize browser if not already done
        if self.browser.is_none() {
            log::debug!("Headless: launching browser");
            self.browser = Some(self.launch_browser()?);
            log::info!("Headless: browser launched successfully");
        }

        let browser = self.browser.as_ref().unwrap();

        // Create a new tab for this request
        log::debug!("Headless: creating new tab");
        let tab = browser
            .new_tab()
            .map_err(|e| AppError::Internal(format!("Failed to create browser tab: {}", e)))?;

        // Inject script to hide webdriver property before navigation
        // This helps bypass Cloudflare's navigator.webdriver detection
        log::debug!("Headless: injecting anti-detection script");
        let _ = tab.evaluate(
            r#"
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined
            });
            "#,
            false,
        );

        // Navigate to the URL
        log::debug!("Headless: navigating to {}", url);
        tab.navigate_to(url)
            .map_err(|e| AppError::Scraping(format!("Failed to navigate to URL: {}", e)))?;

        // Wait for page to load (DOMContentLoaded + network idle)
        log::debug!("Headless: waiting for navigation to complete");
        tab.wait_until_navigated()
            .map_err(|e| AppError::Scraping(format!("Page load timeout: {}", e)))?;

        // Re-inject script after navigation in case page reset it
        let _ = tab.evaluate(
            r#"
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined
            });
            "#,
            false,
        );

        // Get the page HTML
        log::debug!("Headless: getting page content");
        let html = tab
            .get_content()
            .map_err(|e| AppError::Scraping(format!("Failed to get page content: {}", e)))?;

        log::info!(
            "Headless: successfully fetched {} bytes from {}",
            html.len(),
            url
        );

        // Check if we hit a CAPTCHA
        if Self::is_captcha_challenge(&html) {
            log::warn!("Headless: CAPTCHA detected for {}", url);
            return Err(AppError::BotProtection(
                "This site requires manual verification (CAPTCHA). Please check the product page directly.".to_string()
            ));
        }

        Ok(html)
    }

    /// Launch Chrome browser with appropriate options
    fn launch_browser(&self) -> Result<Arc<Browser>, AppError> {
        let chrome_path = Self::find_chrome_binary().ok_or_else(|| {
            AppError::BotProtection(
                "Chrome/Chromium not found. This site has bot protection and requires Chrome to check availability. \
                Please install Chrome from https://www.google.com/chrome/".to_string()
            )
        })?;

        // Use headless: false and manually add --headless=new for Chrome's new headless mode
        // The new headless mode is much harder for bot detection to fingerprint
        let options = LaunchOptions::default_builder()
            .path(Some(chrome_path))
            .headless(false) // We'll add --headless=new manually
            .sandbox(true)
            .idle_browser_timeout(Duration::from_secs(Self::PAGE_TIMEOUT_SECS))
            // Anti-detection arguments to bypass Cloudflare fingerprinting
            .args(vec![
                // Use Chrome's new headless mode (less detectable than old --headless)
                std::ffi::OsStr::new("--headless=new"),
                // Disable automation flags that reveal headless mode
                std::ffi::OsStr::new("--disable-blink-features=AutomationControlled"),
                // Use a realistic window size
                std::ffi::OsStr::new("--window-size=1920,1080"),
                // Disable infobars
                std::ffi::OsStr::new("--disable-infobars"),
                // Disable extensions
                std::ffi::OsStr::new("--disable-extensions"),
                // Disable GPU (more stable in headless)
                std::ffi::OsStr::new("--disable-gpu"),
                // No sandbox warning
                std::ffi::OsStr::new("--no-first-run"),
                // Disable popup blocking
                std::ffi::OsStr::new("--disable-popup-blocking"),
                // Set a realistic user agent
                std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
            ])
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create browser options: {}", e)))?;

        let browser = Browser::new(options)
            .map_err(|e| AppError::Internal(format!("Failed to launch Chrome: {}", e)))?;

        Ok(Arc::new(browser))
    }

    /// Find Chrome/Chromium binary on the system
    ///
    /// Checks:
    /// 1. CHROME_PATH environment variable
    /// 2. Platform-specific common installation paths
    pub fn find_chrome_binary() -> Option<PathBuf> {
        // Check environment variable first
        if let Ok(path) = std::env::var("CHROME_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        // Platform-specific paths
        #[cfg(target_os = "windows")]
        {
            Self::find_chrome_windows()
        }

        #[cfg(target_os = "macos")]
        {
            Self::find_chrome_macos()
        }

        #[cfg(target_os = "linux")]
        {
            Self::find_chrome_linux()
        }
    }

    #[cfg(target_os = "windows")]
    fn find_chrome_windows() -> Option<PathBuf> {
        let paths = [
            // Chrome
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            // Chromium
            r"C:\Program Files\Chromium\Application\chrome.exe",
            // Edge (Chromium-based)
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        ];

        // Also check user's local AppData
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let user_chrome = PathBuf::from(&local_app_data)
                .join("Google")
                .join("Chrome")
                .join("Application")
                .join("chrome.exe");
            if user_chrome.exists() {
                return Some(user_chrome);
            }
        }

        paths.iter().map(PathBuf::from).find(|p| p.exists())
    }

    #[cfg(target_os = "macos")]
    fn find_chrome_macos() -> Option<PathBuf> {
        let paths = [
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        ];

        // Also check user's Applications folder
        if let Ok(home) = std::env::var("HOME") {
            let user_chrome = PathBuf::from(&home)
                .join("Applications")
                .join("Google Chrome.app")
                .join("Contents")
                .join("MacOS")
                .join("Google Chrome");
            if user_chrome.exists() {
                return Some(user_chrome);
            }
        }

        paths.iter().map(PathBuf::from).find(|p| p.exists())
    }

    #[cfg(target_os = "linux")]
    fn find_chrome_linux() -> Option<PathBuf> {
        let paths = [
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/snap/bin/chromium",
        ];

        paths.iter().map(PathBuf::from).find(|p| p.exists())
    }

    /// Check if the HTML contains a CAPTCHA challenge
    ///
    /// Returns true if the page requires human verification that
    /// cannot be bypassed by headless browser.
    fn is_captcha_challenge(html: &str) -> bool {
        let captcha_indicators = [
            "g-recaptcha",
            "h-captcha",
            "cf-turnstile",
            "captcha-container",
            "recaptcha-token",
            "hcaptcha-response",
        ];

        let html_lower = html.to_lowercase();
        captcha_indicators
            .iter()
            .any(|indicator| html_lower.contains(indicator))
    }

    /// Check if Chrome is available on this system
    #[allow(dead_code)]
    pub fn is_chrome_available() -> bool {
        Self::find_chrome_binary().is_some()
    }
}

impl Default for HeadlessService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_instance() {
        let service = HeadlessService::new();
        assert!(service.browser.is_none());
    }

    #[test]
    fn test_default_creates_instance() {
        let service = HeadlessService::default();
        assert!(service.browser.is_none());
    }

    #[test]
    fn test_is_captcha_challenge_detects_recaptcha() {
        let html = r#"<div class="g-recaptcha" data-sitekey="abc"></div>"#;
        assert!(HeadlessService::is_captcha_challenge(html));
    }

    #[test]
    fn test_is_captcha_challenge_detects_hcaptcha() {
        let html = r#"<div class="h-captcha" data-sitekey="abc"></div>"#;
        assert!(HeadlessService::is_captcha_challenge(html));
    }

    #[test]
    fn test_is_captcha_challenge_detects_turnstile() {
        let html = r#"<div class="cf-turnstile"></div>"#;
        assert!(HeadlessService::is_captcha_challenge(html));
    }

    #[test]
    fn test_is_captcha_challenge_case_insensitive() {
        let html = r#"<div class="G-RECAPTCHA"></div>"#;
        assert!(HeadlessService::is_captcha_challenge(html));
    }

    #[test]
    fn test_is_captcha_challenge_returns_false_for_normal_page() {
        let html = r#"
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
        assert!(!HeadlessService::is_captcha_challenge(html));
    }

    #[test]
    fn test_find_chrome_binary_returns_option() {
        // This test just verifies the function runs without panic
        // The result depends on whether Chrome is installed
        let _result = HeadlessService::find_chrome_binary();
    }

    #[test]
    fn test_is_chrome_available_returns_bool() {
        // This test just verifies the function runs without panic
        let _result = HeadlessService::is_chrome_available();
    }
}
