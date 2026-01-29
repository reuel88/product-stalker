use serde::Serialize;
use tauri::{AppHandle, Manager};

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
pub async fn close_splashscreen(app: AppHandle) -> Result<SplashscreenResult, String> {
    let mut result = SplashscreenResult::default();

    if let Some(splash) = app.get_webview_window(window_labels::SPLASH) {
        splash.close().map_err(|e| e.to_string())?;
        result.splash_closed = true;
    }
    if let Some(main) = app.get_webview_window(window_labels::MAIN) {
        main.show().map_err(|e| e.to_string())?;
        main.set_focus().map_err(|e| e.to_string())?;
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

    // Window labels tests

    #[test]
    fn test_splash_label() {
        assert_eq!(window_labels::SPLASH, "splash");
    }

    #[test]
    fn test_main_label() {
        assert_eq!(window_labels::MAIN, "main");
    }

    // SplashscreenResult construction tests

    #[test]
    fn test_splashscreen_result_new() {
        let result = SplashscreenResult::new(true, false);
        assert!(result.splash_closed);
        assert!(!result.main_shown);
    }

    #[test]
    fn test_splashscreen_result_success() {
        let result = SplashscreenResult::success();
        assert!(result.splash_closed);
        assert!(result.main_shown);
    }

    #[test]
    fn test_splashscreen_result_no_windows() {
        let result = SplashscreenResult::no_windows();
        assert!(!result.splash_closed);
        assert!(!result.main_shown);
    }

    #[test]
    fn test_splashscreen_result_default() {
        let result = SplashscreenResult::default();
        assert!(!result.splash_closed);
        assert!(!result.main_shown);
    }

    #[test]
    fn test_splashscreen_result_default_equals_no_windows() {
        assert_eq!(
            SplashscreenResult::default(),
            SplashscreenResult::no_windows()
        );
    }

    // State check tests

    #[test]
    fn test_is_complete_true() {
        let result = SplashscreenResult::success();
        assert!(result.is_complete());
    }

    #[test]
    fn test_is_complete_false_no_splash() {
        let result = SplashscreenResult::new(false, true);
        assert!(!result.is_complete());
    }

    #[test]
    fn test_is_complete_false_no_main() {
        let result = SplashscreenResult::new(true, false);
        assert!(!result.is_complete());
    }

    #[test]
    fn test_is_complete_false_neither() {
        let result = SplashscreenResult::no_windows();
        assert!(!result.is_complete());
    }

    #[test]
    fn test_main_visible_true() {
        let result = SplashscreenResult::new(false, true);
        assert!(result.main_visible());
    }

    #[test]
    fn test_main_visible_false() {
        let result = SplashscreenResult::new(true, false);
        assert!(!result.main_visible());
    }

    // Trait tests

    #[test]
    fn test_splashscreen_result_clone() {
        let result = SplashscreenResult::success();
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    #[test]
    fn test_splashscreen_result_debug() {
        let result = SplashscreenResult::success();
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("SplashscreenResult"));
        assert!(debug_str.contains("splash_closed"));
        assert!(debug_str.contains("main_shown"));
    }

    #[test]
    fn test_splashscreen_result_partial_eq() {
        let result1 = SplashscreenResult::new(true, true);
        let result2 = SplashscreenResult::new(true, true);
        let result3 = SplashscreenResult::new(true, false);

        assert_eq!(result1, result2);
        assert_ne!(result1, result3);
    }

    // Serialization tests

    #[test]
    fn test_serialize_success() {
        let result = SplashscreenResult::success();
        let json = serde_json::to_string(&result).unwrap();

        assert!(json.contains(r#""splash_closed":true"#));
        assert!(json.contains(r#""main_shown":true"#));
    }

    #[test]
    fn test_serialize_no_windows() {
        let result = SplashscreenResult::no_windows();
        let json = serde_json::to_string(&result).unwrap();

        assert!(json.contains(r#""splash_closed":false"#));
        assert!(json.contains(r#""main_shown":false"#));
    }

    #[test]
    fn test_serialize_partial() {
        let result = SplashscreenResult::new(true, false);
        let json = serde_json::to_string(&result).unwrap();

        assert!(json.contains(r#""splash_closed":true"#));
        assert!(json.contains(r#""main_shown":false"#));
    }

    // All state combinations

    #[test]
    fn test_all_state_combinations() {
        let states = [(false, false), (false, true), (true, false), (true, true)];

        for (splash, main) in states {
            let result = SplashscreenResult::new(splash, main);
            assert_eq!(result.splash_closed, splash);
            assert_eq!(result.main_shown, main);
            assert_eq!(result.is_complete(), splash && main);
            assert_eq!(result.main_visible(), main);
        }
    }
}
