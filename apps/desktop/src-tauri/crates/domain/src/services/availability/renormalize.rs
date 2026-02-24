//! Re-normalization of historical availability checks when the preferred currency changes.

use std::collections::HashMap;

use product_stalker_core::services::ExchangeRateService;
use sea_orm::DatabaseConnection;

use crate::repositories::AvailabilityCheckRepository;
use crate::services::currency::currency_exponent;

use super::AvailabilityService;

/// Summary of a re-normalization run.
#[derive(Debug)]
pub struct RenormalizeSummary {
    pub total: usize,
    pub converted: usize,
    pub same_currency: usize,
    pub cleared: usize,
}

impl AvailabilityService {
    /// Re-normalize all historical checks to a new preferred currency.
    ///
    /// Fetches every check with price data, looks up exchange rates, and
    /// updates the `normalized_price_minor_units` / `normalized_currency`
    /// fields. Checks whose original currency already matches the target
    /// get a simple copy; checks with no available rate have their
    /// normalized fields cleared to `None`.
    pub async fn renormalize_all_checks(
        conn: &DatabaseConnection,
        new_preferred_currency: &str,
    ) -> Result<RenormalizeSummary, product_stalker_core::AppError> {
        let checks = AvailabilityCheckRepository::find_all_with_price_data(conn).await?;

        // Collect distinct source currencies and batch-lookup rates
        let distinct_currencies: Vec<String> = {
            let mut set = std::collections::HashSet::new();
            for check in &checks {
                if let Some(ref currency) = check.price_currency {
                    set.insert(currency.clone());
                }
            }
            set.into_iter().collect()
        };

        let mut rate_cache: HashMap<String, Option<f64>> = HashMap::new();
        for currency in &distinct_currencies {
            if currency.eq_ignore_ascii_case(new_preferred_currency) {
                rate_cache.insert(currency.clone(), Some(1.0));
            } else {
                match ExchangeRateService::get_rate(conn, currency, new_preferred_currency).await {
                    Ok(rate) => {
                        rate_cache.insert(currency.clone(), Some(rate));
                    }
                    Err(_) => {
                        rate_cache.insert(currency.clone(), None);
                    }
                }
            }
        }

        let mut summary = RenormalizeSummary {
            total: checks.len(),
            converted: 0,
            same_currency: 0,
            cleared: 0,
        };

        let to_exp = currency_exponent(new_preferred_currency);

        for check in &checks {
            let (Some(amount), Some(ref from_currency)) =
                (check.price_minor_units, &check.price_currency)
            else {
                continue;
            };

            if from_currency.eq_ignore_ascii_case(new_preferred_currency) {
                // Same currency — normalized = original
                AvailabilityCheckRepository::update_normalized_price(
                    conn,
                    check.id,
                    Some(amount),
                    Some(new_preferred_currency.to_string()),
                )
                .await?;
                summary.same_currency += 1;
            } else if let Some(Some(rate)) = rate_cache.get(from_currency) {
                let from_exp = currency_exponent(from_currency);
                let normalized =
                    ExchangeRateService::convert_minor_units(amount, *rate, from_exp, to_exp);
                AvailabilityCheckRepository::update_normalized_price(
                    conn,
                    check.id,
                    Some(normalized),
                    Some(new_preferred_currency.to_string()),
                )
                .await?;
                summary.converted += 1;
            } else {
                // No rate available — clear stale normalized values
                AvailabilityCheckRepository::update_normalized_price(conn, check.id, None, None)
                    .await?;
                summary.cleared += 1;
            }
        }

        log::info!(
            "Re-normalized {} checks to {}: {} converted, {} same-currency, {} cleared",
            summary.total,
            new_preferred_currency,
            summary.converted,
            summary.same_currency,
            summary.cleared,
        );

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use product_stalker_core::repositories::ExchangeRateRepository;
    use uuid::Uuid;

    use crate::entities::availability_check::AvailabilityStatus;
    use crate::repositories::{AvailabilityCheckRepository, CreateCheckParams};
    use crate::services::AvailabilityService;
    use crate::test_utils::create_test_product_default;

    use super::*;

    /// Set up a combined DB with availability tables + exchange rates table
    async fn setup_combined_db() -> DatabaseConnection {
        use product_stalker_core::entities::exchange_rate::Entity as ExchangeRateEntity;
        use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Schema};

        let conn = Database::connect("sqlite::memory:").await.unwrap();
        let schema = Schema::new(DatabaseBackend::Sqlite);

        // Domain tables
        for entity_stmt in [
            schema.create_table_from_entity(crate::entities::product::Entity),
            schema.create_table_from_entity(crate::entities::retailer::Entity),
            schema.create_table_from_entity(crate::entities::product_retailer::Entity),
            schema.create_table_from_entity(crate::entities::availability_check::Entity),
            schema.create_table_from_entity(ExchangeRateEntity),
        ] {
            conn.execute(conn.get_database_backend().build(&entity_stmt))
                .await
                .unwrap();
        }

        // Unique index needed for upsert
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_exchange_rates_currency_pair ON exchange_rates (from_currency, to_currency)",
        )
        .await
        .unwrap();

        conn
    }

    #[tokio::test]
    async fn test_renormalize_converts_to_new_currency() {
        let conn = setup_combined_db().await;
        let product_id = create_test_product_default(&conn).await;

        // Insert a check priced in USD
        let id = Uuid::new_v4();
        AvailabilityCheckRepository::create(
            &conn,
            id,
            product_id,
            CreateCheckParams {
                status: AvailabilityStatus::InStock,
                price_minor_units: Some(1000), // $10.00
                price_currency: Some("USD".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        // Set up exchange rate: USD -> AUD = 1.587
        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.587, "api")
            .await
            .unwrap();

        let summary = AvailabilityService::renormalize_all_checks(&conn, "AUD")
            .await
            .unwrap();

        assert_eq!(summary.total, 1);
        assert_eq!(summary.converted, 1);
        assert_eq!(summary.same_currency, 0);
        assert_eq!(summary.cleared, 0);

        // Verify the normalized value: $10.00 * 1.587 = A$15.87 = 1587 minor units
        let checks = AvailabilityCheckRepository::find_all_with_price_data(&conn)
            .await
            .unwrap();
        assert_eq!(checks[0].normalized_price_minor_units, Some(1587));
        assert_eq!(checks[0].normalized_currency, Some("AUD".to_string()));
    }

    #[tokio::test]
    async fn test_renormalize_same_currency_copies_original() {
        let conn = setup_combined_db().await;
        let product_id = create_test_product_default(&conn).await;

        let id = Uuid::new_v4();
        AvailabilityCheckRepository::create(
            &conn,
            id,
            product_id,
            CreateCheckParams {
                status: AvailabilityStatus::InStock,
                price_minor_units: Some(5000),
                price_currency: Some("AUD".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let summary = AvailabilityService::renormalize_all_checks(&conn, "AUD")
            .await
            .unwrap();

        assert_eq!(summary.same_currency, 1);
        assert_eq!(summary.converted, 0);

        let checks = AvailabilityCheckRepository::find_all_with_price_data(&conn)
            .await
            .unwrap();
        assert_eq!(checks[0].normalized_price_minor_units, Some(5000));
        assert_eq!(checks[0].normalized_currency, Some("AUD".to_string()));
    }

    #[tokio::test]
    async fn test_renormalize_missing_rate_clears_normalized() {
        let conn = setup_combined_db().await;
        let product_id = create_test_product_default(&conn).await;

        let id = Uuid::new_v4();
        AvailabilityCheckRepository::create(
            &conn,
            id,
            product_id,
            CreateCheckParams {
                status: AvailabilityStatus::InStock,
                price_minor_units: Some(5000),
                price_currency: Some("GBP".to_string()),
                normalized_price_minor_units: Some(9999),
                normalized_currency: Some("OLD".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        // No GBP -> AUD rate exists
        let summary = AvailabilityService::renormalize_all_checks(&conn, "AUD")
            .await
            .unwrap();

        assert_eq!(summary.cleared, 1);

        let checks = AvailabilityCheckRepository::find_all_with_price_data(&conn)
            .await
            .unwrap();
        assert_eq!(checks[0].normalized_price_minor_units, None);
        assert_eq!(checks[0].normalized_currency, None);
    }

    #[tokio::test]
    async fn test_renormalize_no_checks_with_price() {
        let conn = setup_combined_db().await;
        let product_id = create_test_product_default(&conn).await;

        // Create a check with no price
        AvailabilityCheckRepository::create(
            &conn,
            Uuid::new_v4(),
            product_id,
            CreateCheckParams {
                status: AvailabilityStatus::InStock,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let summary = AvailabilityService::renormalize_all_checks(&conn, "AUD")
            .await
            .unwrap();

        assert_eq!(summary.total, 0);
        assert_eq!(summary.converted, 0);
        assert_eq!(summary.same_currency, 0);
        assert_eq!(summary.cleared, 0);
    }

    #[tokio::test]
    async fn test_renormalize_uses_original_price_not_old_normalized() {
        let conn = setup_combined_db().await;
        let product_id = create_test_product_default(&conn).await;

        // Check was originally USD, normalized to AUD with old rate
        let id = Uuid::new_v4();
        AvailabilityCheckRepository::create(
            &conn,
            id,
            product_id,
            CreateCheckParams {
                status: AvailabilityStatus::InStock,
                price_minor_units: Some(1000), // $10.00 USD (original)
                price_currency: Some("USD".to_string()),
                normalized_price_minor_units: Some(1500), // old AUD normalization
                normalized_currency: Some("AUD".to_string()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        // Now re-normalize to EUR with rate USD -> EUR = 0.92
        ExchangeRateRepository::upsert_rate(&conn, "USD", "EUR", 0.92, "api")
            .await
            .unwrap();

        let summary = AvailabilityService::renormalize_all_checks(&conn, "EUR")
            .await
            .unwrap();

        assert_eq!(summary.converted, 1);

        let checks = AvailabilityCheckRepository::find_all_with_price_data(&conn)
            .await
            .unwrap();
        // $10.00 * 0.92 = EUR 9.20 = 920 minor units (NOT based on old 1500 AUD)
        assert_eq!(checks[0].normalized_price_minor_units, Some(920));
        assert_eq!(checks[0].normalized_currency, Some("EUR".to_string()));
    }
}
