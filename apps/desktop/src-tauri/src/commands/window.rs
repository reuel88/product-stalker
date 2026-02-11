use product_stalker_core::AppError;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::tauri_error::CommandError;

/// Well-known window labels used in the application
pub mod window_labels {
    /// The splash screen window shown during app startup
    pub const SPLASH: &str = "splash";
    /// The main application window
    pub const MAIN: &str = "main";
}

/// Result of a splashscreen close operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SplashscreenResult {
    /// Whether the splash window was found and closed
    pub splash_closed: bool,
    /// Whether the main window was found and shown
    pub main_shown: bool,
}

impl SplashscreenResult {
    /// Create a new result
    pub fn new(splash_closed: bool, main_shown: bool) -> Self {
        Self {
            splash_closed,
            main_shown,
        }
    }

    /// Create a result indicating full success
    #[cfg(test)]
    pub fn success() -> Self {
        Self::new(true, true)
    }

    /// Create a result indicating no windows were found
    pub fn no_windows() -> Self {
        Self::new(false, false)
    }

    /// Check if the transition was fully successful
    #[cfg(test)]
    pub fn is_complete(&self) -> bool {
        self.splash_closed && self.main_shown
    }

    /// Check if at least the main window was shown
    #[cfg(test)]
    pub fn main_visible(&self) -> bool {
        self.main_shown
    }
}

impl Default for SplashscreenResult {
    fn default() -> Self {
        Self::no_windows()
    }
}

#[tauri::command]
pub async fn close_splashscreen(app: AppHandle) -> Result<SplashscreenResult, CommandError> {
    let mut result = SplashscreenResult::default();

    if let Some(splash) = app.get_webview_window(window_labels::SPLASH) {
        splash
            .close()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        result.splash_closed = true;
    }
    if let Some(main) = app.get_webview_window(window_labels::MAIN) {
        main.show().map_err(|e| AppError::Internal(e.to_string()))?;
        main.set_focus()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        result.main_shown = true;
    }

    log::debug!(
        "Splashscreen transition: splash_closed={}, main_shown={}",
        result.splash_closed,
        result.main_shown
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_labels() {
        assert_eq!(window_labels::SPLASH, "splash");
        assert_eq!(window_labels::MAIN, "main");
    }

    #[test]
    fn test_default_equals_no_windows() {
        assert_eq!(
            SplashscreenResult::default(),
            SplashscreenResult::no_windows()
        );
        assert!(!SplashscreenResult::default().splash_closed);
        assert!(!SplashscreenResult::default().main_shown);
    }

    #[test]
    fn test_all_state_combinations() {
        let states = [(false, false), (false, true), (true, false), (true, true)];

        for (splash, main) in states {
            let result = SplashscreenResult::new(splash, main);
            assert_eq!(result.splash_closed, splash);
            assert_eq!(result.main_shown, main);
            assert_eq!(result.is_complete(), splash && main);
            assert_eq!(result.main_visible(), main);

            // Verify clone and equality
            assert_eq!(result.clone(), result);

            // Verify serialization round-trip
            let json = serde_json::to_string(&result).unwrap();
            assert!(json.contains(&format!(r#""splash_closed":{}"#, splash)));
            assert!(json.contains(&format!(r#""main_shown":{}"#, main)));
        }
    }

    #[test]
    fn test_success_constructor() {
        let result = SplashscreenResult::success();
        assert!(result.is_complete());
        assert!(result.main_visible());
    }
}
