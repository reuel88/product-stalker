//! Manual CAPTCHA verification service with visible browser.

use std::path::PathBuf;

use headless_chrome::{Browser, LaunchOptions};
use url::Url;

use product_stalker_core::AppError;

use super::headless_service::HeadlessService;
use super::scraper::USER_AGENT;

/// Service for manual CAPTCHA verification with visible browser
pub struct ManualVerificationService {
    user_data_dir: PathBuf,
}

impl ManualVerificationService {
    pub fn new() -> Self {
        // Reuse the same profile directory as HeadlessService
        let user_data_dir =
            HeadlessService::get_user_data_dir().unwrap_or_else(|_| PathBuf::from("."));

        Self { user_data_dir }
    }

    /// Launch visible browser for manual CAPTCHA solving
    ///
    /// Returns the HTML and cookies JSON after user completes verification
    pub fn launch_visible_browser(&self, url: &str) -> Result<(String, String), AppError> {
        log::info!("ManualVerification: launching visible browser for {}", url);

        let chrome_path = HeadlessService::find_chrome_binary().ok_or_else(|| {
            AppError::External("Chrome/Chromium not found. Please install Chrome.".to_string())
        })?;

        let user_agent_arg = format!("--user-agent={}", USER_AGENT);
        let user_data_arg = format!("--user-data-dir={}", self.user_data_dir.display());

        // Launch VISIBLE browser (headless: false, no --headless flag)
        let options = LaunchOptions::default_builder()
            .path(Some(chrome_path))
            .headless(false) // VISIBLE WINDOW
            .sandbox(true)
            .idle_browser_timeout(std::time::Duration::from_secs(300)) // 5 min timeout
            .args(vec![
                std::ffi::OsStr::new("--disable-blink-features=AutomationControlled"),
                std::ffi::OsStr::new("--window-size=1280,900"),
                std::ffi::OsStr::new("--disable-infobars"),
                std::ffi::OsStr::new(&user_agent_arg),
                std::ffi::OsStr::new(&user_data_arg),
            ])
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create browser options: {}", e)))?;

        let browser = Browser::new(options)
            .map_err(|e| AppError::Internal(format!("Failed to launch Chrome: {}", e)))?;

        let tab = browser
            .new_tab()
            .map_err(|e| AppError::Internal(format!("Failed to create browser tab: {}", e)))?;

        // Navigate to the URL
        tab.navigate_to(url)
            .map_err(|e| AppError::External(format!("Failed to navigate to URL: {}", e)))?;

        // Wait for user to complete verification
        // We'll wait for the page to stabilize (no CAPTCHA elements)
        log::info!("ManualVerification: waiting for user to solve CAPTCHA...");

        // Poll until CAPTCHA is solved (max 5 minutes)
        let max_attempts = 60; // 60 * 5 seconds = 5 minutes
        let mut verified = false;

        for _ in 0..max_attempts {
            std::thread::sleep(std::time::Duration::from_secs(5));

            if let Ok(html) = tab.get_content() {
                if !HeadlessService::is_captcha_challenge(&html) {
                    verified = true;
                    break;
                }
            }
        }

        if !verified {
            return Err(AppError::External(
                "Manual verification timed out. Please try again.".to_string(),
            ));
        }

        log::info!("ManualVerification: CAPTCHA solved, capturing session");

        // Get cookies and serialize them immediately
        let cookies = tab
            .get_cookies()
            .map_err(|e| AppError::Internal(format!("Failed to get cookies: {}", e)))?;

        let cookies_json = serde_json::to_string(&cookies)
            .map_err(|e| AppError::Internal(format!("Failed to serialize cookies: {}", e)))?;

        // Get final page HTML
        let html = tab
            .get_content()
            .map_err(|e| AppError::External(format!("Failed to get page content: {}", e)))?;

        log::info!("ManualVerification: session captured successfully");

        Ok((html, cookies_json))
    }

    /// Extract domain from URL
    pub fn extract_domain(url: &str) -> Result<String, AppError> {
        let parsed =
            Url::parse(url).map_err(|e| AppError::Validation(format!("Invalid URL: {}", e)))?;

        let domain = parsed
            .host_str()
            .ok_or_else(|| AppError::Validation("URL has no host".to_string()))?;

        Ok(domain.to_string())
    }
}

impl Default for ManualVerificationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        let domain = ManualVerificationService::extract_domain("https://example.com/path").unwrap();
        assert_eq!(domain, "example.com");
    }

    #[test]
    fn test_extract_domain_with_subdomain() {
        let domain =
            ManualVerificationService::extract_domain("https://www.example.com/path").unwrap();
        assert_eq!(domain, "www.example.com");
    }
}
