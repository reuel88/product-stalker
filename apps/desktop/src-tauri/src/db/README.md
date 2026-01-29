# Database Module

## Overview

This module handles database initialization and connection management for SQLite.

## Files

### `connection.rs`

Contains database initialization logic:

- `get_db_path()` - Returns the platform-specific database file path
- `init_db()` - Initializes database connection with WAL mode and runs migrations
- `enable_wal_mode()` - Configures SQLite for optimal performance

### `mod.rs`

Exports the `DbState` struct that wraps the database connection pool.

## Key Points

### Connection Pool

`DatabaseConnection` from SeaORM is already a connection pool internally (Arc-based).
**Do NOT wrap it in a `Mutex`** - this blocks async code unnecessarily.

```rust
// ✅ Good - Direct access to connection pool
pub struct DbState {
    pub conn: DatabaseConnection,
}

// ❌ Bad - Blocks async runtime
pub struct DbState {
    pub conn: Mutex<DatabaseConnection>,
}
```

### SQLite Configuration

The following PRAGMAs are set on initialization:

- `journal_mode=WAL` - Write-Ahead Logging for better concurrency
- `synchronous=NORMAL` - Balanced durability/performance with WAL
- `foreign_keys=ON` - Enable referential integrity

### Pool Settings

- Max connections: 5 (optimal for SQLite)
- Min connections: 1
- Timeouts: 8 seconds

These settings prevent "database is locked" errors while maintaining good performance.

## Usage

### In `lib.rs`

```rust
let conn = db::init_db(&handle).await?;
app.manage(DbState::new(conn));
```

### In Commands

```rust
#[tauri::command]
pub async fn get_products(db: State<'_, DbState>) -> Result<Vec<Product>, AppError> {
    // Access connection directly - no need to lock
    let products = ProductService::get_all(db.conn()).await?;
    Ok(products)
}
```
