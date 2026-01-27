mod connection;

pub use connection::{get_db_path, init_db};

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
