# Template-Readiness Reviewer - Project Memory

## Architecture Summary
- Tauri 2.x desktop app: Rust backend (SeaORM/SQLite) + React/TS frontend
- Monorepo: Turborepo + pnpm, `apps/desktop/`, `packages/{config,env}/`
- Rust workspace: `crates/core/` (infra) + `crates/domain/` (product-specific) + `src/` (Tauri wiring)
- Frontend: TanStack Router (file-based), Query, Table; modules in `src/modules/{feature}/`

## Key Template-Readiness Findings (Feb 2026 audit)
- **Score: 7/10** - Good intentional separation, but coupling violations prevent clean extraction
- Existing guide at `docs/guides/using-as-template.md` with search-replace patterns
- Refactoring plan that created the crate split: `docs/plans/2025-02-rust-workspace-refactor.md`

## Critical Coupling Violations
1. **Core AppError has domain variants**: `Scraping`, `BotProtection`, `Http`, `HttpStatus` in `crates/core/src/error.rs`. Pulls `reqwest` into core.
2. **Core Settings has domain keys**: `enable_headless_browser`, `background_check_*` in `crates/core/src/services/setting_service.rs`
3. **lib.rs hard-codes all commands**: No separation between domain/infra in `src/lib.rs` invoke_handler
4. **Shared palette-provider imports settings module**: `src/modules/shared/providers/palette-provider.tsx` line 9
5. **price-utils.ts in lib/ imports from products module**: `src/lib/price-utils.ts` lines 1-5

## Registration Patterns
- Commands: flat list in `tauri::generate_handler![]` macro in `src/lib.rs`
- Migrations: `fn migrations() -> Vec<Box<dyn MigrationTrait>>` pattern, combined in `src/db/connection.rs` AppMigrator
- Frontend routes: TanStack Router file-based routing, auto-generated `routeTree.gen.ts`
- Settings: EAV store with `ScopedSettingsReader`/`SettingsHelpers` in core

## Clean Separability
- `crates/domain/` is well-isolated; only imported by `src/` layer
- `crates/core/` does NOT import domain (dependency rule respected)
- Frontend products module self-contained (except price-utils.ts in lib/)
- Frontend settings module mostly self-contained (except palette-provider coupling)
- `src/modules/shared/` is the app shell layer; should not import feature modules

## Config Files Needing Project-Specific Updates
- `package.json` (root): name, @product-stalker/* scopes
- `tauri.conf.json`: productName, identifier, updater endpoints, window title
- `Cargo.toml` files: crate names contain "product-stalker"
- `src/db/connection.rs`: hard-coded "product_stalker.db" filename
- `src/routes/__root.tsx`: title meta tag
- `.github/workflows/release.yml`: release name template
