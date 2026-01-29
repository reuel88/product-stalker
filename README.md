# product-stalker

This project was created with [Better-T-Stack](https://github.com/AmanVarshney01/create-better-t-stack), a modern TypeScript stack that combines React, TanStack Router, and more.

## Features

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
- **Tauri** - Build native desktop applications
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
│   └── web/         # Frontend application (React + TanStack Router + Tauri)
├── packages/
│   ├── config/      # Shared configuration
│   └── env/         # Environment variables (T3 Env)
```

## Available Scripts

- `pnpm run dev`: Start all applications in development mode
- `pnpm run dev:web`: Start only the web application
- `pnpm run dev:native`: Start native app in development
- `pnpm run build`: Build all applications
- `pnpm run check-types`: Check TypeScript types across all apps
- `pnpm run check`: Run Biome formatting and linting
- `pnpm run prepare`: Initialize Husky git hooks
- `pnpm -F web test`: Run tests
- `pnpm -F web test:ui`: Run tests with UI
- `cd apps/web && pnpm run desktop:dev`: Start Tauri desktop app in development
- `cd apps/web && pnpm run desktop:build`: Build Tauri desktop app

## Testing

This project uses [Vitest](https://vitest.dev/) for testing. Run tests with:

```bash
pnpm -F web test
```

For an interactive test UI:

```bash
pnpm -F web test:ui
```
