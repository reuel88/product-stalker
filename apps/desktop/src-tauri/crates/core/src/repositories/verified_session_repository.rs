use chrono::Utc;
use sea_orm::{entity::prelude::*, *};

use crate::entities::verified_session::{
    self, Entity as VerifiedSession, Model as VerifiedSessionModel,
};
use crate::AppError;

pub struct VerifiedSessionRepository;

impl VerifiedSessionRepository {
    /// Find a valid session for the given domain
    pub async fn find_by_domain(
        conn: &DatabaseConnection,
        domain: &str,
    ) -> Result<Option<VerifiedSessionModel>, AppError> {
        let now = Utc::now();

        verified_session::Entity::find()
            .filter(verified_session::Column::Domain.eq(domain))
            .filter(verified_session::Column::ExpiresAt.gt(now))
            .one(conn)
            .await
            .map_err(AppError::from)
    }

    /// Create a new verified session
    pub async fn create(
        conn: &DatabaseConnection,
        domain: String,
        cookies_json: String,
        user_agent: String,
        duration_days: i32,
    ) -> Result<VerifiedSessionModel, AppError> {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(duration_days as i64);

        let new_session = verified_session::ActiveModel {
            id: Set(Uuid::new_v4()),
            domain: Set(domain),
            cookies_json: Set(cookies_json),
            user_agent: Set(user_agent),
            expires_at: Set(expires_at),
            created_at: Set(now),
        };

        new_session.insert(conn).await.map_err(AppError::from)
    }

    /// Delete a session by domain
    pub async fn delete_by_domain(
        conn: &DatabaseConnection,
        domain: &str,
    ) -> Result<bool, AppError> {
        let result = VerifiedSession::delete_many()
            .filter(verified_session::Column::Domain.eq(domain))
            .exec(conn)
            .await
            .map_err(AppError::from)?;

        Ok(result.rows_affected > 0)
    }

    /// Delete expired sessions (cleanup)
    pub async fn delete_expired(conn: &DatabaseConnection) -> Result<u64, AppError> {
        let now = Utc::now();

        let result = VerifiedSession::delete_many()
            .filter(verified_session::Column::ExpiresAt.lt(now))
            .exec(conn)
            .await
            .map_err(AppError::from)?;

        Ok(result.rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::setup_in_memory_db;

    async fn setup_db() -> DatabaseConnection {
        let conn = setup_in_memory_db().await;

        // Create table
        let stmt = sea_orm::Schema::new(sea_orm::DatabaseBackend::Sqlite)
            .create_table_from_entity(VerifiedSession);
        conn.execute(conn.get_database_backend().build(&stmt))
            .await
            .unwrap();

        conn
    }

    #[tokio::test]
    async fn test_create_and_find_session() {
        let conn = setup_db().await;

        let session = VerifiedSessionRepository::create(
            &conn,
            "example.com".to_string(),
            r#"[{"name":"session","value":"abc123"}]"#.to_string(),
            "Mozilla/5.0".to_string(),
            14,
        )
        .await
        .unwrap();

        assert_eq!(session.domain, "example.com");

        let found = VerifiedSessionRepository::find_by_domain(&conn, "example.com")
            .await
            .unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().domain, "example.com");
    }

    #[tokio::test]
    async fn test_expired_session_not_found() {
        let conn = setup_db().await;

        // Create session with -1 days (already expired)
        let _session = VerifiedSessionRepository::create(
            &conn,
            "expired.com".to_string(),
            "[]".to_string(),
            "Mozilla/5.0".to_string(),
            -1,
        )
        .await
        .unwrap();

        let found = VerifiedSessionRepository::find_by_domain(&conn, "expired.com")
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_by_domain() {
        let conn = setup_db().await;

        VerifiedSessionRepository::create(
            &conn,
            "delete-me.com".to_string(),
            "[]".to_string(),
            "Mozilla/5.0".to_string(),
            14,
        )
        .await
        .unwrap();

        let deleted = VerifiedSessionRepository::delete_by_domain(&conn, "delete-me.com")
            .await
            .unwrap();

        assert!(deleted);

        let found = VerifiedSessionRepository::find_by_domain(&conn, "delete-me.com")
            .await
            .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_expired() {
        let conn = setup_db().await;

        // Create expired session
        VerifiedSessionRepository::create(
            &conn,
            "expired1.com".to_string(),
            "[]".to_string(),
            "Mozilla/5.0".to_string(),
            -1,
        )
        .await
        .unwrap();

        // Create valid session
        VerifiedSessionRepository::create(
            &conn,
            "valid.com".to_string(),
            "[]".to_string(),
            "Mozilla/5.0".to_string(),
            14,
        )
        .await
        .unwrap();

        let deleted = VerifiedSessionRepository::delete_expired(&conn)
            .await
            .unwrap();

        assert_eq!(deleted, 1);

        // Verify valid session still exists
        let found = VerifiedSessionRepository::find_by_domain(&conn, "valid.com")
            .await
            .unwrap();

        assert!(found.is_some());
    }
}
