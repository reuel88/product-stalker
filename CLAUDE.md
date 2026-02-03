# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
```bash
pnpm install              # Install dependencies
pnpm dev:desktop          # Start Tauri desktop app in dev mode (recommended)
pnpm dev                  # Start all apps in dev mode (web only, no Tauri backend)
```

### Quality Checks (Frontend)
```bash
pnpm run check            # Biome lint & format (auto-fix)
pnpm run check-types      # TypeScript type checking
pnpm -F desktop test:run  # Run all tests
pnpm -F desktop test:unit # Run unit tests only
pnpm -F desktop test:integration  # Run integration tests only
```

### Quality Checks (Rust Backend)
```bash
cd apps/desktop/src-tauri
cargo fmt --check         # Check formatting (cargo fmt to auto-fix)
cargo clippy -- -D warnings  # Linter - treats warnings as errors
cargo test                # Run tests
cargo llvm-cov --fail-under-lines 50  # Coverage check (Windows)
```

### Coverage Thresholds
- **Frontend (Vitest):** 80% for statements, branches, functions, lines
- **Rust:** 50% line coverage

## Architecture

### Monorepo Structure (Turborepo + pnpm)
- `apps/desktop/` - Main Tauri desktop app
  - `src/` - React/TypeScript frontend (TanStack Router, Query, Table)
  - `src-tauri/` - Rust backend (Tauri 2.x, SeaORM, SQLite)
- `packages/config/` - Shared TypeScript config
- `packages/env/` - T3 Env environment variables

### Backend Layered Architecture (Rust)
```
Tauri Commands (IPC) → Services (business logic) → Repositories (data access) → SeaORM Entities → SQLite
```

- **Commands** (`src/commands/`): IPC handlers only - no business logic
- **Services** (`src/services/`): Validation, orchestration, business rules
- **Repositories** (`src/repositories/`): Pure CRUD operations
- **Entities** (`src/entities/`): SeaORM models (auto-generated from migrations)

### Frontend Module Pattern
Each feature module in `src/modules/` follows: `hooks/`, `types.ts`, `ui/components/`, `ui/views/`

## Code Style

### Biome (JS/TS)
- Indentation: Tabs
- Quotes: Double quotes
- Tailwind class sorting enabled (clsx, cva, cn functions)

### Rust
- Clippy treats all warnings as errors
- Use `AppError` for all error returns (defined in `src/error.rs`)
- Connection pool via `db.conn()` - never use Mutex for DB connections

### Control Flow
- Prefer early returns over nested if statements (guard clauses)

## Branch Naming
- `feat/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation
- `refactor/` - Code refactoring
- `chore/` - Maintenance

## Key Patterns

### Rust Database Access
```rust
// Correct: use connection pool directly
let products = ProductService::get_all(db.conn()).await?;

// Wrong: don't use Mutex
let conn = db.conn.lock().unwrap();  // Blocks async runtime!
```

### Adding New Database Entities
1. Create migration in `src-tauri/src/migrations/`
2. Entity files are in `src/entities/` - follow existing patterns
3. Add repository in `src/repositories/`
4. Add service in `src/services/`
5. Expose via Tauri command in `src/commands/`

## Dependencies Note
When bumping `@biomejs/biome` in `package.json`, update the `@biomejs+biome@X.Y.Z` segment in `.vscode/settings.json` biome.lsp.bin paths.
