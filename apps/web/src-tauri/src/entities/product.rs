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

    /// Product URL (where to fetch price data)
    pub url: String,

    /// Optional description
    pub description: Option<String>,

    /// Optional notes
    pub notes: Option<String>,

    /// Creation timestamp
    pub created_at: DateTimeUtc,

    /// Last update timestamp
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    // Future relations will go here (e.g., PriceHistory)
}

impl ActiveModelBehavior for ActiveModel {}
