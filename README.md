# Product Stalker

A desktop application for tracking product availability and prices across online stores.

## Product Features

- **Product Tracking** - Add products by URL and monitor their availability
- **Price Monitoring** - Track price changes with daily averages and drop detection
- **Availability Alerts** - Desktop notifications when products come back in stock
- **Background Checks** - Automatic periodic availability checks
- **Multi-Site Support** - Schema.org parsing + site-specific adapters (Chemist Warehouse)
- **System Tray** - Runs in background with quick access menu

## Tech Stack

- **TypeScript** - For type safety and improved developer experience
- **TanStack Router** - File-based routing with full type safety
- **TanStack Query** - Data fetching and caching
- **TanStack Table** - Table/data grid component
- **React Hook Form** - Form handling
- **TailwindCSS** - Utility-first CSS for rapid UI development
- **shadcn/ui** - Reusable UI components
- **Vite** - Build tool and dev server
- **Vitest** - Testing framework
- **Biome** - Linting and formatting
- **Husky** - Git hooks for code quality
- **Tauri 2.x** - Build native desktop applications with Rust backend
- **SeaORM** - Async ORM for SQLite database
- **Turborepo** - Optimized monorepo build system

## Getting Started

First, install the dependencies:

```bash
pnpm install
```

Then, run the development server:

```bash
pnpm run dev
```

Open [http://localhost:3001](http://localhost:3001) in your browser to see the web application.

## Git Hooks and Formatting

- Initialize hooks: `pnpm run prepare`
- Format and lint fix: `pnpm run check`

## Project Structure

```
product-stalker/
├── apps/
│   └── desktop/     # Desktop application (React + TanStack Router + Tauri)
├── packages/
│   ├── config/      # Shared configuration
│   └── env/         # Environment variables (T3 Env)
```

## Available Scripts

- `pnpm run dev`: Start all applications in development mode
- `pnpm run dev:desktop`: Start only the desktop application
- `pnpm run dev:native`: Start native app in development
- `pnpm run build`: Build all applications
- `pnpm run check-types`: Check TypeScript types across all apps
- `pnpm run check`: Run Biome formatting and linting
- `pnpm run prepare`: Initialize Husky git hooks
- `pnpm -F desktop test`: Run tests
- `pnpm -F desktop test:ui`: Run tests with UI
- `cd apps/desktop && pnpm run desktop:dev`: Start Tauri desktop app in development
- `cd apps/desktop && pnpm run desktop:build`: Build Tauri desktop app

## Testing

### Frontend (Vitest)

```bash
pnpm -F desktop test:run          # Run all tests
pnpm -F desktop test:unit         # Run unit tests only
pnpm -F desktop test:integration  # Run integration tests only
pnpm -F desktop test:ui           # Interactive test UI
```

### Rust Backend

```bash
cd apps/desktop/src-tauri
cargo test                        # Run all tests (~420 tests)
cargo test <module_name>          # Run tests for specific module
cargo clippy -- -D warnings       # Linter
cargo fmt --check                 # Check formatting
```

## Documentation

- [Developer guides](./docs/guides/) - Setup and maintenance procedures
- [Implementation plans](./docs/plans/) - Technical specifications
- [Architecture decisions](./docs/decisions/) - ADRs
