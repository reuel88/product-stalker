//! Product Stalker Domain
//!
//! Product-specific domain logic for Product Stalker.
//! Contains product entities, services, and scraping functionality.

pub mod entities;
pub mod migrations;
pub mod repositories;
pub mod services;

#[cfg(test)]
pub mod test_utils;

// Re-exports for convenience
pub use product_stalker_core::AppError;
