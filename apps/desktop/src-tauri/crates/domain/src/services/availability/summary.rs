//! Bulk check summary building and counter management.

use crate::services::NotificationService;
use product_stalker_core::services::notification_helpers::NotificationData;

use super::types::{BulkCheckCounters, BulkCheckResult, BulkCheckSummary, CheckProcessingResult};
use super::AvailabilityService;

impl AvailabilityService {
    /// Build summary from collected results
    pub fn build_summary_from_results(
        total: usize,
        results: Vec<(BulkCheckResult, CheckProcessingResult)>,
    ) -> BulkCheckSummary {
        let mut counters = BulkCheckCounters::default();
        let mut bulk_results = Vec::with_capacity(results.len());

        for (bulk_result, processing_result) in results {
            Self::update_counters(&mut counters, &processing_result);
            bulk_results.push(bulk_result);
        }

        Self::build_bulk_summary(total, counters, bulk_results)
    }

    /// Update counters based on the check result
    pub fn update_counters(counters: &mut BulkCheckCounters, result: &CheckProcessingResult) {
        if result.error.is_some() {
            counters.failed += 1;
            return;
        }

        counters.successful += 1;
        if result.is_back_in_stock {
            counters.back_in_stock_count += 1;
        }
        if result.is_price_drop {
            counters.price_drop_count += 1;
        }
    }

    /// Build the final bulk check summary from counters and results
    pub fn build_bulk_summary(
        total: usize,
        counters: BulkCheckCounters,
        results: Vec<BulkCheckResult>,
    ) -> BulkCheckSummary {
        BulkCheckSummary {
            total,
            successful: counters.successful,
            failed: counters.failed,
            back_in_stock_count: counters.back_in_stock_count,
            price_drop_count: counters.price_drop_count,
            results,
        }
    }

    /// Build notification data for a bulk check using pre-fetched settings
    ///
    /// Delegates to NotificationService for actual notification composition.
    pub fn build_bulk_notification_with_settings(
        enable_notifications: bool,
        summary: &BulkCheckSummary,
    ) -> Option<NotificationData> {
        NotificationService::build_bulk_notification(
            enable_notifications,
            summary.back_in_stock_count,
            summary.price_drop_count,
            &summary.results,
        )
    }
}
