# SeaORM Integration Setup

This document describes the SeaORM integration for the Tauri application.

## Architecture

The application follows a clean layered architecture:

```
Commands (Tauri IPC)
    ↓
Services (Business Logic & Validation)
    ↓
Repositories (Data Access)
    ↓
SeaORM (Database ORM)
    ↓
SQLite (Database)
```

## Directory Structure

```
src/
├── commands/           # Tauri command handlers (IPC layer)
│   ├── mod.rs
│   └── products.rs     # Product CRUD commands
├── services/           # Business logic layer
│   ├── mod.rs
│   └── product_service.rs
├── repositories/       # Data access layer
│   ├── mod.rs
│   └── product_repository.rs
├── entities/           # SeaORM entity definitions
│   ├── mod.rs
│   ├── prelude.rs
│   └── product.rs      # Product entity
├── migrations/         # Database migrations
│   ├── mod.rs
│   ├── migrator.rs
│   └── m20240101_000001_create_products_table.rs
├── db/                 # Database connection & configuration
│   ├── mod.rs
│   └── connection.rs
└── error.rs           # Error types
```

## Key Features

### 1. SQLite Configuration

- **WAL Mode**: Enabled for better concurrency
- **Connection Pool**: Limited to 5 connections (optimal for SQLite)
- **Foreign Keys**: Enabled
- **Synchronous Mode**: NORMAL (balanced performance with WAL)

### 2. Database Location

The database file is stored in the app's data directory:
- Windows: `%APPDATA%\<app>\product_stalker.db`
- macOS: `~/Library/Application Support/<app>/product_stalker.db`
- Linux: `~/.local/share/<app>/product_stalker.db`

### 3. Entity Design

The `Product` entity uses:
- **UUID for Primary Key**: Stored as TEXT in SQLite
- **Timestamps**: Stored as TEXT in ISO 8601 format
- **Indexes**: Created on `name` and `created_at` for query performance

### 4. Clean Architecture Benefits

**Commands Layer** (`commands/`)
- Handles Tauri IPC
- Converts between JSON and Rust types
- Minimal logic - just delegation

**Services Layer** (`services/`)
- Contains all business logic
- Validates inputs
- Coordinates repository calls
- Returns domain errors

**Repositories Layer** (`repositories/`)
- Pure data access
- No business logic
- Encapsulates SeaORM details
- Easy to test and mock

## Usage Examples

### Creating a Product

```rust
// From the frontend (JavaScript/TypeScript):
const product = await invoke('create_product', {
    input: {
        name: "iPhone 15",
        url: "https://example.com/iphone-15",
        description: "Latest iPhone model",
        notes: "Track Black Friday deals"
    }
});
```

### Getting All Products

```rust
// From the frontend:
const products = await invoke('get_products');
```

### Updating a Product

```rust
// From the frontend:
const updated = await invoke('update_product', {
    id: "uuid-here",
    input: {
        name: "iPhone 15 Pro"  // partial update
    }
});
```

### Deleting a Product

```rust
// From the frontend:
await invoke('delete_product', { id: "uuid-here" });
```

## Adding New Entities

To add a new entity (e.g., `PriceHistory`):

1. **Create Migration**
```bash
# From src-tauri directory:
sea-orm-cli migrate generate create_price_history_table
```

2. **Create Entity** (`src/entities/price_history.rs`)
```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "price_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub product_id: Uuid,
    pub price: Decimal,
    pub captured_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::product::Entity",
        from = "Column::ProductId",
        to = "super::product::Column::Id"
    )]
    Product,
}

impl ActiveModelBehavior for ActiveModel {}
```

3. **Create Repository** (`src/repositories/price_history_repository.rs`)

4. **Create Service** (`src/services/price_history_service.rs`)

5. **Create Commands** (`src/commands/price_history.rs`)

## SQLite Best Practices

### DO:
- Keep transactions short
- Use indexes for frequently queried columns
- Batch operations when possible
- Use prepared statements (SeaORM does this automatically)
- Enable WAL mode (already configured)
- Use connection pooling (already configured)

### DON'T:
- Use large connection pools (> 5)
- Hold transactions open for long periods
- Make many small queries in a loop (use batch operations)
- Block the async runtime with `Mutex` (use the connection pool)
- Use unsupported SQLite features

## Error Handling

The application uses a custom `AppError` type that maps to Tauri's error system:

```rust
pub enum AppError {
    Database(DbErr),       // Database errors
    NotFound(String),      // Entity not found
    Validation(String),    // Input validation errors
}
```

These errors are automatically converted to JSON error responses for the frontend.

## Testing

### Unit Tests

Test services and repositories independently:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_product() {
        // Setup in-memory SQLite database
        let db = Database::connect("sqlite::memory:").await.unwrap();

        // Run migrations
        Migrator::up(&db, None).await.unwrap();

        // Test product creation
        let product = ProductService::create(
            &db,
            "Test Product".to_string(),
            "https://example.com".to_string(),
            None,
            None,
        ).await.unwrap();

        assert_eq!(product.name, "Test Product");
    }
}
```

## Dependencies

Key dependencies in `Cargo.toml`:

```toml
sea-orm = { version = "1.1", features = [
    "sqlx-sqlite",
    "runtime-tokio-rustls",
    "macros",
    "with-chrono",
    "with-uuid"
] }
sea-orm-migration = { version = "1.1", features = [
    "sqlx-sqlite",
    "runtime-tokio-rustls"
] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Performance Considerations

### Indexes

Indexes are automatically created on:
- `products.name` - for searching/filtering
- `products.created_at` - for sorting by date

### Query Optimization

When adding complex queries, use `EXPLAIN QUERY PLAN`:

```rust
// In development, log query execution plans
let products = Product::find()
    .filter(product::Column::Name.contains("iPhone"))
    .all(db)
    .await?;
```

### Connection Pool

The pool is configured for optimal SQLite performance:
- Min connections: 1
- Max connections: 5
- Timeouts: 8 seconds

## Troubleshooting

### Issue: "database is locked"

**Cause**: Long-running transactions or too many concurrent writes.

**Solution**:
- WAL mode is already enabled (helps with this)
- Keep transactions short
- Use the connection pool properly (don't use `Mutex`)

### Issue: "no such table"

**Cause**: Migrations haven't run.

**Solution**: Migrations run automatically on app startup via `init_db()`.

### Issue: UUID format errors

**Cause**: SQLite stores UUIDs as TEXT.

**Solution**: Already handled by SeaORM's UUID serialization.

## Migration Strategy

For production deployments:

1. **Additive Changes**: Always prefer adding new columns/tables over modifying existing ones
2. **Data Migrations**: For complex changes, create new table → copy data → swap tables
3. **Reversibility**: Always implement `down()` migrations
4. **Testing**: Test migrations on a copy of production data

Example of a safe column addition:

```rust
// Adding a new nullable column is always safe
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .alter_table(
            Table::alter()
                .table(Products::Table)
                .add_column(ColumnDef::new(Products::ImageUrl).string().null())
                .to_owned(),
        )
        .await
}
```

## Next Steps

1. Add price history tracking
2. Add web scraping for price updates
3. Add notification system for price changes
4. Add user preferences
5. Add export functionality
