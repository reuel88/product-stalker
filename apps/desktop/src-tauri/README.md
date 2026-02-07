# Product Stalker - Tauri Backend

A production-ready Tauri backend with SeaORM and SQLite, following clean architecture principles.

## Quick Start

### Prerequisites

- Rust 1.77.2+
- Visual Studio Build Tools (Windows)
- Node.js (for Tauri frontend)

### Build & Run

```bash
# Install dependencies
cargo fetch

# Build
cargo build

# Run in development mode
cargo tauri dev

# Build for production
cargo tauri build
```

## Project Structure

```
src/
├── lib.rs                          # App initialization & setup
├── main.rs                         # Entry point
├── error.rs                        # Error types (8 variants)
├── utils.rs                        # Shared utilities
│
├── commands/                       # Tauri IPC layer
│   ├── mod.rs
│   ├── products.rs                 # Product CRUD commands
│   ├── availability.rs             # Availability check commands
│   ├── settings.rs                 # Settings management
│   ├── notifications.rs            # Notification commands
│   ├── window.rs                   # Window management
│   └── updater.rs                  # App update commands
│
├── services/                       # Business logic layer
│   ├── mod.rs
│   ├── product_service.rs          # Product business logic
│   ├── availability_service.rs     # Bulk checking, price tracking
│   ├── notification_service.rs     # Desktop notifications
│   ├── setting_service.rs          # App settings
│   ├── headless_service.rs         # Headless browser support
│   └── scraper/                    # Web scraping module
│       ├── mod.rs                  # ScraperService orchestrator
│       ├── bot_detection.rs        # Cloudflare detection
│       ├── http_client.rs          # HTTP with fallback
│       ├── schema_org.rs           # JSON-LD parsing
│       ├── nextjs_data.rs          # Next.js data extraction
│       ├── price_parser.rs         # Price normalization
│       └── chemist_warehouse.rs    # Site-specific adapter
│
├── repositories/                   # Data access layer
│   ├── mod.rs
│   ├── product_repository.rs       # Product data access
│   ├── availability_check_repository.rs
│   ├── app_settings_repository.rs
│   └── settings_helpers.rs
│
├── entities/                       # SeaORM entities
│   ├── mod.rs
│   ├── prelude.rs
│   ├── product.rs                  # Product entity
│   ├── availability_check.rs       # Availability with price
│   └── app_setting.rs              # EAV settings
│
├── migrations/                     # Database migrations (9 total)
│   ├── mod.rs
│   ├── migrator.rs
│   └── m20240101_*.rs ... m20250208_*.rs
│
├── background/                     # Background tasks
│   ├── mod.rs
│   └── availability_checker.rs     # Periodic availability checks
│
├── plugins/                        # Tauri plugins
│   ├── mod.rs
│   └── system_tray.rs              # System tray integration
│
├── db/                             # Database setup
│   ├── mod.rs
│   └── connection.rs               # Connection & WAL config
│
└── test_utils.rs                   # Test helpers
```

## Architecture

### Clean Layered Architecture

```
Frontend (TypeScript/React)
         ↓
    Commands (IPC)
         ↓
    Services (Business Logic)
         ↓
   Repositories (Data Access)
         ↓
      SeaORM (ORM)
         ↓
   SQLite (Database)
```

### Layer Responsibilities

| Layer | Responsibility | Contains |
|-------|---------------|----------|
| **Commands** | IPC handling | Type conversion, Tauri integration |
| **Services** | Business logic | Validation, orchestration, error handling |
| **Repositories** | Data access | CRUD operations, query building |
| **Entities** | Data models | SeaORM entities, type definitions |
| **Migrations** | Schema changes | Table creation, indexes, constraints |
| **DB** | Setup | Connection pool, WAL mode, pragmas |

## Database

### SQLite Configuration

- **Mode**: Write-Ahead Logging (WAL) for better concurrency
- **Pool Size**: 1-5 connections (optimal for SQLite)
- **Foreign Keys**: Enabled
- **Location**: Platform-specific app data directory

### Entities

#### Product

| Column | Type | Constraints |
|--------|------|------------|
| id | UUID (TEXT) | PRIMARY KEY |
| name | TEXT | NOT NULL, INDEXED |
| url | TEXT | NOT NULL |
| description | TEXT | NULLABLE |
| notes | TEXT | NULLABLE |
| created_at | TIMESTAMP (TEXT) | NOT NULL, INDEXED |
| updated_at | TIMESTAMP (TEXT) | NOT NULL |

#### AvailabilityCheck

| Column | Type | Constraints |
|--------|------|------------|
| id | UUID (TEXT) | PRIMARY KEY |
| product_id | UUID (TEXT) | FK → products.id, CASCADE DELETE |
| status | TEXT | NOT NULL (in_stock, out_of_stock, back_order, unknown) |
| raw_availability | TEXT | NULLABLE (Schema.org value) |
| error_message | TEXT | NULLABLE |
| checked_at | TIMESTAMP (TEXT) | NOT NULL, INDEXED |
| price_cents | INTEGER | NULLABLE |
| price_currency | TEXT | NULLABLE (ISO 4217) |
| raw_price | TEXT | NULLABLE |

#### AppSetting (EAV Model)

| Column | Type | Constraints |
|--------|------|------------|
| id | INTEGER | PRIMARY KEY, AUTO INCREMENT |
| scope_type | TEXT | NOT NULL (global, user, workspace, org) |
| scope_id | TEXT | NULLABLE |
| key | TEXT | NOT NULL |
| value | TEXT | NOT NULL (JSON-encoded) |
| updated_at | TIMESTAMP (TEXT) | NOT NULL |

## API Reference

### Commands

All commands are exposed to the frontend via Tauri's IPC system.

#### Product Commands

```typescript
// Get all products
const products = await invoke<ProductResponse[]>('get_products');

// Get single product
const product = await invoke<ProductResponse>('get_product', { id: 'uuid' });

// Create product
const product = await invoke<ProductResponse>('create_product', {
    input: { name: 'iPhone 15', url: 'https://example.com/iphone', description?: string, notes?: string }
});

// Update product (all fields optional)
const product = await invoke<ProductResponse>('update_product', {
    id: 'uuid', input: { name?: string, url?: string, description?: string, notes?: string }
});

// Delete product
await invoke<void>('delete_product', { id: 'uuid' });
```

#### Availability Commands

```typescript
// Check single product availability (returns with price and daily comparison)
const check = await invoke<AvailabilityCheckResponse>('check_availability', { product_id: 'uuid' });

// Check all products (emits progress events)
const summary = await invoke<BulkCheckSummary>('check_all_availability');

// Get latest availability for product
const latest = await invoke<AvailabilityCheckResponse | null>('get_latest_availability', { product_id: 'uuid' });

// Get availability history
const history = await invoke<AvailabilityCheckResponse[]>('get_availability_history', {
    product_id: 'uuid', limit?: number
});
```

#### Settings Commands

```typescript
// Get all settings
const settings = await invoke<SettingsResponse>('get_settings');

// Update settings (all fields optional)
await invoke<SettingsResponse>('update_settings', {
    input: { theme?: string, notifications_enabled?: boolean, background_check_enabled?: boolean, ... }
});
```

#### Notification Commands

```typescript
await invoke('request_notification_permission');
const hasPermission = await invoke<boolean>('check_notification_permission');
```

#### Window & Updater Commands

```typescript
await invoke('show_window');  // Shows main window
const update = await invoke('check_for_update');  // Returns update info or null
await invoke('install_update');  // Downloads and installs
```

### Types

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

interface ErrorResponse {
    error: string;
    code: 'DATABASE_ERROR' | 'NOT_FOUND' | 'VALIDATION_ERROR' | 'INTERNAL_ERROR'
        | 'HTTP_ERROR' | 'SCRAPING_ERROR' | 'BOT_PROTECTION' | 'HTTP_STATUS_ERROR';
}
```

## Error Handling

### Error Types

- **DATABASE_ERROR**: Database operation failed
- **NOT_FOUND**: Entity not found
- **VALIDATION_ERROR**: Input validation failed
- **INTERNAL_ERROR**: Unexpected internal error
- **HTTP_ERROR**: HTTP request failed (network issues)
- **SCRAPING_ERROR**: Web scraping failed (no data found)
- **BOT_PROTECTION**: Bot protection detected (Cloudflare, etc.)
- **HTTP_STATUS_ERROR**: HTTP status error (403, 404, 503, etc.)

### Example Error Handling (TypeScript)

```typescript
try {
    const product = await invoke('get_product', { id: 'invalid-uuid' });
} catch (error) {
    const err = JSON.parse(error as string) as ErrorResponse;
    console.error(`${err.code}: ${err.error}`);

    // Handle specific error types
    switch (err.code) {
        case 'BOT_PROTECTION':
            // Suggest enabling headless browser
            break;
        case 'SCRAPING_ERROR':
            // Site may not support Schema.org
            break;
    }
}
```

## Development

### Adding New Entities

1. **Create Migration**
   ```bash
   sea-orm-cli migrate generate create_new_table
   ```

2. **Define Entity** (`src/entities/new_entity.rs`)
   - Follow the pattern in `product.rs`
   - Use TEXT for UUIDs and timestamps (SQLite)

3. **Create Repository** (`src/repositories/new_repository.rs`)
   - Pure data access, no business logic

4. **Create Service** (`src/services/new_service.rs`)
   - Business logic, validation

5. **Create Commands** (`src/commands/new_commands.rs`)
   - IPC handlers, DTOs

6. **Register in `lib.rs`**
   - Add module declarations
   - Register commands in `.invoke_handler()`

### Code Style

- **Idiomatic Rust**: Follow Rust best practices
- **Explicit Types**: Avoid type inference for public APIs
- **Documentation**: Comment non-obvious SQLite specifics
- **Error Handling**: Use `Result<T, AppError>` everywhere

### Testing

Run tests with:

```bash
cargo test
```

Example test:

```rust
#[tokio::test]
async fn test_create_product() {
    let db = setup_test_db().await;
    let product = ProductService::create(&db, "Test".into(), "https://example.com".into(), None, None).await.unwrap();
    assert_eq!(product.name, "Test");
}
```

## Performance

### SQLite Optimization

- ✅ WAL mode enabled
- ✅ Small connection pool (1-5)
- ✅ Strategic indexes on frequently queried columns
- ✅ No blocking calls in async code
- ✅ Batch operations where possible

### Best Practices

1. **Keep transactions short**: SQLite doesn't handle long-running transactions well
2. **Use indexes**: Already added on `name` and `created_at`
3. **Avoid N+1 queries**: Use joins or batch loading
4. **Connection pool**: Already configured optimally

## Troubleshooting

### "database is locked"

**Cause**: Long-running transaction or too many concurrent writes

**Solution**: WAL mode is already enabled. Keep transactions short.

### "no such table"

**Cause**: Migrations haven't run

**Solution**: Migrations run automatically on startup. Check logs.

### Build fails with linker errors

**Cause**: Missing Visual Studio Build Tools (Windows)

**Solution**: Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) with C++ workload

## Documentation

- **[SEAORM_SETUP.md](./SEAORM_SETUP.md)** - Complete setup guide
- **[IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)** - What was implemented
- **[CODE_PATTERNS.md](./CODE_PATTERNS.md)** - Code patterns and examples
- **[src/db/README.md](./src/db/README.md)** - Database module docs

## Dependencies

Key dependencies:

- `tauri` - Desktop app framework
- `sea-orm` - Async ORM for Rust
- `sea-orm-migration` - Migration system
- `tokio` - Async runtime
- `uuid` - UUID generation
- `chrono` - Date/time handling
- `thiserror` - Error derivation

## License

[Add your license here]

## Contributing

1. Follow the established architecture
2. Write tests for new features
3. Document SQLite-specific behavior
4. Keep layers separated
5. Use type-safe error handling

## Roadmap

- [x] Product CRUD operations
- [x] SQLite with WAL mode
- [x] Clean architecture
- [x] Error handling
- [x] Price history tracking (AvailabilityCheck with daily averages)
- [x] Web scraping (Schema.org + site-specific adapters)
- [x] Price alerts (desktop notifications)
- [x] User preferences (EAV settings model)
- [x] Background checks (configurable interval)
- [x] System tray integration
- [x] Bot protection handling (headless browser fallback)
- [ ] Export functionality

---

Built with Rust, Tauri, SeaORM, and SQLite.
