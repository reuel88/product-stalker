//! Database connection utilities

mod connection;

pub use connection::{build_connection_options, enable_wal_mode, init_db_from_url};
