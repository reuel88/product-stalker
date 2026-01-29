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
├── error.rs                        # Error types & conversions
│
├── commands/                       # Tauri IPC layer
│   ├── mod.rs
│   └── products.rs                 # Product CRUD commands
│
├── services/                       # Business logic layer
│   ├── mod.rs
│   └── product_service.rs          # Product business logic
│
├── repositories/                   # Data access layer
│   ├── mod.rs
│   └── product_repository.rs       # Product data access
│
├── entities/                       # SeaORM entities
│   ├── mod.rs
│   ├── prelude.rs
│   └── product.rs                  # Product entity
│
├── migrations/                     # Database migrations
│   ├── mod.rs
│   ├── migrator.rs
│   └── m20240101_000001_create_products_table.rs
│
└── db/                            # Database setup
    ├── mod.rs
    ├── connection.rs              # Connection & WAL config
    └── README.md
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

### Entity: Product

| Column | Type | Constraints |
|--------|------|------------|
| id | UUID (TEXT) | PRIMARY KEY |
| name | TEXT | NOT NULL, INDEXED |
| url | TEXT | NOT NULL |
| description | TEXT | NULLABLE |
| notes | TEXT | NULLABLE |
| created_at | TIMESTAMP (TEXT) | NOT NULL, INDEXED |
| updated_at | TIMESTAMP (TEXT) | NOT NULL |

## API Reference

### Commands

All commands are exposed to the frontend via Tauri's IPC system.

#### Get All Products

```typescript
const products = await invoke<ProductResponse[]>('get_products');
```

#### Get Single Product

```typescript
const product = await invoke<ProductResponse>('get_product', {
    id: 'uuid-string'
});
```

#### Create Product

```typescript
const product = await invoke<ProductResponse>('create_product', {
    input: {
        name: 'iPhone 15',
        url: 'https://example.com/iphone-15',
        description: 'Latest model',  // optional
        notes: 'Track deals'           // optional
    }
});
```

#### Update Product

```typescript
const product = await invoke<ProductResponse>('update_product', {
    id: 'uuid-string',
    input: {
        name: 'iPhone 15 Pro',  // All fields optional
        url: 'https://example.com/new-url',
        description: 'Updated description',
        notes: 'Updated notes'
    }
});
```

#### Delete Product

```typescript
await invoke<void>('delete_product', {
    id: 'uuid-string'
});
```

### Types

```typescript
interface ProductResponse {
    id: string;           // UUID
    name: string;
    url: string;
    description?: string;
    notes?: string;
    created_at: string;   // ISO 8601 timestamp
    updated_at: string;   // ISO 8601 timestamp
}

interface ErrorResponse {
    error: string;
    code: 'DATABASE_ERROR' | 'NOT_FOUND' | 'VALIDATION_ERROR';
}
```

## Error Handling

### Error Types

- **DATABASE_ERROR**: Database operation failed
- **NOT_FOUND**: Entity not found
- **VALIDATION_ERROR**: Input validation failed

### Example Error Handling (TypeScript)

```typescript
try {
    const product = await invoke('get_product', { id: 'invalid-uuid' });
} catch (error) {
    const err = JSON.parse(error as string) as ErrorResponse;
    console.error(`${err.code}: ${err.error}`);
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
- [ ] Price history tracking
- [ ] Web scraping
- [ ] Price alerts
- [ ] Export functionality
- [ ] User preferences

---

Built with Rust, Tauri, SeaORM, and SQLite.
