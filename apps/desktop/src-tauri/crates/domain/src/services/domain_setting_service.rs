//! Domain-specific settings management.
//!
//! Contains settings that are specific to this product domain
//! (availability checking, headless browser, etc.) and would be
//! removed when creating a new project from the template.

use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use product_stalker_core::entities::app_setting::SettingScope;
use product_stalker_core::repositories::{ScopedSettingsReader, SettingsHelpers};
use product_stalker_core::AppError;

/// Setting keys for domain-specific settings
pub mod keys {
    pub const BACKGROUND_CHECK_ENABLED: &str = "background_check_enabled";
    pub const BACKGROUND_CHECK_INTERVAL_MINUTES: &str = "background_check_interval_minutes";
    pub const ENABLE_HEADLESS_BROWSER: &str = "enable_headless_browser";
    pub const ALLOW_MANUAL_VERIFICATION: &str = "allow_manual_verification";
    pub const SESSION_CACHE_DURATION_DAYS: &str = "session_cache_duration_days";
}

/// Default values for domain-specific settings
pub mod defaults {
    pub const BACKGROUND_CHECK_ENABLED: bool = false;
    pub const BACKGROUND_CHECK_INTERVAL_MINUTES: i32 = 60;
    pub const ENABLE_HEADLESS_BROWSER: bool = true;
    pub const ALLOW_MANUAL_VERIFICATION: bool = false;
    pub const SESSION_CACHE_DURATION_DAYS: i32 = 14;
}

/// Domain-specific settings
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainSettings {
    pub background_check_enabled: bool,
    pub background_check_interval_minutes: i32,
    pub enable_headless_browser: bool,
    pub allow_manual_verification: bool,
    pub session_cache_duration_days: i32,
}

impl Default for DomainSettings {
    fn default() -> Self {
        Self {
            background_check_enabled: defaults::BACKGROUND_CHECK_ENABLED,
            background_check_interval_minutes: defaults::BACKGROUND_CHECK_INTERVAL_MINUTES,
            enable_headless_browser: defaults::ENABLE_HEADLESS_BROWSER,
            allow_manual_verification: defaults::ALLOW_MANUAL_VERIFICATION,
            session_cache_duration_days: defaults::SESSION_CACHE_DURATION_DAYS,
        }
    }
}

/// Parameters for updating domain settings (all fields optional for partial updates)
#[derive(Default, Deserialize)]
pub struct UpdateDomainSettingsParams {
    pub background_check_enabled: Option<bool>,
    pub background_check_interval_minutes: Option<i32>,
    pub enable_headless_browser: Option<bool>,
    pub allow_manual_verification: Option<bool>,
    pub session_cache_duration_days: Option<i32>,
}

/// Cached domain settings for bulk operations.
///
/// Loads domain settings once to avoid repeated database reads during bulk processing.
/// This is particularly useful during background checking operations where the same
/// settings (like enable_headless_browser) are needed for multiple products.
///
/// # Example
/// ```rust
/// # use product_stalker_domain::services::DomainSettingsCache;
/// # use product_stalker_core::AppError;
/// # use sea_orm::DatabaseConnection;
/// async fn check_all_products(conn: &DatabaseConnection) -> Result<(), AppError> {
///     let cache = DomainSettingsCache::load(conn).await?;
///     # let products: Vec<()> = vec![];
///
///     // Settings are loaded once and can be accessed multiple times
///     for product in products {
///         let use_headless = cache.enable_headless_browser();
///         // Check product availability...
///     }
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct DomainSettingsCache {
    settings: DomainSettings,
    loaded_at: DateTime<Utc>,
}

impl DomainSettingsCache {
    /// Load domain settings from database into cache
    pub async fn load(conn: &DatabaseConnection) -> Result<Self, AppError> {
        let settings = DomainSettingService::get(conn).await?;

        Ok(Self {
            settings,
            loaded_at: Utc::now(),
        })
    }

    /// Check if background checking is enabled
    pub fn background_check_enabled(&self) -> bool {
        self.settings.background_check_enabled
    }

    /// Get background check interval in minutes
    pub fn background_check_interval_minutes(&self) -> i32 {
        self.settings.background_check_interval_minutes
    }

    /// Check if headless browser is enabled
    pub fn enable_headless_browser(&self) -> bool {
        self.settings.enable_headless_browser
    }

    /// Check if manual verification is allowed
    pub fn allow_manual_verification(&self) -> bool {
        self.settings.allow_manual_verification
    }

    /// Get session cache duration in days
    pub fn session_cache_duration_days(&self) -> i32 {
        self.settings.session_cache_duration_days
    }

    /// Get when these settings were loaded
    pub fn loaded_at(&self) -> DateTime<Utc> {
        self.loaded_at
    }

    /// Get the full settings struct
    pub fn settings(&self) -> &DomainSettings {
        &self.settings
    }
}

/// Service layer for domain-specific settings
pub struct DomainSettingService;

impl DomainSettingService {
    /// Get current domain settings
    pub async fn get(conn: &DatabaseConnection) -> Result<DomainSettings, AppError> {
        let scope = SettingScope::Global;
        let r = ScopedSettingsReader::new(conn, &scope);

        let mut settings = DomainSettings {
            background_check_enabled: r
                .bool(
                    keys::BACKGROUND_CHECK_ENABLED,
                    defaults::BACKGROUND_CHECK_ENABLED,
                )
                .await?,
            background_check_interval_minutes: r
                .i32(
                    keys::BACKGROUND_CHECK_INTERVAL_MINUTES,
                    defaults::BACKGROUND_CHECK_INTERVAL_MINUTES,
                )
                .await?,
            enable_headless_browser: r
                .bool(
                    keys::ENABLE_HEADLESS_BROWSER,
                    defaults::ENABLE_HEADLESS_BROWSER,
                )
                .await?,
            allow_manual_verification: r
                .bool(
                    keys::ALLOW_MANUAL_VERIFICATION,
                    defaults::ALLOW_MANUAL_VERIFICATION,
                )
                .await?,
            session_cache_duration_days: r
                .i32(
                    keys::SESSION_CACHE_DURATION_DAYS,
                    defaults::SESSION_CACHE_DURATION_DAYS,
                )
                .await?,
        };

        // Clamp interval to valid range in case of direct DB manipulation
        settings.background_check_interval_minutes = settings
            .background_check_interval_minutes
            .clamp(1, Self::MAX_BACKGROUND_CHECK_INTERVAL_MINUTES);

        // Clamp session cache duration to valid range
        settings.session_cache_duration_days = settings.session_cache_duration_days.clamp(
            Self::MIN_SESSION_CACHE_DURATION_DAYS,
            Self::MAX_SESSION_CACHE_DURATION_DAYS,
        );

        Ok(settings)
    }

    /// Update domain settings with validation
    pub async fn update(
        conn: &DatabaseConnection,
        params: UpdateDomainSettingsParams,
    ) -> Result<DomainSettings, AppError> {
        if let Some(interval) = params.background_check_interval_minutes {
            Self::validate_background_check_interval(interval)?;
        }

        if let Some(duration) = params.session_cache_duration_days {
            Self::validate_session_cache_duration(duration)?;
        }

        let scope = SettingScope::Global;

        if let Some(v) = params.background_check_enabled {
            SettingsHelpers::set_bool(conn, &scope, keys::BACKGROUND_CHECK_ENABLED, v).await?;
        }
        if let Some(v) = params.background_check_interval_minutes {
            SettingsHelpers::set_i32(conn, &scope, keys::BACKGROUND_CHECK_INTERVAL_MINUTES, v)
                .await?;
        }
        if let Some(v) = params.enable_headless_browser {
            SettingsHelpers::set_bool(conn, &scope, keys::ENABLE_HEADLESS_BROWSER, v).await?;
        }
        if let Some(v) = params.allow_manual_verification {
            SettingsHelpers::set_bool(conn, &scope, keys::ALLOW_MANUAL_VERIFICATION, v).await?;
        }
        if let Some(v) = params.session_cache_duration_days {
            SettingsHelpers::set_i32(conn, &scope, keys::SESSION_CACHE_DURATION_DAYS, v).await?;
        }

        Self::get(conn).await
    }

    /// Maximum background check interval: 1 week (10080 minutes)
    const MAX_BACKGROUND_CHECK_INTERVAL_MINUTES: i32 = 10080;

    /// Minimum session cache duration: 1 day
    const MIN_SESSION_CACHE_DURATION_DAYS: i32 = 1;

    /// Maximum session cache duration: 90 days
    const MAX_SESSION_CACHE_DURATION_DAYS: i32 = 90;

    fn validate_background_check_interval(interval: i32) -> Result<(), AppError> {
        if interval <= 0 {
            return Err(AppError::Validation(
                "Background check interval must be a positive number of minutes".to_string(),
            ));
        }
        if interval > Self::MAX_BACKGROUND_CHECK_INTERVAL_MINUTES {
            return Err(AppError::Validation(format!(
                "Background check interval cannot exceed {} minutes (1 week)",
                Self::MAX_BACKGROUND_CHECK_INTERVAL_MINUTES
            )));
        }
        Ok(())
    }

    fn validate_session_cache_duration(duration: i32) -> Result<(), AppError> {
        if duration < Self::MIN_SESSION_CACHE_DURATION_DAYS {
            return Err(AppError::Validation(format!(
                "Session cache duration must be at least {} day",
                Self::MIN_SESSION_CACHE_DURATION_DAYS
            )));
        }
        if duration > Self::MAX_SESSION_CACHE_DURATION_DAYS {
            return Err(AppError::Validation(format!(
                "Session cache duration cannot exceed {} days",
                Self::MAX_SESSION_CACHE_DURATION_DAYS
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_domain_settings() {
        let settings = DomainSettings::default();
        assert!(!settings.background_check_enabled);
        assert_eq!(settings.background_check_interval_minutes, 60);
        assert!(settings.enable_headless_browser);
        assert!(!settings.allow_manual_verification);
        assert_eq!(settings.session_cache_duration_days, 14);
    }

    #[test]
    fn test_validate_background_check_interval_accepts_positive_values() {
        assert!(DomainSettingService::validate_background_check_interval(15).is_ok());
        assert!(DomainSettingService::validate_background_check_interval(60).is_ok());
        assert!(DomainSettingService::validate_background_check_interval(1440).is_ok());
    }

    #[test]
    fn test_validate_background_check_interval_rejects_zero() {
        assert!(DomainSettingService::validate_background_check_interval(0).is_err());
    }

    #[test]
    fn test_validate_background_check_interval_rejects_negative() {
        assert!(DomainSettingService::validate_background_check_interval(-1).is_err());
    }

    #[test]
    fn test_validate_background_check_interval_rejects_exceeding_max() {
        assert!(DomainSettingService::validate_background_check_interval(10081).is_err());
    }

    #[test]
    fn test_validate_background_check_interval_accepts_max() {
        assert!(DomainSettingService::validate_background_check_interval(10080).is_ok());
    }

    #[test]
    fn test_validate_session_cache_duration_accepts_valid_values() {
        assert!(DomainSettingService::validate_session_cache_duration(1).is_ok());
        assert!(DomainSettingService::validate_session_cache_duration(14).is_ok());
        assert!(DomainSettingService::validate_session_cache_duration(30).is_ok());
        assert!(DomainSettingService::validate_session_cache_duration(90).is_ok());
    }

    #[test]
    fn test_validate_session_cache_duration_rejects_zero() {
        assert!(DomainSettingService::validate_session_cache_duration(0).is_err());
    }

    #[test]
    fn test_validate_session_cache_duration_rejects_negative() {
        assert!(DomainSettingService::validate_session_cache_duration(-1).is_err());
    }

    #[test]
    fn test_validate_session_cache_duration_rejects_exceeding_max() {
        assert!(DomainSettingService::validate_session_cache_duration(91).is_err());
    }

    #[test]
    fn test_domain_settings_serialize() {
        let settings = DomainSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"background_check_enabled\":false"));
        assert!(json.contains("\"background_check_interval_minutes\":60"));
        assert!(json.contains("\"enable_headless_browser\":true"));
        assert!(json.contains("\"allow_manual_verification\":false"));
        assert!(json.contains("\"session_cache_duration_days\":14"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use product_stalker_core::test_utils::setup_app_settings_db;

    #[tokio::test]
    async fn test_get_returns_defaults() {
        let conn = setup_app_settings_db().await;
        let result = DomainSettingService::get(&conn).await;

        assert!(result.is_ok());
        let settings = result.unwrap();
        assert!(!settings.background_check_enabled);
        assert_eq!(settings.background_check_interval_minutes, 60);
        assert!(settings.enable_headless_browser);
        assert!(!settings.allow_manual_verification);
        assert_eq!(settings.session_cache_duration_days, 14);
    }

    #[tokio::test]
    async fn test_update_all_fields() {
        let conn = setup_app_settings_db().await;
        let params = UpdateDomainSettingsParams {
            background_check_enabled: Some(true),
            background_check_interval_minutes: Some(30),
            enable_headless_browser: Some(false),
            allow_manual_verification: None,
            session_cache_duration_days: None,
        };

        let result = DomainSettingService::update(&conn, params).await;
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert!(settings.background_check_enabled);
        assert_eq!(settings.background_check_interval_minutes, 30);
        assert!(!settings.enable_headless_browser);
    }

    #[tokio::test]
    async fn test_update_validates_interval() {
        let conn = setup_app_settings_db().await;
        let params = UpdateDomainSettingsParams {
            background_check_interval_minutes: Some(0),
            ..Default::default()
        };

        let result = DomainSettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_settings_persist_across_calls() {
        let conn = setup_app_settings_db().await;

        let params = UpdateDomainSettingsParams {
            background_check_enabled: Some(true),
            ..Default::default()
        };
        DomainSettingService::update(&conn, params).await.unwrap();

        let settings = DomainSettingService::get(&conn).await.unwrap();
        assert!(settings.background_check_enabled);
    }

    #[tokio::test]
    async fn test_get_clamps_invalid_interval_from_database() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        // Test zero value - should be clamped to minimum of 1
        SettingsHelpers::set_i32(&conn, &scope, keys::BACKGROUND_CHECK_INTERVAL_MINUTES, 0)
            .await
            .unwrap();

        let settings = DomainSettingService::get(&conn).await.unwrap();
        assert_eq!(settings.background_check_interval_minutes, 1);

        // Test negative value - should be clamped to minimum of 1
        SettingsHelpers::set_i32(&conn, &scope, keys::BACKGROUND_CHECK_INTERVAL_MINUTES, -5)
            .await
            .unwrap();

        let settings = DomainSettingService::get(&conn).await.unwrap();
        assert_eq!(settings.background_check_interval_minutes, 1);

        // Test value above maximum - should be clamped to MAX
        SettingsHelpers::set_i32(
            &conn,
            &scope,
            keys::BACKGROUND_CHECK_INTERVAL_MINUTES,
            99999,
        )
        .await
        .unwrap();

        let settings = DomainSettingService::get(&conn).await.unwrap();
        assert_eq!(
            settings.background_check_interval_minutes,
            DomainSettingService::MAX_BACKGROUND_CHECK_INTERVAL_MINUTES
        );
    }

    #[tokio::test]
    async fn test_domain_settings_cache_loads() {
        let conn = setup_app_settings_db().await;

        let cache = DomainSettingsCache::load(&conn).await;
        assert!(cache.is_ok());
    }

    #[tokio::test]
    async fn test_domain_settings_cache_provides_access_to_settings() {
        let conn = setup_app_settings_db().await;

        let cache = DomainSettingsCache::load(&conn).await.unwrap();

        assert!(!cache.background_check_enabled());
        assert_eq!(cache.background_check_interval_minutes(), 60);
        assert!(cache.enable_headless_browser());
        assert!(!cache.allow_manual_verification());
        assert_eq!(cache.session_cache_duration_days(), 14);
    }

    #[tokio::test]
    async fn test_domain_settings_cache_reflects_updated_values() {
        let conn = setup_app_settings_db().await;

        // Update settings
        let params = UpdateDomainSettingsParams {
            background_check_enabled: Some(true),
            background_check_interval_minutes: Some(30),
            enable_headless_browser: Some(false),
            allow_manual_verification: None,
            session_cache_duration_days: None,
        };
        DomainSettingService::update(&conn, params).await.unwrap();

        // Load cache and verify it reflects the updates
        let cache = DomainSettingsCache::load(&conn).await.unwrap();
        assert!(cache.background_check_enabled());
        assert_eq!(cache.background_check_interval_minutes(), 30);
        assert!(!cache.enable_headless_browser());
    }

    #[tokio::test]
    async fn test_domain_settings_cache_tracks_load_time() {
        let conn = setup_app_settings_db().await;

        let before = chrono::Utc::now();
        let cache = DomainSettingsCache::load(&conn).await.unwrap();
        let after = chrono::Utc::now();

        let loaded_at = cache.loaded_at();
        assert!(loaded_at >= before);
        assert!(loaded_at <= after);
    }

    #[tokio::test]
    async fn test_update_manual_verification_settings() {
        let conn = setup_app_settings_db().await;

        let params = UpdateDomainSettingsParams {
            allow_manual_verification: Some(true),
            session_cache_duration_days: Some(30),
            ..Default::default()
        };

        let updated = DomainSettingService::update(&conn, params).await.unwrap();
        assert!(updated.allow_manual_verification);
        assert_eq!(updated.session_cache_duration_days, 30);
    }

    #[tokio::test]
    async fn test_validate_session_cache_duration_min() {
        let conn = setup_app_settings_db().await;

        let params = UpdateDomainSettingsParams {
            session_cache_duration_days: Some(0),
            ..Default::default()
        };

        let result = DomainSettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_session_cache_duration_max() {
        let conn = setup_app_settings_db().await;

        let params = UpdateDomainSettingsParams {
            session_cache_duration_days: Some(91),
            ..Default::default()
        };

        let result = DomainSettingService::update(&conn, params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_clamps_invalid_session_cache_duration_from_database() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        // Test zero value - should be clamped to minimum of 1
        SettingsHelpers::set_i32(&conn, &scope, keys::SESSION_CACHE_DURATION_DAYS, 0)
            .await
            .unwrap();

        let settings = DomainSettingService::get(&conn).await.unwrap();
        assert_eq!(settings.session_cache_duration_days, 1);

        // Test negative value - should be clamped to minimum of 1
        SettingsHelpers::set_i32(&conn, &scope, keys::SESSION_CACHE_DURATION_DAYS, -5)
            .await
            .unwrap();

        let settings = DomainSettingService::get(&conn).await.unwrap();
        assert_eq!(settings.session_cache_duration_days, 1);

        // Test value above maximum - should be clamped to MAX
        SettingsHelpers::set_i32(&conn, &scope, keys::SESSION_CACHE_DURATION_DAYS, 999)
            .await
            .unwrap();

        let settings = DomainSettingService::get(&conn).await.unwrap();
        assert_eq!(
            settings.session_cache_duration_days,
            DomainSettingService::MAX_SESSION_CACHE_DURATION_DAYS
        );
    }
}
