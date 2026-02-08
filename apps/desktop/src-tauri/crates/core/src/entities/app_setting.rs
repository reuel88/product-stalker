use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Setting scope for EAV-style settings
///
/// Determines the scope at which a setting applies.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingScope {
    /// Global settings apply to entire app
    #[default]
    Global,
    /// User-specific settings (user_id)
    User(String),
    /// Workspace-specific settings (workspace_id)
    Workspace(String),
    /// Organization-specific settings (org_id)
    Org(String),
}

impl SettingScope {
    /// Get the scope type string for database storage
    pub fn scope_type(&self) -> &'static str {
        match self {
            SettingScope::Global => "global",
            SettingScope::User(_) => "user",
            SettingScope::Workspace(_) => "workspace",
            SettingScope::Org(_) => "org",
        }
    }

    /// Get the scope ID (None for Global)
    pub fn scope_id(&self) -> Option<&str> {
        match self {
            SettingScope::Global => None,
            SettingScope::User(id) | SettingScope::Workspace(id) | SettingScope::Org(id) => {
                Some(id)
            }
        }
    }

    /// Create a scope from type and optional ID
    #[allow(dead_code)]
    pub fn from_parts(scope_type: &str, scope_id: Option<&str>) -> Option<Self> {
        match scope_type {
            "global" => Some(SettingScope::Global),
            "user" => scope_id.map(|id| SettingScope::User(id.to_string())),
            "workspace" => scope_id.map(|id| SettingScope::Workspace(id.to_string())),
            "org" => scope_id.map(|id| SettingScope::Org(id.to_string())),
            _ => None,
        }
    }
}

/// App settings entity (EAV model)
///
/// Stores settings as key-value pairs with scope support.
/// Values are stored as JSON strings for type flexibility.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "app_settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    /// Scope type: "global", "user", "workspace", "org"
    pub scope_type: String,

    /// Scope identifier (null for global)
    pub scope_id: Option<String>,

    /// Setting key (e.g., "theme", "show_in_tray")
    pub key: String,

    /// JSON-encoded setting value
    #[sea_orm(column_type = "Text")]
    pub value: String,

    /// Last update timestamp
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_scope_global() {
        let scope = SettingScope::Global;
        assert_eq!(scope.scope_type(), "global");
        assert_eq!(scope.scope_id(), None);
    }

    #[test]
    fn test_setting_scope_user() {
        let scope = SettingScope::User("user123".to_string());
        assert_eq!(scope.scope_type(), "user");
        assert_eq!(scope.scope_id(), Some("user123"));
    }

    #[test]
    fn test_setting_scope_workspace() {
        let scope = SettingScope::Workspace("ws456".to_string());
        assert_eq!(scope.scope_type(), "workspace");
        assert_eq!(scope.scope_id(), Some("ws456"));
    }

    #[test]
    fn test_setting_scope_org() {
        let scope = SettingScope::Org("org789".to_string());
        assert_eq!(scope.scope_type(), "org");
        assert_eq!(scope.scope_id(), Some("org789"));
    }

    #[test]
    fn test_setting_scope_from_parts_global() {
        let scope = SettingScope::from_parts("global", None);
        assert_eq!(scope, Some(SettingScope::Global));
    }

    #[test]
    fn test_setting_scope_from_parts_user() {
        let scope = SettingScope::from_parts("user", Some("user123"));
        assert_eq!(scope, Some(SettingScope::User("user123".to_string())));
    }

    #[test]
    fn test_setting_scope_from_parts_workspace() {
        let scope = SettingScope::from_parts("workspace", Some("ws456"));
        assert_eq!(scope, Some(SettingScope::Workspace("ws456".to_string())));
    }

    #[test]
    fn test_setting_scope_from_parts_org() {
        let scope = SettingScope::from_parts("org", Some("org789"));
        assert_eq!(scope, Some(SettingScope::Org("org789".to_string())));
    }

    #[test]
    fn test_setting_scope_from_parts_invalid_type() {
        let scope = SettingScope::from_parts("invalid", None);
        assert_eq!(scope, None);
    }

    #[test]
    fn test_setting_scope_from_parts_user_missing_id() {
        let scope = SettingScope::from_parts("user", None);
        assert_eq!(scope, None);
    }

    #[test]
    fn test_setting_scope_default() {
        let scope = SettingScope::default();
        assert_eq!(scope, SettingScope::Global);
    }

    #[test]
    fn test_setting_scope_serialize() {
        let scope = SettingScope::User("user123".to_string());
        let json = serde_json::to_string(&scope).unwrap();
        assert!(json.contains("User"));
        assert!(json.contains("user123"));
    }

    #[test]
    fn test_setting_scope_deserialize() {
        let json = r#"{"User":"user123"}"#;
        let scope: SettingScope = serde_json::from_str(json).unwrap();
        assert_eq!(scope, SettingScope::User("user123".to_string()));
    }

    #[test]
    fn test_model_serialize() {
        let model = Model {
            id: 1,
            scope_type: "global".to_string(),
            scope_id: None,
            key: "theme".to_string(),
            value: "\"dark\"".to_string(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("\"key\":\"theme\""));
        assert!(json.contains("\"scope_type\":\"global\""));
    }

    #[test]
    fn test_model_with_scope_id() {
        let model = Model {
            id: 1,
            scope_type: "user".to_string(),
            scope_id: Some("user123".to_string()),
            key: "theme".to_string(),
            value: "\"light\"".to_string(),
            updated_at: chrono::Utc::now(),
        };
        assert_eq!(model.scope_id, Some("user123".to_string()));
    }
}
