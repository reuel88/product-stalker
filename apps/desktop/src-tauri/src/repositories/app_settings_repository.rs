use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::entities::app_setting::{
    ActiveModel, Column, Entity as AppSetting, Model as AppSettingModel, SettingScope,
};
use crate::error::AppError;

/// Repository for EAV-style app settings
pub struct AppSettingsRepository;

impl AppSettingsRepository {
    /// Get a setting value by scope and key, deserializing from JSON
    pub async fn get_setting<T: DeserializeOwned>(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
    ) -> Result<Option<T>, AppError> {
        let model = Self::find_by_scope_and_key(conn, scope, key).await?;

        match model {
            Some(m) => {
                let value: T = serde_json::from_str(&m.value).map_err(|e| {
                    AppError::Internal(format!("Failed to deserialize setting '{}': {}", key, e))
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set a setting value by scope and key, serializing to JSON
    /// Creates or updates the setting
    pub async fn set_setting<T: Serialize>(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
        value: &T,
    ) -> Result<AppSettingModel, AppError> {
        let json_value = serde_json::to_string(value).map_err(|e| {
            AppError::Internal(format!("Failed to serialize setting '{}': {}", key, e))
        })?;

        let existing = Self::find_by_scope_and_key(conn, scope, key).await?;

        match existing {
            Some(model) => {
                let mut active_model: ActiveModel = model.into();
                active_model.value = Set(json_value);
                active_model.updated_at = Set(chrono::Utc::now());
                let updated = active_model.update(conn).await?;
                Ok(updated)
            }
            None => {
                let active_model = ActiveModel {
                    scope_type: Set(scope.scope_type().to_string()),
                    scope_id: Set(scope.scope_id().map(|s| s.to_string())),
                    key: Set(key.to_string()),
                    value: Set(json_value),
                    updated_at: Set(chrono::Utc::now()),
                    ..Default::default()
                };
                let inserted = active_model.insert(conn).await?;
                Ok(inserted)
            }
        }
    }

    /// List all settings for a given scope as a HashMap
    #[allow(dead_code)]
    pub async fn list_settings(
        conn: &DatabaseConnection,
        scope: &SettingScope,
    ) -> Result<HashMap<String, Value>, AppError> {
        let condition = Self::scope_condition(scope);
        let models = AppSetting::find().filter(condition).all(conn).await?;

        let mut map = HashMap::new();
        for model in models {
            let value: Value = serde_json::from_str(&model.value).map_err(|e| {
                AppError::Internal(format!(
                    "Failed to deserialize setting '{}': {}",
                    model.key, e
                ))
            })?;
            map.insert(model.key, value);
        }

        Ok(map)
    }

    /// Delete a setting by scope and key
    /// Returns true if a setting was deleted, false if it didn't exist
    #[allow(dead_code)]
    pub async fn delete_setting(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
    ) -> Result<bool, AppError> {
        let condition = Self::scope_condition(scope).add(Column::Key.eq(key));
        let result = AppSetting::delete_many()
            .filter(condition)
            .exec(conn)
            .await?;
        Ok(result.rows_affected > 0)
    }

    /// Find a setting by scope and key
    async fn find_by_scope_and_key(
        conn: &DatabaseConnection,
        scope: &SettingScope,
        key: &str,
    ) -> Result<Option<AppSettingModel>, AppError> {
        let condition = Self::scope_condition(scope).add(Column::Key.eq(key));
        let model = AppSetting::find().filter(condition).one(conn).await?;
        Ok(model)
    }

    /// Build a condition for matching a scope
    fn scope_condition(scope: &SettingScope) -> Condition {
        let mut condition = Condition::all().add(Column::ScopeType.eq(scope.scope_type()));

        match scope.scope_id() {
            Some(id) => {
                condition = condition.add(Column::ScopeId.eq(id));
            }
            None => {
                condition = condition.add(Column::ScopeId.is_null());
            }
        }

        condition
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_app_settings_db;

    #[tokio::test]
    async fn test_set_and_get_string_setting() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        AppSettingsRepository::set_setting(&conn, &scope, "theme", &"dark")
            .await
            .unwrap();

        let value: Option<String> = AppSettingsRepository::get_setting(&conn, &scope, "theme")
            .await
            .unwrap();

        assert_eq!(value, Some("dark".to_string()));
    }

    #[tokio::test]
    async fn test_set_and_get_bool_setting() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        AppSettingsRepository::set_setting(&conn, &scope, "show_in_tray", &true)
            .await
            .unwrap();

        let value: Option<bool> = AppSettingsRepository::get_setting(&conn, &scope, "show_in_tray")
            .await
            .unwrap();

        assert_eq!(value, Some(true));
    }

    #[tokio::test]
    async fn test_set_and_get_i32_setting() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        AppSettingsRepository::set_setting(&conn, &scope, "interval", &60)
            .await
            .unwrap();

        let value: Option<i32> = AppSettingsRepository::get_setting(&conn, &scope, "interval")
            .await
            .unwrap();

        assert_eq!(value, Some(60));
    }

    #[tokio::test]
    async fn test_get_nonexistent_setting() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        let value: Option<String> =
            AppSettingsRepository::get_setting(&conn, &scope, "nonexistent")
                .await
                .unwrap();

        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_update_existing_setting() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        AppSettingsRepository::set_setting(&conn, &scope, "theme", &"light")
            .await
            .unwrap();

        AppSettingsRepository::set_setting(&conn, &scope, "theme", &"dark")
            .await
            .unwrap();

        let value: Option<String> = AppSettingsRepository::get_setting(&conn, &scope, "theme")
            .await
            .unwrap();

        assert_eq!(value, Some("dark".to_string()));
    }

    #[tokio::test]
    async fn test_list_settings() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        AppSettingsRepository::set_setting(&conn, &scope, "theme", &"dark")
            .await
            .unwrap();
        AppSettingsRepository::set_setting(&conn, &scope, "show_in_tray", &true)
            .await
            .unwrap();

        let settings = AppSettingsRepository::list_settings(&conn, &scope)
            .await
            .unwrap();

        assert_eq!(settings.len(), 2);
        assert_eq!(
            settings.get("theme"),
            Some(&Value::String("dark".to_string()))
        );
        assert_eq!(settings.get("show_in_tray"), Some(&Value::Bool(true)));
    }

    #[tokio::test]
    async fn test_delete_setting() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        AppSettingsRepository::set_setting(&conn, &scope, "theme", &"dark")
            .await
            .unwrap();

        let deleted = AppSettingsRepository::delete_setting(&conn, &scope, "theme")
            .await
            .unwrap();
        assert!(deleted);

        let value: Option<String> = AppSettingsRepository::get_setting(&conn, &scope, "theme")
            .await
            .unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_delete_nonexistent_setting() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::Global;

        let deleted = AppSettingsRepository::delete_setting(&conn, &scope, "nonexistent")
            .await
            .unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_different_scopes_isolated() {
        let conn = setup_app_settings_db().await;
        let global = SettingScope::Global;
        let user = SettingScope::User("user123".to_string());

        AppSettingsRepository::set_setting(&conn, &global, "theme", &"dark")
            .await
            .unwrap();
        AppSettingsRepository::set_setting(&conn, &user, "theme", &"light")
            .await
            .unwrap();

        let global_theme: Option<String> =
            AppSettingsRepository::get_setting(&conn, &global, "theme")
                .await
                .unwrap();
        let user_theme: Option<String> = AppSettingsRepository::get_setting(&conn, &user, "theme")
            .await
            .unwrap();

        assert_eq!(global_theme, Some("dark".to_string()));
        assert_eq!(user_theme, Some("light".to_string()));
    }

    #[tokio::test]
    async fn test_user_scope_settings() {
        let conn = setup_app_settings_db().await;
        let scope = SettingScope::User("user123".to_string());

        AppSettingsRepository::set_setting(&conn, &scope, "preference", &"value")
            .await
            .unwrap();

        let value: Option<String> = AppSettingsRepository::get_setting(&conn, &scope, "preference")
            .await
            .unwrap();

        assert_eq!(value, Some("value".to_string()));

        // Different user should not see the setting
        let other_user = SettingScope::User("other_user".to_string());
        let other_value: Option<String> =
            AppSettingsRepository::get_setting(&conn, &other_user, "preference")
                .await
                .unwrap();

        assert_eq!(other_value, None);
    }
}
