use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Product-Retailer junction entity
///
/// Links a product to a specific retailer URL. A product can have
/// multiple retailers, each with their own URL for price checking.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "product_retailers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// Product this link belongs to
    pub product_id: Uuid,

    /// Retailer (identified by domain)
    pub retailer_id: Uuid,

    /// The product page URL at this retailer
    pub url: String,

    /// Optional user-provided label (e.g., "64GB version")
    pub label: Option<String>,

    /// Creation timestamp
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::product::Entity",
        from = "Column::ProductId",
        to = "super::product::Column::Id"
    )]
    Product,

    #[sea_orm(
        belongs_to = "super::retailer::Entity",
        from = "Column::RetailerId",
        to = "super::retailer::Column::Id"
    )]
    Retailer,

    #[sea_orm(has_many = "super::availability_check::Entity")]
    AvailabilityChecks,
}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl Related<super::retailer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Retailer.def()
    }
}

impl Related<super::availability_check::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AvailabilityChecks.def()
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
            product_id: Uuid::new_v4(),
            retailer_id: Uuid::new_v4(),
            url: "https://amazon.com/dp/B123".to_string(),
            label: Some("64GB".to_string()),
            created_at: Utc::now(),
        };
        let cloned = model.clone();
        assert_eq!(model.id, cloned.id);
        assert_eq!(model.url, cloned.url);
        assert_eq!(model.label, cloned.label);
    }

    #[test]
    fn test_model_serialize() {
        let model = Model {
            id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            retailer_id: Uuid::new_v4(),
            url: "https://walmart.com/item/456".to_string(),
            label: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("walmart.com"));
    }
}
