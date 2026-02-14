use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Schema.org availability values that map to InStock status
const IN_STOCK_INDICATORS: &[&str] = &[
    "instock",
    "instoreonly",
    "onlineonly",
    "limitedavailability",
];

/// Schema.org availability values that map to OutOfStock status
const OUT_OF_STOCK_INDICATORS: &[&str] = &["outofstock", "soldout", "discontinued"];

/// Schema.org availability values that map to BackOrder status
const BACK_ORDER_INDICATORS: &[&str] = &["backorder", "preorder", "presale"];

/// Availability status for a product
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityStatus {
    InStock,
    OutOfStock,
    BackOrder,
    #[default]
    Unknown,
}

/// Check if the normalized availability string contains any of the given indicators
fn contains_any_indicator(normalized: &str, indicators: &[&str]) -> bool {
    indicators
        .iter()
        .any(|indicator| normalized.contains(indicator))
}

impl AvailabilityStatus {
    /// Parse a Schema.org availability value into an AvailabilityStatus
    ///
    /// Handles all 10 official Schema.org ItemAvailability values:
    /// - InStock, InStoreOnly, OnlineOnly, LimitedAvailability -> InStock
    /// - OutOfStock, SoldOut, Discontinued -> OutOfStock
    /// - BackOrder, PreOrder, PreSale -> BackOrder
    pub fn from_schema_org(value: &str) -> Self {
        let normalized = value.to_lowercase();

        if contains_any_indicator(&normalized, IN_STOCK_INDICATORS) {
            return Self::InStock;
        }

        if contains_any_indicator(&normalized, OUT_OF_STOCK_INDICATORS) {
            return Self::OutOfStock;
        }

        if contains_any_indicator(&normalized, BACK_ORDER_INDICATORS) {
            return Self::BackOrder;
        }

        log::warn!(
            "Unrecognized Schema.org availability value: '{}' - returning Unknown",
            value
        );
        Self::Unknown
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

impl std::str::FromStr for AvailabilityStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in_stock" => Ok(Self::InStock),
            "out_of_stock" => Ok(Self::OutOfStock),
            "back_order" => Ok(Self::BackOrder),
            _ => Ok(Self::Unknown),
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

    /// Product-retailer link this check was performed against
    pub product_retailer_id: Option<Uuid>,

    /// Status as stored in DB (in_stock, out_of_stock, back_order, unknown)
    pub status: String,

    /// Original schema.org availability value
    pub raw_availability: Option<String>,

    /// Error message if check failed
    pub error_message: Option<String>,

    /// When the check was performed
    pub checked_at: DateTimeUtc,

    /// Price in minor units (smallest currency unit)
    pub price_minor_units: Option<i64>,

    /// ISO 4217 currency code (e.g., USD, EUR, AUD)
    pub price_currency: Option<String>,

    /// Original schema.org price value for debugging
    pub raw_price: Option<String>,
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
        belongs_to = "super::product_retailer::Entity",
        from = "Column::ProductRetailerId",
        to = "super::product_retailer::Column::Id"
    )]
    ProductRetailer,
}

impl Related<super::product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl Related<super::product_retailer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductRetailer.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Parse the stored status string into a typed `AvailabilityStatus` enum.
    pub fn status_enum(&self) -> AvailabilityStatus {
        self.status.parse().unwrap_or_default()
    }
}

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

    #[test]
    fn test_from_schema_org_case_insensitive() {
        // Test uppercase
        assert_eq!(
            AvailabilityStatus::from_schema_org("INSTOCK"),
            AvailabilityStatus::InStock
        );
        assert_eq!(
            AvailabilityStatus::from_schema_org("OUTOFSTOCK"),
            AvailabilityStatus::OutOfStock
        );
        // Test mixed case
        assert_eq!(
            AvailabilityStatus::from_schema_org("InStock"),
            AvailabilityStatus::InStock
        );
        assert_eq!(
            AvailabilityStatus::from_schema_org("BackOrder"),
            AvailabilityStatus::BackOrder
        );
    }

    #[test]
    fn test_from_schema_org_preorder() {
        assert_eq!(
            AvailabilityStatus::from_schema_org("http://schema.org/PreOrder"),
            AvailabilityStatus::BackOrder
        );
        assert_eq!(
            AvailabilityStatus::from_schema_org("PreOrder"),
            AvailabilityStatus::BackOrder
        );
        assert_eq!(
            AvailabilityStatus::from_schema_org("preorder"),
            AvailabilityStatus::BackOrder
        );
    }

    #[test]
    fn test_as_str() {
        assert_eq!(AvailabilityStatus::InStock.as_str(), "in_stock");
        assert_eq!(AvailabilityStatus::OutOfStock.as_str(), "out_of_stock");
        assert_eq!(AvailabilityStatus::BackOrder.as_str(), "back_order");
        assert_eq!(AvailabilityStatus::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_availability_status_clone() {
        let status = AvailabilityStatus::InStock;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_availability_status_debug() {
        let status = AvailabilityStatus::InStock;
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("InStock"));
    }

    #[test]
    fn test_availability_status_serialize() {
        let status = AvailabilityStatus::InStock;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"in_stock\"");

        let status = AvailabilityStatus::OutOfStock;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"out_of_stock\"");
    }

    #[test]
    fn test_availability_status_deserialize() {
        let status: AvailabilityStatus = serde_json::from_str("\"in_stock\"").unwrap();
        assert_eq!(status, AvailabilityStatus::InStock);

        let status: AvailabilityStatus = serde_json::from_str("\"out_of_stock\"").unwrap();
        assert_eq!(status, AvailabilityStatus::OutOfStock);

        let status: AvailabilityStatus = serde_json::from_str("\"back_order\"").unwrap();
        assert_eq!(status, AvailabilityStatus::BackOrder);

        let status: AvailabilityStatus = serde_json::from_str("\"unknown\"").unwrap();
        assert_eq!(status, AvailabilityStatus::Unknown);
    }

    #[test]
    fn test_availability_status_partial_eq() {
        assert_eq!(AvailabilityStatus::InStock, AvailabilityStatus::InStock);
        assert_ne!(AvailabilityStatus::InStock, AvailabilityStatus::OutOfStock);
    }

    #[test]
    fn test_from_schema_org_with_partial_match() {
        // Should match even with extra text
        assert_eq!(
            AvailabilityStatus::from_schema_org("http://schema.org/InStock#fragment"),
            AvailabilityStatus::InStock
        );
    }

    #[test]
    fn test_status_enum_helper() {
        let model = Model {
            id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            product_retailer_id: None,
            status: "in_stock".to_string(),
            raw_availability: None,
            error_message: None,
            checked_at: chrono::Utc::now(),
            price_minor_units: None,
            price_currency: None,
            raw_price: None,
        };
        assert_eq!(model.status_enum(), AvailabilityStatus::InStock);

        let model = Model {
            status: "out_of_stock".to_string(),
            ..model
        };
        assert_eq!(model.status_enum(), AvailabilityStatus::OutOfStock);

        let model = Model {
            status: "back_order".to_string(),
            ..model
        };
        assert_eq!(model.status_enum(), AvailabilityStatus::BackOrder);

        let model = Model {
            status: "unknown".to_string(),
            ..model
        };
        assert_eq!(model.status_enum(), AvailabilityStatus::Unknown);

        let model = Model {
            status: "garbage".to_string(),
            ..model
        };
        assert_eq!(model.status_enum(), AvailabilityStatus::Unknown);
    }

    #[test]
    fn test_from_schema_org_with_whitespace() {
        // Whitespace shouldn't break the match since we check contains
        assert_eq!(
            AvailabilityStatus::from_schema_org("  InStock  "),
            AvailabilityStatus::InStock
        );
    }

    /// Assert that a Schema.org value (bare, http://, https://) all map to the expected status
    fn assert_schema_org_maps_to(value: &str, expected: AvailabilityStatus) {
        let variants = [
            value.to_string(),
            format!("http://schema.org/{}", value),
            format!("https://schema.org/{}", value),
        ];
        for input in &variants {
            assert_eq!(
                AvailabilityStatus::from_schema_org(input),
                expected,
                "Failed for input: '{}'",
                input
            );
        }
    }

    // Tests for additional Schema.org ItemAvailability values

    #[test]
    fn test_from_schema_org_in_stock_variants() {
        assert_schema_org_maps_to("InStoreOnly", AvailabilityStatus::InStock);
        assert_schema_org_maps_to("OnlineOnly", AvailabilityStatus::InStock);
        assert_schema_org_maps_to("LimitedAvailability", AvailabilityStatus::InStock);
    }

    #[test]
    fn test_from_schema_org_out_of_stock_variants() {
        assert_schema_org_maps_to("SoldOut", AvailabilityStatus::OutOfStock);
        assert_schema_org_maps_to("Discontinued", AvailabilityStatus::OutOfStock);
    }

    #[test]
    fn test_from_schema_org_back_order_variants() {
        assert_schema_org_maps_to("PreSale", AvailabilityStatus::BackOrder);
    }
}
