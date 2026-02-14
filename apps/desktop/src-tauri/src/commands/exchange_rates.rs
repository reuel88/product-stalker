use serde::Serialize;
use tauri::State;

use crate::core::services::{ExchangeRateService, SettingService};
use crate::db::DbState;
use crate::tauri_error::CommandError;

#[derive(Debug, Serialize)]
pub struct ExchangeRateResponse {
    pub id: i32,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub source: String,
    pub fetched_at: String,
}

/// Refresh exchange rates from the API
#[tauri::command]
pub async fn refresh_exchange_rates(db: State<'_, DbState>) -> Result<(), CommandError> {
    let settings = SettingService::get(db.conn()).await?;
    ExchangeRateService::refresh_rates(db.conn(), &settings.preferred_currency).await?;
    Ok(())
}

/// Get all exchange rates
#[tauri::command]
pub async fn get_exchange_rates(
    db: State<'_, DbState>,
) -> Result<Vec<ExchangeRateResponse>, CommandError> {
    let rates = ExchangeRateService::get_all(db.conn()).await?;
    Ok(rates
        .into_iter()
        .map(|r| ExchangeRateResponse {
            id: r.id,
            from_currency: r.from_currency,
            to_currency: r.to_currency,
            rate: r.rate,
            source: r.source,
            fetched_at: r.fetched_at.to_rfc3339(),
        })
        .collect())
}

/// Set a manual exchange rate override
#[tauri::command]
pub async fn set_manual_exchange_rate(
    from: String,
    to: String,
    rate: f64,
    db: State<'_, DbState>,
) -> Result<(), CommandError> {
    ExchangeRateService::set_manual_rate(db.conn(), &from, &to, rate).await?;
    Ok(())
}

/// Delete an exchange rate
#[tauri::command]
pub async fn delete_exchange_rate(
    from: String,
    to: String,
    db: State<'_, DbState>,
) -> Result<(), CommandError> {
    ExchangeRateService::delete_rate(db.conn(), &from, &to).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_rate_response_serializes() {
        let response = ExchangeRateResponse {
            id: 1,
            from_currency: "USD".to_string(),
            to_currency: "AUD".to_string(),
            rate: 1.587,
            source: "api".to_string(),
            fetched_at: "2026-02-15T00:00:00+00:00".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"from_currency\":\"USD\""));
        assert!(json.contains("\"to_currency\":\"AUD\""));
        assert!(json.contains("\"source\":\"api\""));
    }
}
