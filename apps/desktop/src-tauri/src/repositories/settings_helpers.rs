use sea_orm::DatabaseConnection;
use serde::{de::DeserializeOwned, Serialize};

use crate::entities::app_setting::SettingScope;
use crate::error::AppError;

use super::app_settings_repository::AppSettingsRepository;

/// Typed helper functions for common setting operations
pub struct SettingsHelpers;

/// Scoped settings reader with pre-bound connection and scope.
///
/// Provides a cleaner API for fetching multiple settings with the same scope,
/// reducing repetitive parameter passing.
///
/// # Example
/// ```ignore
/// let reader = ScopedSettingsReader::new(conn, &scope);
/// let theme = reader.string(keys::THEME, defaults::THEME).await?;
/// let enabled = reader.bool(keys::ENABLED, defaults::ENABLED).await?;
/// ```
pub struct ScopedSettingsReader<'a> {
    conn: &'a DatabaseConnection,
    scope: &'a SettingScope,
}

impl<'a> ScopedSettingsReader<'a> {
    /// Create a new scoped settings reader
    pub fn new(conn: &'a DatabaseConnection, scope: &'a SettingScope) -> Self {
        Self { conn, scope }
    }

    /// Get a boolean setting with a default value
    pub async fn bool(&self, key: &str, default: bool) -> Result<bool, AppError> {
        SettingsHelpers::get_bool_or(self.conn, self.scope, key, default).await
    }

    /// Get a string setting with a default value
    pub async fn string(&self, key: &str, default: &str) -> Result<String, AppError> {
        SettingsHelpers::get_string_or(self.conn, self.scope, key, default).await
    }

    /// Get an i32 setting with a default value
    pub async fn i32(&self, key: &str, default: i32) -> Result<i32, AppError> {
        SettingsHelpers::get_i32_or(self.conn, self.scope, key, default).await
    }
}

impl SettingsHelpers {
    // ===== Boolean helpers =====

    /// Get a boolean setting, returning None if not set
    pub async fn get_bool(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
    ) -> Result<Option<bool>, AppError> {
        AppSettingsRepository::get_setting(conn, scope, key).await
    }

    /// Get a boolean setting, returning a default if not set
    pub async fn get_bool_or(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
        default: bool,
    ) -> Result<bool, AppError> {
        Ok(Self::get_bool(conn, scope, key).await?.unwrap_or(default))
    }

    /// Set a boolean setting
    pub async fn set_bool(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
        value: bool,
    ) -> Result<(), AppError> {
        AppSettingsRepository::set_setting(conn, scope, key, &value).await?;
        Ok(())
    }

    // ===== String helpers =====

    /// Get a string setting, returning None if not set
    pub async fn get_string(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
    ) -> Result<Option<String>, AppError> {
        AppSettingsRepository::get_setting(conn, scope, key).await
    }

    /// Get a string setting, returning a default if not set
    pub async fn get_string_or(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
        default: &str,
    ) -> Result<String, AppError> {
        Ok(Self::get_string(conn, scope, key)
            .await?
            .unwrap_or_else(|| default.to_string()))
    }

    /// Set a string setting
    pub async fn set_string(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
        value: &str,
    ) -> Result<(), AppError> {
        AppSettingsRepository::set_setting(conn, scope, key, &value).await?;
        Ok(())
    }

    // ===== i32 helpers =====

    /// Get an i32 setting, returning None if not set
    pub async fn get_i32(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
    ) -> Result<Option<i32>, AppError> {
        AppSettingsRepository::get_setting(conn, scope, key).await
    }

    /// Get an i32 setting, returning a default if not set
    pub async fn get_i32_or(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
        default: i32,
    ) -> Result<i32, AppError> {
        Ok(Self::get_i32(conn, scope, key).await?.unwrap_or(default))
    }

    /// Set an i32 setting
    pub async fn set_i32(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
        value: i32,
    ) -> Result<(), AppError> {
        AppSettingsRepository::set_setting(conn, scope, key, &value).await?;
        Ok(())
    }

    // ===== Generic JSON helpers =====

    /// Get a JSON-serializable setting, returning None if not set
    #[allow(dead_code)]
    pub async fn get_json<T: DeserializeOwned>(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
    ) -> Result<Option<T>, AppError> {
        AppSettingsRepository::get_setting(conn, scope, key).await
    }

    /// Get a JSON-serializable setting, returning a default if not set
    #[allow(dead_code)]
    pub async fn get_json_or<T: DeserializeOwned + Clone>(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
        default: T,
    ) -> Result<T, AppError> {
        Ok(Self::get_json(conn, scope, key).await?.unwrap_or(default))
    }

    /// Set a JSON-serializable setting
    #[allow(dead_code)]
    pub async fn set_json<T: Serialize>(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
        value: &T,
    ) -> Result<(), AppError> {
        AppSettingsRepository::set_setting(conn, scope, key, value).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_app_settings_db;

    #[tokio::test]
    async fn test_get_bool_not_set() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        let value = SettingsHelpers::get_bool(&conn, &scope, "not_set")
            .await
            .unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_set_and_get_bool() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        SettingsHelpers::set_bool(&conn, &scope, "flag", true)
            .await
            .unwrap();

        let value = SettingsHelpers::get_bool(&conn, &scope, "flag")
            .await
            .unwrap();
        assert_eq!(value, Some(true));
    }

    #[tokio::test]
    async fn test_get_bool_or_with_default() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        let value = SettingsHelpers::get_bool_or(&conn, &scope, "not_set", true)
            .await
            .unwrap();
        assert!(value);
    }

    #[tokio::test]
    async fn test_get_bool_or_with_existing() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        SettingsHelpers::set_bool(&conn, &scope, "flag", false)
            .await
            .unwrap();

        let value = SettingsHelpers::get_bool_or(&conn, &scope, "flag", true)
            .await
            .unwrap();
        assert!(!value);
    }

    #[tokio::test]
    async fn test_get_string_not_set() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        let value = SettingsHelpers::get_string(&conn, &scope, "not_set")
            .await
            .unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_set_and_get_string() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        SettingsHelpers::set_string(&conn, &scope, "theme", "dark")
            .await
            .unwrap();

        let value = SettingsHelpers::get_string(&conn, &scope, "theme")
            .await
            .unwrap();
        assert_eq!(value, Some("dark".to_string()));
    }

    #[tokio::test]
    async fn test_get_string_or_with_default() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        let value = SettingsHelpers::get_string_or(&conn, &scope, "not_set", "default")
            .await
            .unwrap();
        assert_eq!(value, "default");
    }

    #[tokio::test]
    async fn test_get_string_or_with_existing() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        SettingsHelpers::set_string(&conn, &scope, "theme", "light")
            .await
            .unwrap();

        let value = SettingsHelpers::get_string_or(&conn, &scope, "theme", "default")
            .await
            .unwrap();
        assert_eq!(value, "light");
    }

    #[tokio::test]
    async fn test_get_i32_not_set() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        let value = SettingsHelpers::get_i32(&conn, &scope, "not_set")
            .await
            .unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_set_and_get_i32() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        SettingsHelpers::set_i32(&conn, &scope, "interval", 60)
            .await
            .unwrap();

        let value = SettingsHelpers::get_i32(&conn, &scope, "interval")
            .await
            .unwrap();
        assert_eq!(value, Some(60));
    }

    #[tokio::test]
    async fn test_get_i32_or_with_default() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        let value = SettingsHelpers::get_i32_or(&conn, &scope, "not_set", 30)
            .await
            .unwrap();
        assert_eq!(value, 30);
    }

    #[tokio::test]
    async fn test_get_i32_or_with_existing() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        SettingsHelpers::set_i32(&conn, &scope, "interval", 15)
            .await
            .unwrap();

        let value = SettingsHelpers::get_i32_or(&conn, &scope, "interval", 30)
            .await
            .unwrap();
        assert_eq!(value, 15);
    }

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct CustomConfig {
        name: String,
        count: i32,
    }

    #[tokio::test]
    async fn test_set_and_get_json() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        let config = CustomConfig {
            name: "test".to_string(),
            count: 42,
        };

        SettingsHelpers::set_json(&conn, &scope, "config", &config)
            .await
            .unwrap();

        let value: Option<CustomConfig> = SettingsHelpers::get_json(&conn, &scope, "config")
            .await
            .unwrap();
        assert_eq!(value, Some(config));
    }

    #[tokio::test]
    async fn test_get_json_or_with_default() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        let default = CustomConfig {
            name: "default".to_string(),
            count: 0,
        };

        let value: CustomConfig =
            SettingsHelpers::get_json_or(&conn, &scope, "not_set", default.clone())
                .await
                .unwrap();
        assert_eq!(value, default);
    }

    // ScopedSettingsReader tests

    #[tokio::test]
    async fn test_scoped_reader_bool() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;
        let reader = ScopedSettingsReader::new(&conn, &scope);

        // Default value when not set
        let value = reader.bool("not_set", true).await.unwrap();
        assert!(value);

        // Existing value
        SettingsHelpers::set_bool(&conn, &scope, "flag", false)
            .await
            .unwrap();
        let value = reader.bool("flag", true).await.unwrap();
        assert!(!value);
    }

    #[tokio::test]
    async fn test_scoped_reader_string() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;
        let reader = ScopedSettingsReader::new(&conn, &scope);

        // Default value when not set
        let value = reader.string("not_set", "default").await.unwrap();
        assert_eq!(value, "default");

        // Existing value
        SettingsHelpers::set_string(&conn, &scope, "theme", "dark")
            .await
            .unwrap();
        let value = reader.string("theme", "light").await.unwrap();
        assert_eq!(value, "dark");
    }

    #[tokio::test]
    async fn test_scoped_reader_i32() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;
        let reader = ScopedSettingsReader::new(&conn, &scope);

        // Default value when not set
        let value = reader.i32("not_set", 60).await.unwrap();
        assert_eq!(value, 60);

        // Existing value
        SettingsHelpers::set_i32(&conn, &scope, "interval", 30)
            .await
            .unwrap();
        let value = reader.i32("interval", 60).await.unwrap();
        assert_eq!(value, 30);
    }
}
