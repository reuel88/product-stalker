# Using Product Stalker as a Template

This guide explains how to fork Product Stalker as a base template for building new Tauri + React desktop applications.

## Overview

Product Stalker provides a solid foundation with:
- Tauri 2.x + React + TypeScript frontend
- Rust backend with SeaORM + SQLite
- Monorepo structure (Turborepo + pnpm)
- System tray, notifications, auto-updates
- Background task infrastructure
- CI/CD with GitHub Actions

## Quick Start Checklist

- [ ] Fork/clone the repository
- [ ] Run global find-and-replace (see [Search Patterns](#search-patterns))
- [ ] Update configuration files (see sections below)
- [ ] Replace icon assets
- [ ] Review and adapt database migrations
- [ ] Remove product-specific features
- [ ] Generate new signing keys
- [ ] Test build on target platforms

---

## 1. Package Names

### Root package.json
```diff
- "name": "product-stalker"
+ "name": "your-app-name"
```

Update workspace dependencies:
```diff
- "@product-stalker/env": "workspace:*"
- "@product-stalker/config": "workspace:*"
+ "@your-app/env": "workspace:*"
+ "@your-app/config": "workspace:*"
```

### apps/desktop/package.json
Update scoped package references to match root.

### packages/env/package.json
```diff
- "name": "@product-stalker/env"
+ "name": "@your-app/env"
```

### packages/config/package.json
```diff
- "name": "@product-stalker/config"
+ "name": "@your-app/config"
```

---

## 2. Rust Crates

### apps/desktop/src-tauri/Cargo.toml

> **Note:** The root crate is named `app` (Tauri default). Consider renaming it to match your project (e.g., `your-app`).

```diff
  [package]
- name = "app"
+ name = "your-app"
- description = "A Tauri App"
+ description = "Your app description"
- authors = ["you"]
+ authors = ["Your Name <email@example.com>"]
+ license = "MIT"
+ repository = "https://github.com/you/your-app"

  [dependencies]
- product-stalker-core = { path = "crates/core" }
- product-stalker-domain = { path = "crates/domain" }
+ your-app-core = { path = "crates/core" }
+ your-app-domain = { path = "crates/domain" }
```

### crates/core/Cargo.toml
```diff
- name = "product-stalker-core"
+ name = "your-app-core"
```

### crates/domain/Cargo.toml
```diff
- name = "product-stalker-domain"
+ name = "your-app-domain"

  [dependencies]
- product-stalker-core = { path = "../core" }
+ your-app-core = { path = "../core" }
```

### src/lib.rs
```diff
- pub(crate) use product_stalker_core as core;
- pub(crate) use product_stalker_domain as domain;
+ pub(crate) use your_app_core as core;
+ pub(crate) use your_app_domain as domain;
```

---

## 3. Tauri Configuration

### apps/desktop/src-tauri/tauri.conf.json

```diff
  {
-   "productName": "product-stalker",
+   "productName": "Your App Name",
    "version": "0.1.0",
-   "identifier": "com.productstalker.desktop",
+   "identifier": "com.yourcompany.yourapp",
    ...
    "windows": [
      {
-       "title": "Product Stalker",
+       "title": "Your App Name",
-       "backgroundColor": "#667eea"
+       "backgroundColor": "#your-brand-color"
      }
    ],
    ...
    "plugins": {
      "updater": {
        "endpoints": [
-         "https://github.com/reuel88/product-stalker/releases/latest/download/latest.json"
+         "https://github.com/you/your-app/releases/latest/download/latest.json"
        ]
      }
    }
  }
```

---

## 4. Frontend Assets

### apps/desktop/index.html
```diff
- <title>product-stalker</title>
+ <title>Your App Name</title>
```

### apps/desktop/src/routes/__root.tsx
```diff
- title: "product-stalker"
+ title: "Your App Name"
```

### apps/desktop/public/splash.html
Update the logo text and gradient colors to match your branding.

---

## 5. Database

### apps/desktop/src-tauri/src/db/connection.rs
```diff
- Ok(app_data_dir.join("product_stalker.db"))
+ Ok(app_data_dir.join("your_app.db"))
```

### Migrations
Located in `crates/core/src/migrations/`. Options:

1. **Keep** - If your app tracks similar data
2. **Modify** - Rename tables/columns for your domain
3. **Replace** - Delete existing migrations and create new ones

Product-specific migrations to review:
- `create_products_table` - rename to your entity
- `create_availability_checks_table` - only if tracking availability
- `add_price_tracking` - only if tracking prices

---

## 6. Icons

Replace all files in `apps/desktop/src-tauri/icons/`:

| File | Purpose | Size |
|------|---------|------|
| `icon.png` | System tray, general | 512x512 |
| `icon.ico` | Windows | Multi-size |
| `icon.icns` | macOS | Multi-size |
| `32x32.png` | Small icon | 32x32 |
| `128x128.png` | Medium icon | 128x128 |
| `128x128@2x.png` | Retina | 256x256 |
| `Square*.png` | Windows Store | Various |

Use [tauri-icon](https://tauri.app/v1/guides/features/icons/) to generate from a single source.

---

## 7. GitHub Actions

### .github/workflows/release.yml
```diff
- releaseName: 'Product Stalker v__VERSION__'
+ releaseName: 'Your App Name v__VERSION__'
```

Update the release notes template in the same file.

### Signing Keys
Generate new keys for your app:
```bash
pnpm tauri signer generate -w ~/.tauri/your-app.key
```

Add to GitHub repository secrets:
- `TAURI_SIGNING_PRIVATE_KEY` - contents of the .key file
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` - password you set

---

## 8. Product-Specific Code to Remove

These features are specific to product availability tracking:

### Remove entirely:
- `crates/domain/src/services/scraper/` - HTML parsing for product pages
- Chemist Warehouse adapter
- Schema.org parsing
- Headless browser integration (if not needed)

### Rename/adapt:
- `crates/domain/src/repositories/product_repository.rs` → `YourEntityRepository`
- `crates/domain/src/services/product_service.rs` → `YourEntityService`
- `crates/domain/src/services/availability_service.rs` → Your feature service
- `crates/domain/src/services/notification_service.rs` → Your notification logic
- `src/tauri_services/availability_service.rs` → Your Tauri-specific wrapper (or remove)
- `src/background/availability_checker.rs` → Your background task (or remove)
- Frontend modules in `src/modules/products/`

### Keep:
- Settings system (`crates/core/`)
- Notification infrastructure (`crates/core/src/services/notification_helpers.rs`)
- System tray integration (`src/plugins/`)
- Background task runner pattern (`src/background/`)
- Database connection handling (`src/db/`, `crates/core/src/db/`)
- Auto-updater

---

## Search Patterns

Run these find-and-replace operations across the codebase:

| Find | Replace |
|------|---------|
| `product-stalker-core` | `your-app-core` |
| `product-stalker-domain` | `your-app-domain` |
| `product_stalker_core` | `your_app_core` |
| `product_stalker_domain` | `your_app_domain` |
| `@product-stalker/` | `@your-app/` |
| `com.productstalker` | `com.yourcompany.yourapp` |
| `Product Stalker` | `Your App Name` |
| `product-stalker` | `your-app-name` |
| `product_stalker` | `your_app_name` |
| `reuel88/product-stalker` | `you/your-app` |

---

## Verification

After making changes:

```bash
# Install dependencies
pnpm install

# Check TypeScript
pnpm run check-types

# Check Rust
cd apps/desktop/src-tauri
cargo check
cargo clippy -- -D warnings

# Run tests
cargo test
cd ../../..
pnpm -F desktop test:run

# Development build
pnpm dev:desktop
```

---

## Architecture Reference

```
your-app/
├── apps/desktop/
│   ├── src/                    # React frontend
│   │   ├── modules/            # Feature modules
│   │   ├── routes/             # TanStack Router pages
│   │   └── lib/                # Shared utilities
│   └── src-tauri/
│       ├── src/
│       │   ├── commands/       # Tauri IPC handlers
│       │   ├── background/     # Background tasks
│       │   ├── tauri_services/ # Tauri-specific wrappers (event emission)
│       │   ├── plugins/        # System tray, etc.
│       │   └── db/             # Tauri-specific database connection
│       └── crates/
│           ├── core/           # Infrastructure (entities, repos, migrations)
│           └── domain/         # Business logic (services, scraper)
├── packages/
│   ├── config/                 # Shared TS config
│   └── env/                    # Environment variables
└── docs/
    ├── decisions/              # ADRs
    ├── guides/                 # This guide
    └── plans/                  # Implementation specs
```

See [CLAUDE.md](/CLAUDE.md) for development commands and code style guidelines.
