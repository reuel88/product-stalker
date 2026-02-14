use product_stalker_core::AppError;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::entities::prelude::*;

/// Repository for retailer data access
pub struct RetailerRepository;

impl RetailerRepository {
    /// Find or create a retailer by domain.
    ///
    /// If a retailer with the given domain exists, returns it.
    /// Otherwise creates a new one with the domain as the name.
    pub async fn find_or_create_by_domain(
        conn: &DatabaseConnection,
        domain: &str,
    ) -> Result<RetailerModel, AppError> {
        if let Some(existing) = Self::find_by_domain(conn, domain).await? {
            return Ok(existing);
        }

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let active_model = RetailerActiveModel {
            id: Set(id),
            domain: Set(domain.to_string()),
            name: Set(domain.to_string()),
            created_at: Set(now),
        };

        let retailer = active_model.insert(conn).await?;
        Ok(retailer)
    }

    /// Find a retailer by domain
    pub async fn find_by_domain(
        conn: &DatabaseConnection,
        domain: &str,
    ) -> Result<Option<RetailerModel>, AppError> {
        let retailer = Retailer::find()
            .filter(RetailerColumn::Domain.eq(domain))
            .one(conn)
            .await?;
        Ok(retailer)
    }

    /// Find a retailer by ID
    pub async fn find_by_id(
        conn: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<RetailerModel>, AppError> {
        let retailer = Retailer::find_by_id(id).one(conn).await?;
        Ok(retailer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_retailer_db;

    #[tokio::test]
    async fn test_find_or_create_new() {
        let conn = setup_retailer_db().await;

        let retailer = RetailerRepository::find_or_create_by_domain(&conn, "amazon.com")
            .await
            .unwrap();

        assert_eq!(retailer.domain, "amazon.com");
        assert_eq!(retailer.name, "amazon.com");
    }

    #[tokio::test]
    async fn test_find_or_create_existing() {
        let conn = setup_retailer_db().await;

        let first = RetailerRepository::find_or_create_by_domain(&conn, "amazon.com")
            .await
            .unwrap();
        let second = RetailerRepository::find_or_create_by_domain(&conn, "amazon.com")
            .await
            .unwrap();

        assert_eq!(first.id, second.id);
    }

    #[tokio::test]
    async fn test_find_by_domain_none() {
        let conn = setup_retailer_db().await;

        let result = RetailerRepository::find_by_domain(&conn, "nonexistent.com")
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let conn = setup_retailer_db().await;

        let created = RetailerRepository::find_or_create_by_domain(&conn, "test.com")
            .await
            .unwrap();

        let found = RetailerRepository::find_by_id(&conn, created.id)
            .await
            .unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().domain, "test.com");
    }
}
