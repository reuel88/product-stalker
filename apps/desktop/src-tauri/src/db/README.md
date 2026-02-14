# Database Module

## Overview

This module handles database initialization and connection management for SQLite.

## Files

### `connection.rs`

Contains database initialization logic:

- `get_db_path()` - Returns the platform-specific database file path
- `init_db()` - Initializes database connection with WAL mode and runs migrations
- `enable_wal_mode()` - Sets WAL journal mode (database-level)

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

PRAGMAs are split into two categories based on scope:

**Database-level** (persists in the file, set once via `enable_wal_mode()`):
- `journal_mode=WAL` - Write-Ahead Logging for better concurrency

**Per-connection** (set via `map_sqlx_sqlite_opts()` in `build_connection_options()`):
- `synchronous=NORMAL` - Balanced durability/performance with WAL
- `foreign_keys=ON` - Enable referential integrity (also sqlx default)

Using `map_sqlx_sqlite_opts()` ensures these pragmas apply to **every** connection
in the pool, not just the first one acquired. This is important because
`DatabaseConnection` is a pool of 5 connections, and pragmas set via
`conn.execute("PRAGMA ...")` only affect whichever connection is acquired.

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

### Migration Safety

SeaORM does **not** wrap SQLite migrations in transactions. Each `execute_unprepared()`
acquires a random connection from the pool, so multi-step DDL (DROP TABLE + ALTER TABLE
RENAME) can fail if statements land on different connections with stale schema state.

**Always wrap multi-step DDL in a transaction:**

```rust
let txn = db.begin().await?;
txn.execute_unprepared("CREATE TABLE t_new (...)").await?;
txn.execute_unprepared("DROP TABLE t").await?;
txn.execute_unprepared("ALTER TABLE t_new RENAME TO t").await?;
txn.commit().await?;
```

Single-statement DDL (ALTER TABLE ADD COLUMN, CREATE TABLE) does not need a transaction.
