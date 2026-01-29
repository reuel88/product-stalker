mod connection;

pub use connection::init_db;

use sea_orm::DatabaseConnection;

/// Shared database state that can be used across Tauri commands
///
/// DatabaseConnection is already a connection pool internally,
/// so we don't need to wrap it in a Mutex. It's Arc-based and
/// thread-safe by design.
pub struct DbState {
    pub conn: DatabaseConnection,
}

impl DbState {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Get a reference to the database connection
    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    #[tokio::test]
    async fn test_db_state_new_stores_connection() {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        let state = DbState::new(conn);

        // Verify we can access the connection
        assert!(state.conn().ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_db_state_conn_returns_reference() {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        let state = DbState::new(conn);

        // Call conn() multiple times to verify it returns a reference
        let conn_ref1 = state.conn();
        let conn_ref2 = state.conn();

        // Both should be valid and work
        assert!(conn_ref1.ping().await.is_ok());
        assert!(conn_ref2.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_db_state_public_conn_field_accessible() {
        let conn = Database::connect("sqlite::memory:").await.unwrap();
        let state = DbState::new(conn);

        // Verify public field is accessible
        assert!(state.conn.ping().await.is_ok());
    }
}
