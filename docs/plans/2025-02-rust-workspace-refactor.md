# Rust Workspace Refactoring Plan

**Status: COMPLETED** (February 2025)

Restructure the Tauri backend into a Cargo workspace with `core` and `domain` crates for better separation and reusability.

---

## Target Structure

```
apps/desktop/src-tauri/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── core/               # Reusable infrastructure
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── db/         # Database setup (Tauri-agnostic)
│   │       ├── entities/   # app_setting.rs only
│   │       ├── error.rs    # AppError (no Tauri conversion)
│   │       ├── migrations/ # All migrations
│   │       ├── repositories/ # settings repos
│   │       ├── services/   # setting_service, notification_helpers
│   │       └── test_utils.rs
│   │
│   └── domain/             # Product-specific (swappable)
│       └── src/
│           ├── lib.rs
│           ├── entities/   # product.rs, availability_check.rs
│           ├── repositories/
│           ├── services/   # product, availability, notification, scraper/, headless
│           ├── test_utils.rs
│           └── utils.rs
│
└── src/                    # Tauri wiring only
    ├── main.rs
    ├── lib.rs              # Imports from crates
    ├── commands/           # IPC handlers
    ├── background/         # Background tasks
    ├── plugins/            # Tauri plugins (system tray)
    ├── tauri_services/     # Tauri-specific service wrappers (event emission)
    ├── db/                 # Tauri-specific db init (connection.rs)
    ├── tauri_error.rs      # AppError -> InvokeError conversion
    ├── utils.rs
    └── test_utils.rs
```

---

## What Goes Where

| Core Crate | Domain Crate | Main Crate (stays) |
|------------|--------------|-------------------|
| `error.rs` (AppError only) | `entities/product.rs` | `commands/*` |
| `entities/app_setting.rs` | `entities/availability_check.rs` | `background/*` |
| `migrations/*` (all) | `repositories/product_repository.rs` | `plugins/*` |
| `db/connection.rs` (init_db_from_url) | `repositories/availability_check_repository.rs` | `tauri_services/availability_service.rs` |
| `repositories/app_settings_repository.rs` | `services/product_service.rs` | `tauri_error.rs` |
| `repositories/settings_helpers.rs` | `services/availability_service.rs` | `db/connection.rs` (Tauri init) |
| `services/setting_service.rs` | `services/notification_service.rs` | `utils.rs` |
| `services/notification_helpers.rs` | `services/scraper/*` (all) | `test_utils.rs` |
| `test_utils.rs` (db setup) | `services/headless_service.rs` | |
| | `test_utils.rs` | |
| | `utils.rs` | |

---

## Implementation Phases

### Phase 1: Setup Workspace Structure
1. Create `crates/core/` and `crates/domain/` directories
2. Create placeholder `Cargo.toml` files with workspace dependencies
3. Add `[workspace]` section to root `Cargo.toml`
4. Run `cargo check` to verify workspace compiles

### Phase 2: Extract Core Crate
1. Move `error.rs` (remove Tauri conversion)
2. Move `entities/app_setting.rs`
3. Move all `migrations/`
4. Move `db/connection.rs` (keep Tauri parts in main)
5. Move settings repositories and service
6. Move notification helpers
7. Create `lib.rs` with public exports
8. Run `cargo test -p product-stalker-core`

### Phase 3: Extract Domain Crate
1. Move product and availability entities
2. Move product and availability repositories
3. Move product, availability, and headless services
4. Move entire `scraper/` directory
5. Move `utils.rs`
6. Create `lib.rs` with public exports
7. Run `cargo test -p product-stalker-domain`

### Phase 4: Update Main Crate
1. Create `tauri_error.rs` with `impl From<AppError> for InvokeError`
2. Create `db/connection.rs` with Tauri-specific `init_db(app: &AppHandle)`
3. Create `tauri_services/availability_service.rs` for Tauri-specific wrapper with event emission
4. Update `lib.rs` to import from workspace crates
5. Update all command imports
6. Update background task imports
7. Run `cargo test --workspace`

### Phase 5: Cleanup & Verify
1. Delete moved files from `src/`
2. Run `cargo clippy --workspace`
3. Run `cargo test --workspace`
4. Test app: `pnpm dev:desktop`
5. Run coverage: `cargo llvm-cov --workspace`

---

## Key Files to Modify

- `apps/desktop/src-tauri/Cargo.toml` - Convert to workspace root
- `apps/desktop/src-tauri/src/lib.rs` - Rewire to use crate imports
- `apps/desktop/src-tauri/src/error.rs` - Split into `crates/core/src/error.rs` + `src/tauri_error.rs`
- `apps/desktop/src-tauri/src/db/connection.rs` - Split into `crates/core/src/db/connection.rs` + `src/db/connection.rs`
- `apps/desktop/src-tauri/src/services/availability_service.rs` - Split into `crates/domain/` + `src/tauri_services/`

---

## Cargo.toml Configurations

### Workspace Root (`apps/desktop/src-tauri/Cargo.toml`)

```toml
[workspace]
resolver = "2"
members = ["crates/core", "crates/domain", "."]

[package]
name = "app"
version = "0.2.5"
edition = "2021"

[dependencies]
product-stalker-core = { path = "crates/core" }
product-stalker-domain = { path = "crates/domain" }
# ... tauri and other deps

[workspace.dependencies]
sea-orm = { version = "1.1", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros", "with-chrono", "with-uuid"] }
sea-orm-migration = { version = "1.1", features = ["sqlx-sqlite", "runtime-tokio-rustls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
log = "0.4"
tokio = { version = "1", features = ["full"] }
```

### Core Crate (`crates/core/Cargo.toml`)

```toml
[package]
name = "product-stalker-core"
version = "0.1.0"
edition = "2021"

[dependencies]
sea-orm.workspace = true
sea-orm-migration.workspace = true
serde.workspace = true
thiserror.workspace = true
chrono.workspace = true
uuid.workspace = true
log.workspace = true
tokio.workspace = true
```

### Domain Crate (`crates/domain/Cargo.toml`)

```toml
[package]
name = "product-stalker-domain"
version = "0.1.0"
edition = "2021"

[dependencies]
product-stalker-core = { path = "../core" }
sea-orm.workspace = true
serde.workspace = true
chrono.workspace = true
uuid.workspace = true
log.workspace = true
tokio.workspace = true

# Scraping dependencies
reqwest = { version = "0.12", features = ["json", "rustls-tls", "gzip", "deflate", "brotli"] }
scraper = "0.23"
url = "2"
headless_chrome = "1.0"
```

---

## Verification

1. **Unit tests**: `cargo test --workspace`
2. **Linting**: `cargo clippy --workspace -- -D warnings`
3. **Coverage**: `cargo llvm-cov --workspace --fail-under-lines 65`
4. **App test**: `pnpm dev:desktop` - verify app starts and works
5. **Frontend integration**: Test product CRUD and availability checks

---

## Benefits After Refactor

- **To start a new project**: Delete `crates/domain/`, create new domain crate
- **Core is reusable**: Could power CLI tools, web backends, etc.
- **Domain is isolated**: No Tauri dependencies in business logic
- **Faster incremental builds**: Only changed crates recompile
