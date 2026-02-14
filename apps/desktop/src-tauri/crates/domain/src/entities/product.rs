use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Product entity
///
/// Represents a product being tracked for price changes.
/// SQLite will store UUIDs as TEXT and timestamps as TEXT (ISO 8601).
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "products")]
pub struct Model {
    /// Unique identifier (UUID v4)
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Product name
    pub name: String,

    /// Product URL (deprecated — use product_retailers instead)
    pub url: Option<String>,

    /// Optional description
    pub description: Option<String>,

    /// Optional notes
    pub notes: Option<String>,

    /// ISO 4217 currency code (e.g., USD, AUD), auto-set on first scrape
    pub currency: Option<String>,

    /// User-defined display order (0 = first)
    pub sort_order: i32,

    /// Creation timestamp
    pub created_at: DateTimeUtc,

    /// Last update timestamp
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::availability_check::Entity")]
    AvailabilityChecks,

    #[sea_orm(has_many = "super::product_retailer::Entity")]
    ProductRetailers,
}

impl Related<super::availability_check::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AvailabilityChecks.def()
    }
}

impl Related<super::product_retailer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductRetailers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_model_clone() {
        let model = Model {
            id: Uuid::new_v4(),
            name: "Test Product".to_string(),
            url: Some("https://example.com".to_string()),
            description: Some("A description".to_string()),
            notes: None,
            currency: None,
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let cloned = model.clone();
        assert_eq!(model.id, cloned.id);
        assert_eq!(model.name, cloned.name);
        assert_eq!(model.url, cloned.url);
    }

    #[test]
    fn test_model_debug() {
        let model = Model {
            id: Uuid::new_v4(),
            name: "Debug Test".to_string(),
            url: Some("https://debug.test".to_string()),
            description: None,
            notes: None,
            currency: None,
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let debug_str = format!("{:?}", model);
        assert!(debug_str.contains("Model"));
        assert!(debug_str.contains("Debug Test"));
    }

    #[test]
    fn test_model_partial_eq() {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let model1 = Model {
            id,
            name: "Product".to_string(),
            url: Some("https://product.com".to_string()),
            description: None,
            notes: None,
            currency: None,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        };
        let model2 = Model {
            id,
            name: "Product".to_string(),
            url: Some("https://product.com".to_string()),
            description: None,
            notes: None,
            currency: None,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(model1, model2);
    }

    #[test]
    fn test_model_not_equal_different_ids() {
        let now = Utc::now();
        let model1 = Model {
            id: Uuid::new_v4(),
            name: "Product".to_string(),
            url: Some("https://product.com".to_string()),
            description: None,
            notes: None,
            currency: None,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        };
        let model2 = Model {
            id: Uuid::new_v4(),
            name: "Product".to_string(),
            url: Some("https://product.com".to_string()),
            description: None,
            notes: None,
            currency: None,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        };
        assert_ne!(model1, model2);
    }

    #[test]
    fn test_model_serialize() {
        let id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let model = Model {
            id,
            name: "Serializable Product".to_string(),
            url: Some("https://serial.test".to_string()),
            description: Some("desc".to_string()),
            notes: Some("notes".to_string()),
            currency: None,
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("550e8400-e29b-41d4-a716-446655440000"));
        assert!(json.contains("Serializable Product"));
        assert!(json.contains("serial.test"));
        assert!(json.contains("desc"));
        assert!(json.contains("notes"));
    }

    #[test]
    fn test_model_with_all_optional_fields() {
        let model = Model {
            id: Uuid::new_v4(),
            name: "Full Product".to_string(),
            url: Some("https://full.com".to_string()),
            description: Some("Full description with details".to_string()),
            notes: Some("Important notes about this product".to_string()),
            currency: None,
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(model.description.is_some());
        assert!(model.notes.is_some());
    }

    #[test]
    fn test_model_with_no_optional_fields() {
        let model = Model {
            id: Uuid::new_v4(),
            name: "Minimal Product".to_string(),
            url: Some("https://minimal.com".to_string()),
            description: None,
            notes: None,
            currency: None,
            sort_order: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(model.description.is_none());
        assert!(model.notes.is_none());
    }

    #[test]
    fn test_model_timestamps_not_equal_after_update() {
        let created = Utc::now();
        let updated = created + chrono::Duration::seconds(1);
        let model = Model {
            id: Uuid::new_v4(),
            name: "Time Test".to_string(),
            url: Some("https://time.test".to_string()),
            description: None,
            notes: None,
            currency: None,
            sort_order: 0,
            created_at: created,
            updated_at: updated,
        };
        assert_ne!(model.created_at, model.updated_at);
        assert!(model.updated_at > model.created_at);
    }
}
