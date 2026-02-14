//! Domain entity prelude - exports product and availability check entity types

#[allow(unused_imports)]
pub use super::availability_check::ActiveModel as AvailabilityCheckActiveModel;
#[allow(unused_imports)]
pub use super::availability_check::AvailabilityStatus;
#[allow(unused_imports)]
pub use super::availability_check::Column as AvailabilityCheckColumn;
#[allow(unused_imports)]
pub use super::availability_check::Entity as AvailabilityCheck;
#[allow(unused_imports)]
pub use super::availability_check::Model as AvailabilityCheckModel;

#[allow(unused_imports)]
pub use super::product::ActiveModel as ProductActiveModel;
#[allow(unused_imports)]
pub use super::product::Column as ProductColumn;
#[allow(unused_imports)]
pub use super::product::Entity as Product;
#[allow(unused_imports)]
pub use super::product::Model as ProductModel;

#[allow(unused_imports)]
pub use super::product_retailer::ActiveModel as ProductRetailerActiveModel;
#[allow(unused_imports)]
pub use super::product_retailer::Column as ProductRetailerColumn;
#[allow(unused_imports)]
pub use super::product_retailer::Entity as ProductRetailer;
#[allow(unused_imports)]
pub use super::product_retailer::Model as ProductRetailerModel;

#[allow(unused_imports)]
pub use super::retailer::ActiveModel as RetailerActiveModel;
#[allow(unused_imports)]
pub use super::retailer::Column as RetailerColumn;
#[allow(unused_imports)]
pub use super::retailer::Entity as Retailer;
#[allow(unused_imports)]
pub use super::retailer::Model as RetailerModel;
