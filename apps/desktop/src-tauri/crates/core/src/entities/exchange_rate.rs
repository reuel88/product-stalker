use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "exchange_rates")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub source: String,
    pub fetched_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_model_clone() {
        let model = Model {
            id: 1,
            from_currency: "USD".to_string(),
            to_currency: "AUD".to_string(),
            rate: 1.587,
            source: "api".to_string(),
            fetched_at: Utc::now(),
        };
        let cloned = model.clone();
        assert_eq!(model.id, cloned.id);
        assert_eq!(model.from_currency, cloned.from_currency);
        assert_eq!(model.rate, cloned.rate);
    }

    #[test]
    fn test_model_serialize() {
        let model = Model {
            id: 1,
            from_currency: "USD".to_string(),
            to_currency: "AUD".to_string(),
            rate: 1.587,
            source: "api".to_string(),
            fetched_at: Utc::now(),
        };
        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("\"from_currency\":\"USD\""));
        assert!(json.contains("\"to_currency\":\"AUD\""));
        assert!(json.contains("\"source\":\"api\""));
    }
}
