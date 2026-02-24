use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::entities::app_setting::SettingScope;
use crate::error::AppError;
use crate::repositories::{ScopedSettingsReader, SettingsHelpers};

/// Setting keys for global settings
pub mod keys {
    pub const THEME: &str = "theme";
    pub const SHOW_IN_TRAY: &str = "show_in_tray";
    pub const LAUNCH_AT_LOGIN: &str = "launch_at_login";
    pub const ENABLE_LOGGING: &str = "enable_logging";
    pub const LOG_LEVEL: &str = "log_level";
    pub const ENABLE_NOTIFICATIONS: &str = "enable_notifications";
    pub const SIDEBAR_EXPANDED: &str = "sidebar_expanded";
    pub const COLOR_PALETTE: &str = "color_palette";
    pub const DISPLAY_TIMEZONE: &str = "display_timezone";
    pub const DATE_FORMAT: &str = "date_format";
    pub const PREFERRED_CURRENCY: &str = "preferred_currency";
}

/// Default values for settings
pub mod defaults {
    pub const THEME: &str = "system";
    pub const SHOW_IN_TRAY: bool = true;
    pub const LAUNCH_AT_LOGIN: bool = false;
    pub const ENABLE_LOGGING: bool = true;
    pub const LOG_LEVEL: &str = "info";
    pub const ENABLE_NOTIFICATIONS: bool = true;
    pub const SIDEBAR_EXPANDED: bool = true;
    pub const COLOR_PALETTE: &str = "default";
    pub const DISPLAY_TIMEZONE: &str = "auto";
    pub const DATE_FORMAT: &str = "system";
    pub const PREFERRED_CURRENCY: &str = "AUD";
}

/// Settings model returned by the service
///
/// Maintains the same shape as the old SettingModel for backward compatibility
/// with the frontend.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub show_in_tray: bool,
    pub launch_at_login: bool,
    pub enable_logging: bool,
    pub log_level: String,
    pub enable_notifications: bool,
    pub sidebar_expanded: bool,
    pub color_palette: String,
    pub display_timezone: String,
    pub date_format: String,
    pub preferred_currency: String,
    pub updated_at: DateTime<Utc>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: defaults::THEME.to_string(),
            show_in_tray: defaults::SHOW_IN_TRAY,
            launch_at_login: defaults::LAUNCH_AT_LOGIN,
            enable_logging: defaults::ENABLE_LOGGING,
            log_level: defaults::LOG_LEVEL.to_string(),
            enable_notifications: defaults::ENABLE_NOTIFICATIONS,
            sidebar_expanded: defaults::SIDEBAR_EXPANDED,
            color_palette: defaults::COLOR_PALETTE.to_string(),
            display_timezone: defaults::DISPLAY_TIMEZONE.to_string(),
            date_format: defaults::DATE_FORMAT.to_string(),
            preferred_currency: defaults::PREFERRED_CURRENCY.to_string(),
            updated_at: Utc::now(),
        }
    }
}

/// Parameters for updating settings (all fields optional for partial updates)
#[derive(Default, Deserialize)]
pub struct UpdateSettingsParams {
    pub theme: Option<String>,
    pub show_in_tray: Option<bool>,
    pub launch_at_login: Option<bool>,
    pub enable_logging: Option<bool>,
    pub log_level: Option<String>,
    pub enable_notifications: Option<bool>,
    pub sidebar_expanded: Option<bool>,
    pub color_palette: Option<String>,
    pub display_timezone: Option<String>,
    pub date_format: Option<String>,
    pub preferred_currency: Option<String>,
}

/// Cached settings for bulk operations.
///
/// Loads settings once to avoid repeated database reads during bulk processing.
/// This struct provides efficient access to both global settings and domain settings
/// during operations that process multiple items (e.g., checking all products).
///
/// # Example
/// ```rust
/// # use product_stalker_core::services::setting_service::SettingsCache;
/// # use product_stalker_core::AppError;
/// # use sea_orm::DatabaseConnection;
/// async fn process_products(conn: &DatabaseConnection) -> Result<(), AppError> {
///     let cache = SettingsCache::load(conn).await?;
///     # let products: Vec<()> = vec![];
///
///     // Settings are loaded once and can be accessed multiple times
///     for product in products {
///         let enabled = cache.enable_notifications();
///         // Process product...
///     }
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct SettingsCache {
    settings: Settings,
    loaded_at: DateTime<Utc>,
}

impl SettingsCache {
    /// Load settings from database into cache
    pub async fn load(conn: &DatabaseConnection) -> Result<Self, AppError> {
        let settings = SettingService::get(conn).await?;

        Ok(Self {
            settings,
            loaded_at: Utc::now(),
        })
    }

    /// Get the current theme setting
    pub fn theme(&self) -> &str {
        &self.settings.theme
    }

    /// Check if notifications are enabled
    pub fn enable_notifications(&self) -> bool {
        self.settings.enable_notifications
    }

    /// Check if logging is enabled
    pub fn enable_logging(&self) -> bool {
        self.settings.enable_logging
    }

    /// Get the log level
    pub fn log_level(&self) -> &str {
        &self.settings.log_level
    }

    /// Get the preferred currency
    pub fn preferred_currency(&self) -> &str {
        &self.settings.preferred_currency
    }

    /// Get when these settings were loaded
    pub fn loaded_at(&self) -> DateTime<Utc> {
        self.loaded_at
    }

    /// Get the full settings struct
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

/// Service layer for settings business logic
///
/// Validates inputs and orchestrates EAV repository calls.
pub struct SettingService;

impl SettingService {
    /// Get current settings, reading from EAV storage with defaults
    pub async fn get(conn: &DatabaseConnection) -> Result<Settings, AppError> {
        let scope = SettingScope::Global;
        let r = ScopedSettingsReader::new(conn, &scope);

        Ok(Settings {
            theme: r.string(keys::THEME, defaults::THEME).await?,
            show_in_tray: r.bool(keys::SHOW_IN_TRAY, defaults::SHOW_IN_TRAY).await?,
            launch_at_login: r
                .bool(keys::LAUNCH_AT_LOGIN, defaults::LAUNCH_AT_LOGIN)
                .await?,
            enable_logging: r
                .bool(keys::ENABLE_LOGGING, defaults::ENABLE_LOGGING)
                .await?,
            log_level: r.string(keys::LOG_LEVEL, defaults::LOG_LEVEL).await?,
            enable_notifications: r
                .bool(keys::ENABLE_NOTIFICATIONS, defaults::ENABLE_NOTIFICATIONS)
                .await?,
            sidebar_expanded: r
                .bool(keys::SIDEBAR_EXPANDED, defaults::SIDEBAR_EXPANDED)
                .await?,
            color_palette: r
                .string(keys::COLOR_PALETTE, defaults::COLOR_PALETTE)
                .await?,
            display_timezone: r
                .string(keys::DISPLAY_TIMEZONE, defaults::DISPLAY_TIMEZONE)
                .await?,
            date_format: r.string(keys::DATE_FORMAT, defaults::DATE_FORMAT).await?,
            preferred_currency: r
                .string(keys::PREFERRED_CURRENCY, defaults::PREFERRED_CURRENCY)
                .await?,
            updated_at: Utc::now(),
        })
    }

    /// Update settings with validation
    ///
    /// Each provided field is persisted independently. Validation runs upfront
    /// so no writes occur if any value is invalid.
    pub async fn update(
        conn: &DatabaseConnection,
        params: UpdateSettingsParams,
    ) -> Result<Settings, AppError> {
        // Validate before touching the database
        if let Some(ref theme) = params.theme {
            Self::validate_theme(theme)?;
        }
        if let Some(ref level) = params.log_level {
            Self::validate_log_level(level)?;
        }
        if let Some(ref palette) = params.color_palette {
            Self::validate_color_palette(palette)?;
        }
        if let Some(ref timezone) = params.display_timezone {
            Self::validate_display_timezone(timezone)?;
        }
        if let Some(ref format) = params.date_format {
            Self::validate_date_format(format)?;
        }
        if let Some(ref currency) = params.preferred_currency {
            Self::validate_preferred_currency(currency)?;
        }

        let scope = SettingScope::Global;

        // Appearance
        Self::persist_optional_string(conn, &scope, keys::THEME, params.theme).await?;
        Self::persist_optional_string(conn, &scope, keys::COLOR_PALETTE, params.color_palette)
            .await?;
        Self::persist_optional_bool(
            conn,
            &scope,
            keys::SIDEBAR_EXPANDED,
            params.sidebar_expanded,
        )
        .await?;

        // Startup
        Self::persist_optional_bool(conn, &scope, keys::SHOW_IN_TRAY, params.show_in_tray).await?;
        Self::persist_optional_bool(conn, &scope, keys::LAUNCH_AT_LOGIN, params.launch_at_login)
            .await?;

        // Logging
        Self::persist_optional_bool(conn, &scope, keys::ENABLE_LOGGING, params.enable_logging)
            .await?;
        Self::persist_optional_string(conn, &scope, keys::LOG_LEVEL, params.log_level).await?;

        // Features
        Self::persist_optional_bool(
            conn,
            &scope,
            keys::ENABLE_NOTIFICATIONS,
            params.enable_notifications,
        )
        .await?;

        // Display
        Self::persist_optional_string(
            conn,
            &scope,
            keys::DISPLAY_TIMEZONE,
            params.display_timezone,
        )
        .await?;
        Self::persist_optional_string(conn, &scope, keys::DATE_FORMAT, params.date_format).await?;

        // Currency
        Self::persist_optional_string(
            conn,
            &scope,
            keys::PREFERRED_CURRENCY,
            params.preferred_currency,
        )
        .await?;

        Self::get(conn).await
    }

    /// Persist an optional string setting (no-op if `None`)
    async fn persist_optional_string(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
        value: Option<String>,
    ) -> Result<(), AppError> {
        if let Some(v) = value {
            SettingsHelpers::set_string(conn, scope, key, &v).await?;
        }
        Ok(())
    }

    /// Persist an optional bool setting (no-op if `None`)
    async fn persist_optional_bool(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
        value: Option<bool>,
    ) -> Result<(), AppError> {
        if let Some(v) = value {
            SettingsHelpers::set_bool(conn, scope, key, v).await?;
        }
        Ok(())
    }

    fn validate_theme(theme: &str) -> Result<(), AppError> {
        match theme {
            "light" | "dark" | "system" => Ok(()),
            _ => Err(AppError::Validation(format!(
                "Invalid theme: {}. Must be 'light', 'dark', or 'system'",
                theme
            ))),
        }
    }

    fn validate_log_level(level: &str) -> Result<(), AppError> {
        match level {
            "error" | "warn" | "info" | "debug" | "trace" => Ok(()),
            _ => Err(AppError::Validation(format!(
                "Invalid log level: {}. Must be 'error', 'warn', 'info', 'debug', or 'trace'",
                level
            ))),
        }
    }

    fn validate_color_palette(palette: &str) -> Result<(), AppError> {
        match palette {
            "default" | "ocean" | "rose" => Ok(()),
            _ => Err(AppError::Validation(format!(
                "Invalid color palette: {}. Must be 'default', 'ocean', or 'rose'",
                palette
            ))),
        }
    }

    fn validate_display_timezone(timezone: &str) -> Result<(), AppError> {
        match timezone {
            "auto"
            | "America/New_York"
            | "America/Chicago"
            | "America/Denver"
            | "America/Los_Angeles"
            | "America/Anchorage"
            | "Pacific/Honolulu"
            | "America/Toronto"
            | "America/Vancouver"
            | "America/Mexico_City"
            | "America/Sao_Paulo"
            | "America/Buenos_Aires"
            | "Europe/London"
            | "Europe/Paris"
            | "Europe/Berlin"
            | "Europe/Madrid"
            | "Europe/Rome"
            | "Europe/Amsterdam"
            | "Europe/Stockholm"
            | "Europe/Moscow"
            | "Asia/Dubai"
            | "Asia/Kolkata"
            | "Asia/Shanghai"
            | "Asia/Tokyo"
            | "Asia/Seoul"
            | "Asia/Singapore"
            | "Asia/Hong_Kong"
            | "Australia/Sydney"
            | "Australia/Melbourne"
            | "Pacific/Auckland" => Ok(()),
            _ => Err(AppError::Validation(format!(
                "Invalid timezone: {}. Must be 'auto' or a supported IANA timezone",
                timezone
            ))),
        }
    }

    fn validate_preferred_currency(currency: &str) -> Result<(), AppError> {
        match currency {
            "USD" | "EUR" | "GBP" | "JPY" | "AUD" | "CAD" | "NZD" | "CHF" | "CNY" | "HKD"
            | "SGD" | "SEK" | "NOK" | "DKK" | "KRW" | "INR" | "BRL" | "ZAR" | "MXN" | "TWD"
            | "THB" | "MYR" | "PHP" | "IDR" | "PLN" | "CZK" | "HUF" | "ILS" | "TRY" | "AED" => {
                Ok(())
            }
            _ => Err(AppError::Validation(format!(
                "Invalid currency: {}. Must be a supported ISO 4217 code",
                currency
            ))),
        }
    }

    fn validate_date_format(format: &str) -> Result<(), AppError> {
        match format {
            "system" | "MM/DD/YYYY" | "DD/MM/YYYY" | "YYYY-MM-DD" => Ok(()),
            _ => Err(AppError::Validation(format!(
                "Invalid date format: {}. Must be 'system', 'MM/DD/YYYY', 'DD/MM/YYYY', or 'YYYY-MM-DD'",
                format
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_theme_accepts_light() {
        assert!(SettingService::validate_theme("light").is_ok());
    }

    #[test]
    fn test_validate_theme_accepts_dark() {
        assert!(SettingService::validate_theme("dark").is_ok());
    }

    #[test]
    fn test_validate_theme_accepts_system() {
        assert!(SettingService::validate_theme("system").is_ok());
    }

    #[test]
    fn test_validate_theme_rejects_invalid_value() {
        let result = SettingService::validate_theme("blue");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_log_level_accepts_all_valid_levels() {
        assert!(SettingService::validate_log_level("error").is_ok());
        assert!(SettingService::validate_log_level("warn").is_ok());
        assert!(SettingService::validate_log_level("info").is_ok());
        assert!(SettingService::validate_log_level("debug").is_ok());
        assert!(SettingService::validate_log_level("trace").is_ok());
    }

    #[test]
    fn test_validate_log_level_rejects_invalid_value() {
        let result = SettingService::validate_log_level("verbose");
        assert!(result.is_err());
    }

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.theme, "system");
        assert!(settings.show_in_tray);
        assert!(!settings.launch_at_login);
        assert!(settings.enable_logging);
        assert_eq!(settings.log_level, "info");
        assert!(settings.enable_notifications);
        assert!(settings.sidebar_expanded);
        assert_eq!(settings.color_palette, "default");
        assert_eq!(settings.display_timezone, "auto");
        assert_eq!(settings.date_format, "system");
        assert_eq!(settings.preferred_currency, "AUD");
    }

    #[test]
    fn test_settings_serialize() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"theme\":\"system\""));
        assert!(json.contains("\"show_in_tray\":true"));
        assert!(json.contains("\"launch_at_login\":false"));
        assert!(json.contains("\"enable_logging\":true"));
        assert!(json.contains("\"log_level\":\"info\""));
        assert!(json.contains("\"enable_notifications\":true"));
        assert!(json.contains("\"sidebar_expanded\":true"));
        assert!(json.contains("\"color_palette\":\"default\""));
        assert!(json.contains("\"display_timezone\":\"auto\""));
        assert!(json.contains("\"date_format\":\"system\""));
        assert!(json.contains("\"preferred_currency\":\"AUD\""));
    }

    #[test]
    fn test_validate_preferred_currency_accepts_all_valid_codes() {
        let valid = [
            "USD", "EUR", "GBP", "JPY", "AUD", "CAD", "NZD", "CHF", "CNY", "HKD", "SGD", "SEK",
            "NOK", "DKK", "KRW", "INR", "BRL", "ZAR", "MXN", "TWD", "THB", "MYR", "PHP", "IDR",
            "PLN", "CZK", "HUF", "ILS", "TRY", "AED",
        ];
        for code in valid.iter() {
            assert!(
                SettingService::validate_preferred_currency(code).is_ok(),
                "Currency {} should be valid",
                code
            );
        }
    }

    #[test]
    fn test_validate_preferred_currency_rejects_invalid() {
        assert!(SettingService::validate_preferred_currency("XYZ").is_err());
        assert!(SettingService::validate_preferred_currency("usd").is_err());
        assert!(SettingService::validate_preferred_currency("").is_err());
    }

    #[test]
    fn test_validate_color_palette_accepts_default() {
        assert!(SettingService::validate_color_palette("default").is_ok());
    }

    #[test]
    fn test_validate_color_palette_accepts_ocean() {
        assert!(SettingService::validate_color_palette("ocean").is_ok());
    }

    #[test]
    fn test_validate_color_palette_accepts_rose() {
        assert!(SettingService::validate_color_palette("rose").is_ok());
    }

    #[test]
    fn test_validate_color_palette_rejects_invalid_value() {
        let result = SettingService::validate_color_palette("neon");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_display_timezone_accepts_auto() {
        assert!(SettingService::validate_display_timezone("auto").is_ok());
    }

    #[test]
    fn test_validate_display_timezone_accepts_iana_format() {
        assert!(SettingService::validate_display_timezone("America/New_York").is_ok());
        assert!(SettingService::validate_display_timezone("Europe/London").is_ok());
        assert!(SettingService::validate_display_timezone("Asia/Tokyo").is_ok());
    }

    #[test]
    fn test_validate_display_timezone_rejects_invalid_format() {
        // Simple string without slash
        let result = SettingService::validate_display_timezone("EST");
        assert!(result.is_err());

        // Invalid timezone that looks like IANA but isn't in allowlist
        let result = SettingService::validate_display_timezone("Fake/Timezone");
        assert!(result.is_err());

        // Generic slash pattern
        let result = SettingService::validate_display_timezone("just/slash");
        assert!(result.is_err());

        // Just a slash
        let result = SettingService::validate_display_timezone("/");
        assert!(result.is_err());

        // Real region but fake city
        let result = SettingService::validate_display_timezone("America/Invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_display_timezone_allowlist_complete() {
        // Verify all 31 timezones from frontend TIMEZONE_OPTIONS are accepted
        let valid_timezones = [
            "auto",
            "America/New_York",
            "America/Chicago",
            "America/Denver",
            "America/Los_Angeles",
            "America/Anchorage",
            "Pacific/Honolulu",
            "America/Toronto",
            "America/Vancouver",
            "America/Mexico_City",
            "America/Sao_Paulo",
            "America/Buenos_Aires",
            "Europe/London",
            "Europe/Paris",
            "Europe/Berlin",
            "Europe/Madrid",
            "Europe/Rome",
            "Europe/Amsterdam",
            "Europe/Stockholm",
            "Europe/Moscow",
            "Asia/Dubai",
            "Asia/Kolkata",
            "Asia/Shanghai",
            "Asia/Tokyo",
            "Asia/Seoul",
            "Asia/Singapore",
            "Asia/Hong_Kong",
            "Australia/Sydney",
            "Australia/Melbourne",
            "Pacific/Auckland",
        ];

        for timezone in valid_timezones.iter() {
            assert!(
                SettingService::validate_display_timezone(timezone).is_ok(),
                "Timezone {} should be valid",
                timezone
            );
        }
    }

    #[test]
    fn test_validate_date_format_accepts_system() {
        assert!(SettingService::validate_date_format("system").is_ok());
    }

    #[test]
    fn test_validate_date_format_accepts_all_formats() {
        assert!(SettingService::validate_date_format("MM/DD/YYYY").is_ok());
        assert!(SettingService::validate_date_format("DD/MM/YYYY").is_ok());
        assert!(SettingService::validate_date_format("YYYY-MM-DD").is_ok());
    }

    #[test]
    fn test_validate_date_format_rejects_invalid_format() {
        let result = SettingService::validate_date_format("DD-MM-YYYY");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::test_utils::setup_app_settings_db;

    #[tokio::test]
    async fn test_get_returns_defaults() {
        let conn = setup_app_settings_db().await;
        let result = SettingService::get(&conn).await;

        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.theme, "system");
        assert!(settings.show_in_tray);
        assert!(!settings.launch_at_login);
        assert!(settings.enable_logging);
        assert_eq!(settings.log_level, "info");
        assert!(settings.enable_notifications);
        assert!(settings.sidebar_expanded);
        assert_eq!(settings.color_palette, "default");
        assert_eq!(settings.display_timezone, "auto");
        assert_eq!(settings.date_format, "system");
        assert_eq!(settings.preferred_currency, "AUD");
    }

    #[tokio::test]
    async fn test_get_returns_same_settings() {
        let conn = setup_app_settings_db().await;

        let first = SettingService::get(&conn).await.unwrap();
        let second = SettingService::get(&conn).await.unwrap();

        assert_eq!(first.theme, second.theme);
        assert_eq!(first.show_in_tray, second.show_in_tray);
    }

    #[tokio::test]
    async fn test_update_validates_theme() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            theme: Some("invalid".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_validates_log_level() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            log_level: Some("invalid_level".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_theme_success() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            theme: Some("dark".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().theme, "dark");
    }

    #[tokio::test]
    async fn test_update_log_level_success() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            log_level: Some("debug".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().log_level, "debug");
    }

    #[tokio::test]
    async fn test_update_show_in_tray() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            show_in_tray: Some(false),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert!(!result.unwrap().show_in_tray);
    }

    #[tokio::test]
    async fn test_update_multiple_fields() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            theme: Some("light".to_string()),
            show_in_tray: Some(true),
            launch_at_login: Some(true),
            enable_logging: Some(true),
            log_level: Some("trace".to_string()),
            enable_notifications: Some(true),
            sidebar_expanded: Some(true),
            color_palette: Some("ocean".to_string()),
            display_timezone: Some("Asia/Tokyo".to_string()),
            date_format: Some("YYYY-MM-DD".to_string()),
            preferred_currency: Some("USD".to_string()),
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.theme, "light");
        assert!(settings.show_in_tray);
        assert!(settings.launch_at_login);
        assert!(settings.enable_logging);
        assert_eq!(settings.log_level, "trace");
        assert!(settings.enable_notifications);
        assert!(settings.sidebar_expanded);
        assert_eq!(settings.color_palette, "ocean");
        assert_eq!(settings.display_timezone, "Asia/Tokyo");
        assert_eq!(settings.date_format, "YYYY-MM-DD");
        assert_eq!(settings.preferred_currency, "USD");
    }

    #[tokio::test]
    async fn test_update_no_fields_does_not_error() {
        let conn = setup_app_settings_db().await;
        let result = SettingService::update(&conn, UpdateSettingsParams::default()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_settings_persist_across_calls() {
        let conn = setup_app_settings_db().await;

        // Update theme
        let params = UpdateSettingsParams {
            theme: Some("dark".to_string()),
            ..Default::default()
        };
        SettingService::update(&conn, params).await.unwrap();

        // Get settings and verify theme persisted
        let settings = SettingService::get(&conn).await.unwrap();
        assert_eq!(settings.theme, "dark");
    }

    #[tokio::test]
    async fn test_update_validates_color_palette() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            color_palette: Some("neon".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_color_palette_success() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            color_palette: Some("rose".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().color_palette, "rose");
    }

    #[tokio::test]
    async fn test_color_palette_persists_across_calls() {
        let conn = setup_app_settings_db().await;

        let params = UpdateSettingsParams {
            color_palette: Some("ocean".to_string()),
            ..Default::default()
        };
        SettingService::update(&conn, params).await.unwrap();

        let settings = SettingService::get(&conn).await.unwrap();
        assert_eq!(settings.color_palette, "ocean");
    }

    #[tokio::test]
    async fn test_settings_cache_loads() {
        let conn = setup_app_settings_db().await;

        let cache = SettingsCache::load(&conn).await;
        assert!(cache.is_ok());
    }

    #[tokio::test]
    async fn test_settings_cache_provides_access_to_settings() {
        let conn = setup_app_settings_db().await;

        let cache = SettingsCache::load(&conn).await.unwrap();

        assert_eq!(cache.theme(), "system");
        assert!(cache.enable_notifications());
        assert!(cache.enable_logging());
        assert_eq!(cache.log_level(), "info");
    }

    #[tokio::test]
    async fn test_settings_cache_reflects_updated_values() {
        let conn = setup_app_settings_db().await;

        // Update theme
        let params = UpdateSettingsParams {
            theme: Some("dark".to_string()),
            enable_notifications: Some(false),
            ..Default::default()
        };
        SettingService::update(&conn, params).await.unwrap();

        // Load cache and verify it reflects the updates
        let cache = SettingsCache::load(&conn).await.unwrap();
        assert_eq!(cache.theme(), "dark");
        assert!(!cache.enable_notifications());
    }

    #[tokio::test]
    async fn test_settings_cache_tracks_load_time() {
        let conn = setup_app_settings_db().await;

        let before = chrono::Utc::now();
        let cache = SettingsCache::load(&conn).await.unwrap();
        let after = chrono::Utc::now();

        let loaded_at = cache.loaded_at();
        assert!(loaded_at >= before);
        assert!(loaded_at <= after);
    }

    #[tokio::test]
    async fn test_update_validates_display_timezone() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            display_timezone: Some("EST".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_validates_date_format() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            date_format: Some("invalid_format".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_display_timezone_success() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            display_timezone: Some("America/New_York".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().display_timezone, "America/New_York");
    }

    #[tokio::test]
    async fn test_update_date_format_success() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            date_format: Some("MM/DD/YYYY".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().date_format, "MM/DD/YYYY");
    }

    #[tokio::test]
    async fn test_display_settings_persist_across_calls() {
        let conn = setup_app_settings_db().await;

        let params = UpdateSettingsParams {
            display_timezone: Some("Europe/London".to_string()),
            date_format: Some("DD/MM/YYYY".to_string()),
            ..Default::default()
        };
        SettingService::update(&conn, params).await.unwrap();

        let settings = SettingService::get(&conn).await.unwrap();
        assert_eq!(settings.display_timezone, "Europe/London");
        assert_eq!(settings.date_format, "DD/MM/YYYY");
    }

    #[tokio::test]
    async fn test_update_validates_preferred_currency() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            preferred_currency: Some("XYZ".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_preferred_currency_success() {
        let conn = setup_app_settings_db().await;
        let params = UpdateSettingsParams {
            preferred_currency: Some("USD".to_string()),
            ..Default::default()
        };

        let result = SettingService::update(&conn, params).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().preferred_currency, "USD");
    }

    #[tokio::test]
    async fn test_preferred_currency_persists_across_calls() {
        let conn = setup_app_settings_db().await;

        let params = UpdateSettingsParams {
            preferred_currency: Some("EUR".to_string()),
            ..Default::default()
        };
        SettingService::update(&conn, params).await.unwrap();

        let settings = SettingService::get(&conn).await.unwrap();
        assert_eq!(settings.preferred_currency, "EUR");
    }

    #[tokio::test]
    async fn test_settings_cache_reflects_preferred_currency() {
        let conn = setup_app_settings_db().await;

        let params = UpdateSettingsParams {
            preferred_currency: Some("GBP".to_string()),
            ..Default::default()
        };
        SettingService::update(&conn, params).await.unwrap();

        let cache = SettingsCache::load(&conn).await.unwrap();
        assert_eq!(cache.preferred_currency(), "GBP");
    }
}
