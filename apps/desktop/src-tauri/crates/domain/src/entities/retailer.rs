use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Retailer entity
///
/// Represents a retailer (identified by domain) where products are sold.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "retailers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Unique domain name (e.g., "amazon.com")
    #[sea_orm(unique)]
    pub domain: String,

    /// Display name (e.g., "amazon.com")
    pub name: String,

    /// Creation timestamp
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::product_retailer::Entity")]
    ProductRetailers,
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
            domain: "amazon.com".to_string(),
            name: "amazon.com".to_string(),
            created_at: Utc::now(),
        };
        let cloned = model.clone();
        assert_eq!(model.id, cloned.id);
        assert_eq!(model.domain, cloned.domain);
    }

    #[test]
    fn test_model_serialize() {
        let model = Model {
            id: Uuid::new_v4(),
            domain: "walmart.com".to_string(),
            name: "walmart.com".to_string(),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("walmart.com"));
    }
}
