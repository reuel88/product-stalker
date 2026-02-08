//! Product Stalker Core
//!
//! Reusable infrastructure for Product Stalker application.
//! Contains database setup, settings management, and error handling.

pub mod db;
pub mod entities;
pub mod error;
pub mod migrations;
pub mod repositories;
pub mod services;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

// Re-exports for convenience
pub use error::AppError;
