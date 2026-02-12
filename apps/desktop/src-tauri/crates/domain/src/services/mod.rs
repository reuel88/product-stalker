//! Domain services

mod availability_service;
pub mod currency;
mod domain_setting_service;
mod headless_service;
mod notification_service;
mod product_service;
pub mod scraper;

pub use availability_service::{
    AvailabilityService, BulkCheckResult, BulkCheckSummary, CheckProcessingResult,
    CheckResultWithNotification, DailyPriceComparison, ProductCheckContext,
};
pub use domain_setting_service::{
    DomainSettingService, DomainSettings, DomainSettingsCache, UpdateDomainSettingsParams,
};
pub use headless_service::HeadlessService;
pub use notification_service::NotificationService;
pub use product_service::{CreateProductParams, ProductService, UpdateProductParams};
pub use product_stalker_core::services::notification_helpers::NotificationData;
pub use scraper::ScraperService;
