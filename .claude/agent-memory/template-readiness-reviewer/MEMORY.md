# Template-Readiness Reviewer - Project Memory

## Architecture Summary
- Tauri 2.x desktop app: Rust backend (SeaORM/SQLite) + React/TS frontend
- Monorepo: Turborepo + pnpm, `apps/desktop/`, `packages/{config,env}/`
- Rust workspace: `crates/core/` (infra) + `crates/domain/` (product-specific) + `src/` (Tauri wiring)
- Frontend: TanStack Router (file-based), Query, Table; modules in `src/modules/{feature}/`

## Key Template-Readiness Findings (Feb 2026 audit - v2)
- **Score: 8/10** - Significant improvement since last audit
- Previous domain-leaking AppError variants consolidated into generic `External` variant
- Previous domain-leaking settings keys moved to domain's `DomainSettingService`
- Core crate `error.rs` and `setting_service.rs` are now clean
- `reqwest` removed from core Cargo.toml
- Existing guide at `docs/guides/using-as-template.md`
- Refactoring plan: `docs/plans/2025-02-rust-workspace-refactor.md`

## Remaining Coupling Violations (Feb 2026)
1. **Root Cargo.toml has domain deps**: `reqwest`, `scraper`, `url`, `headless_chrome` only needed by domain
2. **lib.rs hard-codes all commands**: No separation in `src/lib.rs` invoke_handler (lines 130-150)
3. **lib.rs hard-codes background checker**: `background::spawn_background_checker` at line 114
4. **Core migrations include domain backfill**: `m20250207` references domain setting keys
5. **Core migrations include domain columns**: `m20240104` (background_check), `m20240105` (headless_browser)
6. **Settings command merges core+domain**: `src/commands/settings.rs` imports DomainSettingService
7. **Frontend Settings interface includes domain fields**: `useSettings.ts`
8. **Frontend constants mix domain/infra**: `api.ts`, `queryKeys.ts`, `messages.ts`
9. **Frontend header hard-codes /products nav**: `header.tsx` line 9
10. **Frontend test mocks contain domain types**: `data.ts`
11. **Home view has product branding**: ASCII art in `home-view.tsx`

## Registration Patterns
- Commands: flat list in `tauri::generate_handler![]` macro
- Migrations: `fn migrations()` pattern, combined in `src/db/connection.rs` AppMigrator
- Frontend routes: file-based routing, auto-generated `routeTree.gen.ts`
- Settings: EAV store; domain settings in separate DomainSettingService

## Clean Separability (confirmed)
- `crates/core/` does NOT import domain
- `crates/domain/` only imported by `src/` layer
- PaletteProvider is clean -- receives props, no feature module imports
- Frontend products module fully self-contained
- `src/modules/shared/` does not import feature modules
