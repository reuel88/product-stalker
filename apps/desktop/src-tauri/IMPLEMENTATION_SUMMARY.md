# SeaORM Integration - Implementation Summary

## Overview

Complete SeaORM integration with SQLite for a Tauri app, following clean architecture principles and SQLite best practices.

## What Was Implemented

### 1. Database Layer (`src/db/`)

#### `connection.rs`
- **SQLite Configuration**: WAL mode, foreign keys, optimized synchronous mode
- **Connection Pool**: Configured for SQLite (1-5 connections)
- **Database Path**: Platform-specific app data directory
- **Auto Migration**: Runs migrations on startup

#### `mod.rs`
- **DbState**: Thread-safe connection pool wrapper
- **No Mutex**: Uses SeaORM's built-in Arc-based connection pool

### 2. Entities Layer (`src/entities/`)

#### `product.rs`
- **UUID Primary Key**: Stored as TEXT in SQLite
- **Timestamps**: Created/updated timestamps with chrono
- **Optional Fields**: Description and notes
- **Documented**: Clear comments explaining SQLite storage

#### `availability_check.rs`
- **AvailabilityStatus Enum**: InStock, OutOfStock, BackOrder, Unknown with Schema.org parsing
- **Price Tracking**: price_cents (i64), price_currency (ISO 4217), raw_price
- **Foreign Key Relation**: Belongs to Product entity with cascade delete
- **Error Tracking**: error_message field for scraping failures

#### `app_setting.rs`
- **EAV Model**: Entity-Attribute-Value pattern for flexible settings
- **SettingScope Enum**: Global, User, Workspace, Org with scope_id support
- **JSON Values**: Settings stored as JSON strings for type flexibility

#### `prelude.rs`
- Convenient re-exports for entity types and model aliases

### 3. Migrations Layer (`src/migrations/`)

9 migrations implementing the complete schema:
1. `m20240101_000001_create_products_table.rs` - Products table with indexes
2. `m20240102_000001_create_settings_table.rs` - Initial settings table (deprecated)
3. `m20240103_000001_create_availability_checks_table.rs` - Availability tracking
4. `m20240104_000001_add_background_check_settings.rs` - Background check interval
5. `m20240105_000001_add_headless_browser_setting.rs` - Headless browser toggle
6. `m20250205_000001_add_price_tracking.rs` - Price fields to availability_checks
7. `m20250206_000001_create_app_settings_table.rs` - New EAV settings table
8. `m20250207_000001_backfill_app_settings.rs` - Data migration to new settings
9. `m20250208_000001_drop_old_settings_table.rs` - Cleanup old settings table

### 4. Repositories Layer (`src/repositories/`)

#### `product_repository.rs`
- **Pure Data Access**: No business logic
- **CRUD Operations**: find_all, find_by_id, create, update, delete_by_id
- **SeaORM Encapsulation**: Isolates ORM details from business logic

#### `availability_check_repository.rs`
- **History Queries**: find_all_for_product, find_latest_for_product
- **Price Aggregation**: get_daily_average_price for daily comparisons
- **Bulk Operations**: Support for checking all products

#### `app_settings_repository.rs`
- **Scoped Queries**: get_by_scope_and_key, get_all_for_scope
- **Upsert Pattern**: set_setting with proper update-or-insert logic
- **JSON Handling**: Value serialization/deserialization

### 5. Services Layer (`src/services/`)

#### `product_service.rs`
- **Business Logic**: Input validation, error handling
- **Validation Helpers**: Private methods for name and URL validation
- **Orchestration**: Coordinates repository calls

#### `availability_service.rs`
- **Bulk Checking**: check_all_products with rate limiting and progress events
- **Price Tracking**: Daily average calculation, price drop detection
- **Notification Preparation**: Prepares notification data for back-in-stock alerts

#### `notification_service.rs`
- **Settings-Aware**: Respects user notification preferences
- **Back-in-Stock Detection**: Compares current vs previous availability status
- **Price Drop Alerts**: Detects significant price changes

#### `setting_service.rs`
- **Typed Settings**: get_theme, get_show_in_tray, get_notifications_enabled
- **Update Validation**: Validates setting values before saving
- **Default Values**: Provides sensible defaults for all settings

#### `headless_service.rs`
- **Chrome Integration**: Spawns headless Chrome for bot-protected sites
- **Platform Support**: Works on macOS, Windows, Linux

#### `scraper/` Module
- **Orchestrator Pattern**: ScraperService coordinates extraction strategies
- **Schema.org Parsing**: JSON-LD extraction with ProductGroup/variant support
- **Site-Specific Adapters**: Chemist Warehouse via Next.js __NEXT_DATA__
- **Bot Detection**: Cloudflare challenge detection with headless fallback
- **Price Normalization**: Handles various price formats and currencies

### 6. Commands Layer (`src/commands/`)

#### `products.rs`
- **Tauri IPC Handlers**: 5 commands (get_products, get_product, create_product, update_product, delete_product)
- **DTOs**: CreateProductInput, UpdateProductInput, ProductResponse

#### `availability.rs`
- **Check Commands**: check_availability, check_all_availability
- **History Commands**: get_latest_availability, get_availability_history
- **Desktop Notifications**: Sends notifications via Tauri plugin

#### `settings.rs`
- **Get/Update**: get_settings, update_settings
- **Typed Responses**: SettingsResponse with all setting fields

#### `notifications.rs`
- **Permission Handling**: request_notification_permission, check_notification_permission

#### `window.rs`
- **Window Management**: show_window for tray-click handling

#### `updater.rs`
- **App Updates**: check_for_update, install_update

### 7. Background Tasks (`src/background/`)

#### `availability_checker.rs`
- **Polling Loop**: Settings-driven interval (15min to 24hr)
- **Async Spawn**: Uses Tauri's async_runtime::spawn
- **Progress Events**: Emits events for UI updates

### 8. Plugins (`src/plugins/`)

#### `system_tray.rs`
- **Tray Icon**: Product Stalker icon with menu
- **Context Menu**: Show, Check All, Quit actions
- **Click Handling**: Shows main window on tray icon click

### 9. Error Handling (`src/error.rs`)

- **AppError Enum**: 8 variants covering all error cases
  - `Database`, `NotFound`, `Validation`, `Internal`
  - `Http`, `Scraping`, `BotProtection`, `HttpStatus`
- **Tauri Integration**: Converts to InvokeError with proper error codes
- **JSON Serialization**: Error responses are properly formatted

### 10. Main Integration (`src/lib.rs`)

- **Module Declarations**: All layers properly imported
- **App Setup**: Database initialized on startup
- **State Management**: DbState managed by Tauri
- **Command Registration**: All commands registered
- **Plugin Registration**: Notification, updater, system tray plugins
- **Background Tasks**: Availability checker started on app launch

## Architecture Flow

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (TypeScript)                 │
│                    invoke('create_product')              │
└──────────────────────┬──────────────────────────────────┘
                       │ IPC
┌──────────────────────▼──────────────────────────────────┐
│              Commands Layer (commands/products.rs)       │
│              - Parse JSON input                          │
│              - Validate UUID format                      │
│              - Call service layer                        │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│          Services Layer (services/product_service.rs)    │
│          - Validate business rules                       │
│          - Orchestrate repository calls                  │
│          - Return domain errors                          │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│      Repositories Layer (repositories/product_repo.rs)   │
│      - Pure data access                                  │
│      - SeaORM query building                             │
│      - No business logic                                 │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│              SeaORM (ORM Layer)                          │
│              - Entity mapping                            │
│              - SQL generation                            │
│              - Connection pooling                        │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│            SQLite Database (WAL mode)                    │
│            product_stalker.db                            │
└─────────────────────────────────────────────────────────┘
```

## Key Features

### SQLite Optimization

1. **WAL Mode**: Better concurrency, multiple readers + one writer
2. **Small Pool**: 1-5 connections (SQLite sweet spot)
3. **Foreign Keys**: Enabled for referential integrity
4. **Indexes**: Strategic indexes on frequently queried columns
5. **Text Storage**: UUIDs and timestamps stored as TEXT (SQLite native)

### Clean Architecture

1. **Separation of Concerns**: Each layer has one responsibility
2. **Testability**: Easy to mock/test each layer independently
3. **Maintainability**: Changes in one layer don't affect others
4. **No Magic**: Explicit types, clear data flow

### Best Practices

1. **No Blocking**: No `Mutex` around connection pool
2. **Proper Async**: All database calls are async
3. **Type Safety**: UUIDs, Options, Result types used correctly
4. **Error Handling**: Custom error types with proper conversion
5. **Documentation**: Code comments explaining SQLite specifics

## File Structure

```
apps/desktop/src-tauri/
├── Cargo.toml                    # Dependencies configured
├── src/
│   ├── lib.rs                    # Main app setup
│   ├── main.rs                   # Entry point
│   ├── error.rs                  # Error types (8 variants)
│   ├── utils.rs                  # Shared utilities
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── products.rs           # Product CRUD handlers
│   │   ├── availability.rs       # Availability check handlers
│   │   ├── settings.rs           # Settings handlers
│   │   ├── notifications.rs      # Notification handlers
│   │   ├── window.rs             # Window management
│   │   └── updater.rs            # App update handlers
│   ├── services/
│   │   ├── mod.rs
│   │   ├── product_service.rs    # Product business logic
│   │   ├── availability_service.rs  # Availability checking
│   │   ├── notification_service.rs  # Desktop notifications
│   │   ├── setting_service.rs    # Settings management
│   │   ├── headless_service.rs   # Headless browser
│   │   └── scraper/              # Web scraping module
│   │       ├── mod.rs            # ScraperService orchestrator
│   │       ├── bot_detection.rs  # Cloudflare detection
│   │       ├── http_client.rs    # HTTP with fallback
│   │       ├── schema_org.rs     # JSON-LD parsing
│   │       ├── nextjs_data.rs    # Next.js data extraction
│   │       ├── price_parser.rs   # Price normalization
│   │       └── chemist_warehouse.rs  # Site adapter
│   ├── repositories/
│   │   ├── mod.rs
│   │   ├── product_repository.rs
│   │   ├── availability_check_repository.rs
│   │   ├── app_settings_repository.rs
│   │   └── settings_helpers.rs
│   ├── entities/
│   │   ├── mod.rs
│   │   ├── prelude.rs
│   │   ├── product.rs            # Product entity
│   │   ├── availability_check.rs # Availability with price
│   │   └── app_setting.rs        # EAV settings
│   ├── migrations/               # 9 migrations
│   │   ├── mod.rs
│   │   ├── migrator.rs
│   │   └── m20240101_*.rs ... m20250208_*.rs
│   ├── background/
│   │   ├── mod.rs
│   │   └── availability_checker.rs  # Periodic checks
│   ├── plugins/
│   │   ├── mod.rs
│   │   └── system_tray.rs        # System tray
│   ├── db/
│   │   ├── mod.rs
│   │   └── connection.rs         # Database setup
│   └── test_utils.rs             # Test helpers
├── SEAORM_SETUP.md               # Complete setup guide
├── IMPLEMENTATION_SUMMARY.md     # This file
├── CODE_PATTERNS.md              # Code patterns reference
└── README.md                     # Backend documentation
```

## Dependencies

All required dependencies are in `Cargo.toml`:

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
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"
```

## API Overview

### Product Commands

#### `get_products`
```typescript
const products = await invoke('get_products');
// Returns: ProductResponse[]
```

#### `get_product`
```typescript
const product = await invoke('get_product', { id: 'uuid-string' });
// Returns: ProductResponse
```

#### `create_product`
```typescript
const product = await invoke('create_product', {
    input: { name: string, url: string, description?: string, notes?: string }
});
// Returns: ProductResponse
```

#### `update_product` / `delete_product`
```typescript
await invoke('update_product', { id: 'uuid-string', input: { name?: string, ... } });
await invoke('delete_product', { id: 'uuid-string' });
```

### Availability Commands

#### `check_availability`
```typescript
const check = await invoke('check_availability', { product_id: 'uuid-string' });
// Returns: AvailabilityCheckResponse (includes price and daily comparison)
```

#### `check_all_availability`
```typescript
const summary = await invoke('check_all_availability');
// Returns: BulkCheckSummary { total, successful, failed }
// Emits progress events: "availability-check-progress"
```

#### `get_latest_availability` / `get_availability_history`
```typescript
const latest = await invoke('get_latest_availability', { product_id: 'uuid' });
const history = await invoke('get_availability_history', { product_id: 'uuid', limit?: 50 });
```

### Settings Commands

#### `get_settings` / `update_settings`
```typescript
const settings = await invoke('get_settings');
// Returns: SettingsResponse { theme, show_in_tray, notifications_enabled, ... }

await invoke('update_settings', {
    input: { theme?: string, notifications_enabled?: boolean, ... }
});
```

### Notification Commands

```typescript
await invoke('request_notification_permission');
const hasPermission = await invoke('check_notification_permission');
```

### Window Commands

```typescript
await invoke('show_window');  // Shows main window (used by system tray)
```

### Updater Commands

```typescript
const update = await invoke('check_for_update');  // Returns update info or null
await invoke('install_update');  // Downloads and installs update
```

### Response Types

```typescript
interface ProductResponse {
    id: string;
    name: string;
    url: string;
    description?: string;
    notes?: string;
    created_at: string;
    updated_at: string;
}

interface AvailabilityCheckResponse {
    id: string;
    product_id: string;
    status: 'in_stock' | 'out_of_stock' | 'back_order' | 'unknown';
    raw_availability?: string;
    error_message?: string;
    checked_at: string;
    price_cents?: number;
    price_currency?: string;
    raw_price?: string;
    today_average_price_cents?: number;
    yesterday_average_price_cents?: number;
    is_price_drop: boolean;
}

interface BulkCheckSummary {
    total: number;
    successful: number;
    failed: number;
}

interface SettingsResponse {
    theme: string;
    show_in_tray: boolean;
    notifications_enabled: boolean;
    background_check_enabled: boolean;
    background_check_interval_minutes: number;
    headless_browser_enabled: boolean;
}
```

### Error Response

```typescript
interface ErrorResponse {
    error: string;
    code: "DATABASE_ERROR" | "NOT_FOUND" | "VALIDATION_ERROR" | "INTERNAL_ERROR"
        | "HTTP_ERROR" | "SCRAPING_ERROR" | "BOT_PROTECTION" | "HTTP_STATUS_ERROR";
}
```

## Testing the Integration

Once the build tools are installed, test with:

```bash
# Build the app
cargo build

# Run the app
cargo tauri dev
```

### Manual Testing

1. Create a product from the frontend
2. List all products
3. Update a product
4. Delete a product
5. Verify database file exists in app data directory
6. Verify WAL files are created (*.db-wal, *.db-shm)

## Next Steps

- [x] ~~Install Visual Studio Build Tools~~ - Completed
- [x] ~~Build and Test~~ - ~420 tests passing
- [x] ~~Add Price History~~ - AvailabilityCheck entity with price_cents, daily averages
- [x] ~~Add Web Scraping~~ - ScraperService with Schema.org and site-specific adapters
- [x] ~~Add Notifications~~ - NotificationService with desktop notifications
- [x] ~~Add Background Checks~~ - availability_checker with configurable interval
- [x] ~~Add System Tray~~ - system_tray plugin with context menu
- [ ] **Add Export Functionality** - Export product data to CSV/JSON

## Maintenance

### Adding New Entities

1. Create migration: `sea-orm-cli migrate generate <name>`
2. Create entity in `entities/`
3. Create repository in `repositories/`
4. Create service in `services/`
5. Create commands in `commands/`
6. Register commands in `lib.rs`

### Schema Changes

For production-safe schema changes:
- Add new columns as nullable
- Use table rebuild strategy for complex changes
- Always test migrations on production data copy

## References

- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [SQLite WAL Mode](https://www.sqlite.org/wal.html)
- [Tauri Command System](https://tauri.app/v1/guides/features/command/)
