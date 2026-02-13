//! Availability service for checking product availability.
//!
//! Organized into focused submodules:
//! - `checker`: Product availability checking and result processing
//! - `comparison`: Price comparison and stock transition detection
//! - `summary`: Bulk check summary building and counter management
//! - `types`: Data types for availability checks and bulk operations

mod checker;
mod comparison;
mod summary;
mod types;

pub use types::{
    BulkCheckResult, BulkCheckSummary, CheckProcessingResult, CheckResultWithNotification,
    DailyPriceComparison, ProductCheckContext,
};

/// Service layer for availability checking business logic
pub struct AvailabilityService;
