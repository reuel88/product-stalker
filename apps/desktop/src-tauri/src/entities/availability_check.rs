use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Availability status for a product
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityStatus {
    InStock,
    OutOfStock,
    BackOrder,
    Unknown,
}

impl AvailabilityStatus {
    /// Parse a Schema.org availability value into an AvailabilityStatus
    pub fn from_schema_org(value: &str) -> Self {
        let normalized = value.to_lowercase();

        if normalized.contains("instock") {
            Self::InStock
        } else if normalized.contains("outofstock") {
            Self::OutOfStock
        } else if normalized.contains("backorder") || normalized.contains("preorder") {
            Self::BackOrder
        } else {
            Self::Unknown
        }
    }

    /// Convert to database string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InStock => "in_stock",
            Self::OutOfStock => "out_of_stock",
            Self::BackOrder => "back_order",
            Self::Unknown => "unknown",
        }
    }

}

impl std::fmt::Display for AvailabilityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Availability check entity
///
/// Represents a single availability check for a product.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "availability_checks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    pub product_id: Uuid,

    /// Status as stored in DB (in_stock, out_of_stock, back_order, unknown)
    pub status: String,

    /// Original schema.org availability value
    pub raw_availability: Option<String>,

    /// Error message if check failed
    pub error_message: Option<String>,

    /// When the check was performed
    pub checked_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::product::Entity",
        from = "Column::ProductId",
        to = "super::product::Column::Id"
    )]
    Product,
}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_schema_org_in_stock() {
        assert_eq!(
            AvailabilityStatus::from_schema_org("http://schema.org/InStock"),
            AvailabilityStatus::InStock
        );
        assert_eq!(
            AvailabilityStatus::from_schema_org("https://schema.org/InStock"),
            AvailabilityStatus::InStock
        );
        assert_eq!(
            AvailabilityStatus::from_schema_org("InStock"),
            AvailabilityStatus::InStock
        );
    }

    #[test]
    fn test_from_schema_org_out_of_stock() {
        assert_eq!(
            AvailabilityStatus::from_schema_org("http://schema.org/OutOfStock"),
            AvailabilityStatus::OutOfStock
        );
        assert_eq!(
            AvailabilityStatus::from_schema_org("https://schema.org/OutOfStock"),
            AvailabilityStatus::OutOfStock
        );
    }

    #[test]
    fn test_from_schema_org_back_order() {
        assert_eq!(
            AvailabilityStatus::from_schema_org("http://schema.org/BackOrder"),
            AvailabilityStatus::BackOrder
        );
        assert_eq!(
            AvailabilityStatus::from_schema_org("http://schema.org/PreOrder"),
            AvailabilityStatus::BackOrder
        );
    }

    #[test]
    fn test_from_schema_org_unknown() {
        assert_eq!(
            AvailabilityStatus::from_schema_org("something random"),
            AvailabilityStatus::Unknown
        );
        assert_eq!(
            AvailabilityStatus::from_schema_org(""),
            AvailabilityStatus::Unknown
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", AvailabilityStatus::InStock), "in_stock");
        assert_eq!(
            format!("{}", AvailabilityStatus::OutOfStock),
            "out_of_stock"
        );
        assert_eq!(format!("{}", AvailabilityStatus::BackOrder), "back_order");
        assert_eq!(format!("{}", AvailabilityStatus::Unknown), "unknown");
    }
}
