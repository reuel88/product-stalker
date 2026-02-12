//! Domain-specific settings management.
//!
//! Contains settings that are specific to this product domain
//! (availability checking, headless browser, etc.) and would be
//! removed when creating a new project from the template.

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
}

/// Default values for domain-specific settings
pub mod defaults {
    pub const BACKGROUND_CHECK_ENABLED: bool = false;
    pub const BACKGROUND_CHECK_INTERVAL_MINUTES: i32 = 60;
    pub const ENABLE_HEADLESS_BROWSER: bool = true;
}

/// Domain-specific settings
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainSettings {
    pub background_check_enabled: bool,
    pub background_check_interval_minutes: i32,
    pub enable_headless_browser: bool,
}

impl Default for DomainSettings {
    fn default() -> Self {
        Self {
            background_check_enabled: defaults::BACKGROUND_CHECK_ENABLED,
            background_check_interval_minutes: defaults::BACKGROUND_CHECK_INTERVAL_MINUTES,
            enable_headless_browser: defaults::ENABLE_HEADLESS_BROWSER,
        }
    }
}

/// Parameters for updating domain settings (all fields optional for partial updates)
#[derive(Default, Deserialize)]
pub struct UpdateDomainSettingsParams {
    pub background_check_enabled: Option<bool>,
    pub background_check_interval_minutes: Option<i32>,
    pub enable_headless_browser: Option<bool>,
}

/// Service layer for domain-specific settings
pub struct DomainSettingService;

impl DomainSettingService {
    /// Get current domain settings
    pub async fn get(conn: &DatabaseConnection) -> Result<DomainSettings, AppError> {
        let scope = SettingScope::Global;
        let r = ScopedSettingsReader::new(conn, &scope);

        Ok(DomainSettings {
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
        })
    }

    /// Update domain settings with validation
    pub async fn update(
        conn: &DatabaseConnection,
        params: UpdateDomainSettingsParams,
    ) -> Result<DomainSettings, AppError> {
        if let Some(interval) = params.background_check_interval_minutes {
            Self::validate_background_check_interval(interval)?;
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

        Self::get(conn).await
    }

    /// Maximum background check interval: 1 week (10080 minutes)
    const MAX_BACKGROUND_CHECK_INTERVAL_MINUTES: i32 = 10080;

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
    fn test_domain_settings_serialize() {
        let settings = DomainSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"background_check_enabled\":false"));
        assert!(json.contains("\"background_check_interval_minutes\":60"));
        assert!(json.contains("\"enable_headless_browser\":true"));
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
    }

    #[tokio::test]
    async fn test_update_all_fields() {
        let conn = setup_app_settings_db().await;
        let params = UpdateDomainSettingsParams {
            background_check_enabled: Some(true),
            background_check_interval_minutes: Some(30),
            enable_headless_browser: Some(false),
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
}
