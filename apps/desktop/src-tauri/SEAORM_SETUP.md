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
├── commands/              # Tauri command handlers (IPC layer)
│   ├── mod.rs
│   ├── products.rs        # Product CRUD commands
│   ├── availability.rs    # Availability check commands
│   ├── settings.rs        # Settings management commands
│   ├── notifications.rs   # Notification commands
│   ├── window.rs          # Window management commands
│   └── updater.rs         # App update commands
├── services/              # Business logic layer
│   ├── mod.rs
│   ├── product_service.rs
│   ├── availability_service.rs    # Bulk checking, price tracking
│   ├── notification_service.rs    # Desktop notifications
│   ├── setting_service.rs         # App settings management
│   ├── headless_service.rs        # Headless browser support
│   └── scraper/                   # Web scraping module
│       ├── mod.rs                 # ScraperService orchestrator
│       ├── bot_detection.rs       # Cloudflare detection
│       ├── http_client.rs         # HTTP with headless fallback
│       ├── schema_org.rs          # Schema.org JSON-LD parsing
│       ├── nextjs_data.rs         # Next.js __NEXT_DATA__ extraction
│       ├── price_parser.rs        # Price normalization
│       └── chemist_warehouse.rs   # Site-specific adapter
├── repositories/          # Data access layer
│   ├── mod.rs
│   ├── product_repository.rs
│   ├── availability_check_repository.rs
│   ├── app_settings_repository.rs
│   └── settings_helpers.rs
├── entities/              # SeaORM entity definitions
│   ├── mod.rs
│   ├── prelude.rs
│   ├── product.rs         # Product entity
│   ├── availability_check.rs   # Availability check with price tracking
│   └── app_setting.rs     # EAV-style app settings
├── migrations/            # Database migrations (9 total)
│   ├── mod.rs
│   ├── migrator.rs
│   ├── m20240101_000001_create_products_table.rs
│   ├── m20240102_000001_create_settings_table.rs
│   ├── m20240103_000001_create_availability_checks_table.rs
│   ├── m20240104_000001_add_background_check_settings.rs
│   ├── m20240105_000001_add_headless_browser_setting.rs
│   ├── m20250205_000001_add_price_tracking.rs
│   ├── m20250206_000001_create_app_settings_table.rs
│   ├── m20250207_000001_backfill_app_settings.rs
│   └── m20250208_000001_drop_old_settings_table.rs
├── background/            # Background task runners
│   ├── mod.rs
│   └── availability_checker.rs   # Periodic availability checks
├── plugins/               # Tauri plugins
│   ├── mod.rs
│   └── system_tray.rs     # System tray integration
├── db/                    # Database connection & configuration
│   ├── mod.rs
│   └── connection.rs
└── error.rs               # Error types (8 variants)
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

#### Product Entity
- **UUID for Primary Key**: Stored as TEXT in SQLite
- **Timestamps**: Stored as TEXT in ISO 8601 format
- **Indexes**: Created on `name` and `created_at` for query performance

#### AvailabilityCheck Entity
Tracks product availability and price over time:
- **UUID Primary Key**: Links to Product via foreign key
- **AvailabilityStatus Enum**: `in_stock`, `out_of_stock`, `back_order`, `unknown`
- **Price Tracking**: `price_cents` (i64), `price_currency` (ISO 4217), `raw_price` (original)
- **Schema.org Support**: `raw_availability` stores original availability value
- **Error Tracking**: `error_message` captures scraping failures

#### AppSetting Entity (EAV Model)
Flexible key-value settings with scope support:
- **SettingScope Enum**: `Global`, `User(id)`, `Workspace(id)`, `Org(id)`
- **JSON Values**: Values stored as JSON strings for type flexibility
- **Indexed Lookups**: Composite index on (scope_type, scope_id, key)

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
    Database(DbErr),       // Database errors (DATABASE_ERROR)
    NotFound(String),      // Entity not found (NOT_FOUND)
    Validation(String),    // Input validation errors (VALIDATION_ERROR)
    Internal(String),      // Internal errors (INTERNAL_ERROR)
    Http(reqwest::Error),  // HTTP request errors (HTTP_ERROR)
    Scraping(String),      // Web scraping errors (SCRAPING_ERROR)
    BotProtection(String), // Bot protection detected (BOT_PROTECTION)
    HttpStatus { status: u16, url: String },  // HTTP status errors (HTTP_STATUS_ERROR)
}
```

These errors are automatically converted to JSON error responses for the frontend with appropriate error codes.

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

- [x] ~~Add price history tracking~~ - Implemented via AvailabilityCheck entity with price_cents, price_currency
- [x] ~~Add web scraping for price updates~~ - ScraperService with Schema.org and site-specific adapters
- [x] ~~Add notification system for price changes~~ - NotificationService with desktop notifications
- [x] ~~Add user preferences~~ - AppSetting entity with EAV model and SettingService
- [ ] Add export functionality
