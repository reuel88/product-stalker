use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Statement,
};

use crate::entities::exchange_rate::{self, Entity as ExchangeRate};
use crate::error::AppError;

pub struct ExchangeRateRepository;

impl ExchangeRateRepository {
    /// Find the latest rate for a currency pair (any source)
    pub async fn find_rate(
        conn: &DatabaseConnection,
        from: &str,
        to: &str,
    ) -> Result<Option<exchange_rate::Model>, AppError> {
        let result = ExchangeRate::find()
            .filter(exchange_rate::Column::FromCurrency.eq(from))
            .filter(exchange_rate::Column::ToCurrency.eq(to))
            .one(conn)
            .await?;
        Ok(result)
    }

    /// Find a manual rate override for a currency pair
    pub async fn find_manual_rate(
        conn: &DatabaseConnection,
        from: &str,
        to: &str,
    ) -> Result<Option<exchange_rate::Model>, AppError> {
        let result = ExchangeRate::find()
            .filter(exchange_rate::Column::FromCurrency.eq(from))
            .filter(exchange_rate::Column::ToCurrency.eq(to))
            .filter(exchange_rate::Column::Source.eq("manual"))
            .one(conn)
            .await?;
        Ok(result)
    }

    /// Upsert an exchange rate (insert or update on conflict)
    pub async fn upsert_rate(
        conn: &DatabaseConnection,
        from: &str,
        to: &str,
        rate: f64,
        source: &str,
    ) -> Result<exchange_rate::Model, AppError> {
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            r#"INSERT INTO exchange_rates (from_currency, to_currency, rate, source, fetched_at)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT(from_currency, to_currency) DO UPDATE SET
                   rate = excluded.rate,
                   source = excluded.source,
                   fetched_at = excluded.fetched_at"#,
            [
                from.into(),
                to.into(),
                rate.into(),
                source.into(),
                now.into(),
            ],
        ))
        .await?;

        // Return the upserted row
        Self::find_rate(conn, from, to)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve upserted exchange rate".into()))
    }

    /// Find all exchange rates
    pub async fn find_all(
        conn: &DatabaseConnection,
    ) -> Result<Vec<exchange_rate::Model>, AppError> {
        let rates = ExchangeRate::find().all(conn).await?;
        Ok(rates)
    }

    /// Delete a rate by currency pair
    pub async fn delete_by_pair(
        conn: &DatabaseConnection,
        from: &str,
        to: &str,
    ) -> Result<(), AppError> {
        ExchangeRate::delete_many()
            .filter(exchange_rate::Column::FromCurrency.eq(from))
            .filter(exchange_rate::Column::ToCurrency.eq(to))
            .exec(conn)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_app_settings_db;

    #[tokio::test]
    async fn test_upsert_and_find_rate() {
        let conn = setup_app_settings_db().await;

        let result = ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.587, "api").await;
        assert!(result.is_ok());

        let rate = ExchangeRateRepository::find_rate(&conn, "USD", "AUD")
            .await
            .unwrap();
        assert!(rate.is_some());
        let rate = rate.unwrap();
        assert_eq!(rate.from_currency, "USD");
        assert_eq!(rate.to_currency, "AUD");
        assert!((rate.rate - 1.587).abs() < 0.001);
        assert_eq!(rate.source, "api");
    }

    #[tokio::test]
    async fn test_upsert_updates_existing() {
        let conn = setup_app_settings_db().await;

        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.5, "api")
            .await
            .unwrap();
        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.6, "api")
            .await
            .unwrap();

        let rate = ExchangeRateRepository::find_rate(&conn, "USD", "AUD")
            .await
            .unwrap()
            .unwrap();
        assert!((rate.rate - 1.6).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_find_manual_rate() {
        let conn = setup_app_settings_db().await;

        // Insert an API rate
        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.5, "api")
            .await
            .unwrap();

        // No manual rate should exist
        let manual = ExchangeRateRepository::find_manual_rate(&conn, "USD", "AUD")
            .await
            .unwrap();
        assert!(manual.is_none());

        // Upsert changes source to manual
        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.6, "manual")
            .await
            .unwrap();

        let manual = ExchangeRateRepository::find_manual_rate(&conn, "USD", "AUD")
            .await
            .unwrap();
        assert!(manual.is_some());
        assert!((manual.unwrap().rate - 1.6).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_find_rate_not_found() {
        let conn = setup_app_settings_db().await;

        let rate = ExchangeRateRepository::find_rate(&conn, "XYZ", "ABC")
            .await
            .unwrap();
        assert!(rate.is_none());
    }

    #[tokio::test]
    async fn test_find_all() {
        let conn = setup_app_settings_db().await;

        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.587, "api")
            .await
            .unwrap();
        ExchangeRateRepository::upsert_rate(&conn, "EUR", "AUD", 1.72, "api")
            .await
            .unwrap();

        let all = ExchangeRateRepository::find_all(&conn).await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_by_pair() {
        let conn = setup_app_settings_db().await;

        ExchangeRateRepository::upsert_rate(&conn, "USD", "AUD", 1.587, "api")
            .await
            .unwrap();
        ExchangeRateRepository::delete_by_pair(&conn, "USD", "AUD")
            .await
            .unwrap();

        let rate = ExchangeRateRepository::find_rate(&conn, "USD", "AUD")
            .await
            .unwrap();
        assert!(rate.is_none());
    }

    #[tokio::test]
    async fn test_delete_by_pair_nonexistent() {
        let conn = setup_app_settings_db().await;

        // Should not error when deleting non-existent pair
        let result = ExchangeRateRepository::delete_by_pair(&conn, "XYZ", "ABC").await;
        assert!(result.is_ok());
    }
}
