use std::collections::HashMap;

use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::error::AppError;
use crate::repositories::ExchangeRateRepository;

/// Response from frankfurter.app API
#[derive(Debug, Deserialize)]
struct FrankfurterResponse {
    rates: HashMap<String, f64>,
}

/// Service for managing exchange rates
pub struct ExchangeRateService;

impl ExchangeRateService {
    /// Fetch latest rates from frankfurter.app for a base currency
    pub async fn fetch_from_api(base: &str) -> Result<HashMap<String, f64>, AppError> {
        let url = format!("https://api.frankfurter.app/latest?base={}", base);
        let response: FrankfurterResponse = reqwest::get(&url)
            .await
            .map_err(|e| AppError::External(format!("Failed to fetch exchange rates: {}", e)))?
            .json()
            .await
            .map_err(|e| AppError::External(format!("Failed to parse exchange rates: {}", e)))?;
        Ok(response.rates)
    }

    /// Refresh all rates for the preferred currency, storing in DB.
    ///
    /// When base=AUD, the API returns rates like `{"USD": 0.63}` meaning 1 AUD = 0.63 USD.
    /// We store both directions:
    /// - from=USD, to=AUD, rate=1/0.63 (how many AUD per 1 USD)
    /// - from=AUD, to=USD, rate=0.63 (how many USD per 1 AUD)
    pub async fn refresh_rates(
        conn: &DatabaseConnection,
        preferred_currency: &str,
    ) -> Result<(), AppError> {
        let rates = Self::fetch_from_api(preferred_currency).await?;

        for (currency, rate) in rates {
            // Store: from=other_currency to=preferred, rate = 1/api_rate
            // e.g., USD->AUD = 1/0.63 ≈ 1.587
            ExchangeRateRepository::upsert_rate(
                conn,
                &currency,
                preferred_currency,
                1.0 / rate,
                "api",
            )
            .await?;

            // Store: from=preferred to=other_currency, rate = api_rate
            // e.g., AUD->USD = 0.63
            ExchangeRateRepository::upsert_rate(conn, preferred_currency, &currency, rate, "api")
                .await?;
        }

        Ok(())
    }

    /// Refresh rates only if stale (>24h) or no rates exist
    pub async fn refresh_if_stale(
        conn: &DatabaseConnection,
        preferred_currency: &str,
    ) -> Result<(), AppError> {
        let all_rates = ExchangeRateRepository::find_all(conn).await?;

        if all_rates.is_empty() {
            return Self::refresh_rates(conn, preferred_currency).await;
        }

        // Check if any rate is more than 24 hours old
        let now = chrono::Utc::now();
        let oldest = all_rates
            .iter()
            .filter(|r| r.source == "api")
            .map(|r| r.fetched_at)
            .min();

        if let Some(oldest_time) = oldest {
            let age = now - oldest_time;
            if age.num_hours() >= 24 {
                return Self::refresh_rates(conn, preferred_currency).await;
            }
        }

        Ok(())
    }

    /// Get rate for a currency pair. Checks manual override first, then API rate.
    /// Identity (same currency) returns 1.0.
    pub async fn get_rate(
        conn: &DatabaseConnection,
        from: &str,
        to: &str,
    ) -> Result<f64, AppError> {
        if from.eq_ignore_ascii_case(to) {
            return Ok(1.0);
        }

        // Check manual override first
        if let Some(manual) = ExchangeRateRepository::find_manual_rate(conn, from, to).await? {
            return Ok(manual.rate);
        }

        // Fall back to API rate
        if let Some(api_rate) = ExchangeRateRepository::find_rate(conn, from, to).await? {
            return Ok(api_rate.rate);
        }

        Err(AppError::NotFound(format!(
            "No exchange rate found for {} -> {}",
            from, to
        )))
    }

    /// Pure conversion function: convert minor units from one currency to another.
    /// Handles different currency exponents (e.g., JPY has 0 decimals, USD has 2).
    pub fn convert_minor_units(amount: i64, rate: f64, from_exp: u32, to_exp: u32) -> i64 {
        let major = amount as f64 / 10_f64.powi(from_exp as i32);
        let converted_major = major * rate;
        (converted_major * 10_f64.powi(to_exp as i32)).round() as i64
    }

    /// Set a manual exchange rate override
    pub async fn set_manual_rate(
        conn: &DatabaseConnection,
        from: &str,
        to: &str,
        rate: f64,
    ) -> Result<(), AppError> {
        if rate <= 0.0 {
            return Err(AppError::Validation(
                "Exchange rate must be positive".to_string(),
            ));
        }
        ExchangeRateRepository::upsert_rate(conn, from, to, rate, "manual").await?;
        Ok(())
    }

    /// Delete an exchange rate by currency pair
    pub async fn delete_rate(
        conn: &DatabaseConnection,
        from: &str,
        to: &str,
    ) -> Result<(), AppError> {
        ExchangeRateRepository::delete_by_pair(conn, from, to).await
    }

    /// Get all exchange rates
    pub async fn get_all(
        conn: &DatabaseConnection,
    ) -> Result<Vec<crate::entities::exchange_rate::Model>, AppError> {
        ExchangeRateRepository::find_all(conn).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_minor_units_same_exponent() {
        // 1000 USD cents at rate 1.587 = 1587 AUD cents
        let result = ExchangeRateService::convert_minor_units(1000, 1.587, 2, 2);
        assert_eq!(result, 1587);
    }

    #[test]
    fn test_convert_minor_units_different_exponents() {
        // 1000 JPY (0 exp) at rate 0.0089 to USD (2 exp) = $8.90 = 890 cents
        let result = ExchangeRateService::convert_minor_units(1000, 0.0089, 0, 2);
        assert_eq!(result, 890);
    }

    #[test]
    fn test_convert_minor_units_to_zero_exponent() {
        // 1000 USD cents ($10.00) at rate 150.0 to JPY (0 exp) = 1500 yen
        let result = ExchangeRateService::convert_minor_units(1000, 150.0, 2, 0);
        assert_eq!(result, 1500);
    }

    #[test]
    fn test_convert_minor_units_identity_rate() {
        // 1000 at rate 1.0 with same exponents = 1000
        let result = ExchangeRateService::convert_minor_units(1000, 1.0, 2, 2);
        assert_eq!(result, 1000);
    }

    #[test]
    fn test_convert_minor_units_rounds_correctly() {
        // 999 cents at rate 1.5 = $9.99 * 1.5 = $14.985 = 1499 cents (rounded)
        let result = ExchangeRateService::convert_minor_units(999, 1.5, 2, 2);
        assert_eq!(result, 1499);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::test_utils::setup_app_settings_db;

    #[tokio::test]
    async fn test_get_rate_identity() {
        let conn = setup_app_settings_db().await;
        let rate = ExchangeRateService::get_rate(&conn, "USD", "USD")
            .await
            .unwrap();
        assert!((rate - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_get_rate_identity_case_insensitive() {
        let conn = setup_app_settings_db().await;
        let rate = ExchangeRateService::get_rate(&conn, "usd", "USD")
            .await
            .unwrap();
        assert!((rate - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_get_rate_not_found() {
        let conn = setup_app_settings_db().await;
        let result = ExchangeRateService::get_rate(&conn, "XYZ", "ABC").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_rate_returns_api_rate() {
        let conn = setup_app_settings_db().await;
        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.587, "api")
            .await
            .unwrap();

        let rate = ExchangeRateService::get_rate(&conn, "USD", "AUD")
            .await
            .unwrap();
        assert!((rate - 1.587).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_get_rate_prefers_manual_over_api() {
        let conn = setup_app_settings_db().await;

        // First set an API rate
        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.5, "api")
            .await
            .unwrap();

        // Then set a manual rate (overwrites via unique constraint)
        ExchangeRateService::set_manual_rate(&conn, "USD", "AUD", 1.6)
            .await
            .unwrap();

        let rate = ExchangeRateService::get_rate(&conn, "USD", "AUD")
            .await
            .unwrap();
        assert!((rate - 1.6).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_set_manual_rate_rejects_zero() {
        let conn = setup_app_settings_db().await;
        let result = ExchangeRateService::set_manual_rate(&conn, "USD", "AUD", 0.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_set_manual_rate_rejects_negative() {
        let conn = setup_app_settings_db().await;
        let result = ExchangeRateService::set_manual_rate(&conn, "USD", "AUD", -1.0).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_rate() {
        let conn = setup_app_settings_db().await;
        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.5, "manual")
            .await
            .unwrap();

        ExchangeRateService::delete_rate(&conn, "USD", "AUD")
            .await
            .unwrap();

        let result = ExchangeRateService::get_rate(&conn, "USD", "AUD").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all() {
        let conn = setup_app_settings_db().await;
        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.587, "api")
            .await
            .unwrap();
        ExchangeRateRepository::upsert_rate(&conn, "EUR", "AUD", 1.72, "api")
            .await
            .unwrap();

        let all = ExchangeRateService::get_all(&conn).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_refresh_if_stale_with_fresh_rates() {
        let conn = setup_app_settings_db().await;

        // Insert a fresh rate
        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.587, "api")
            .await
            .unwrap();

        // Should NOT try to refresh since the rate is fresh
        let result = ExchangeRateService::refresh_if_stale(&conn, "AUD").await;
        assert!(result.is_ok());
    }
}
