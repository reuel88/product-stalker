# Contributing to Product Stalker

## Prerequisites

- **Node.js** 20+
- **pnpm** 10+
- **Rust** 1.77+
- Platform-specific dependencies:
  - **Linux**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools with C++ workload

## Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/reuel88/product-stalker.git
   cd product-stalker
   ```

2. Install dependencies:
   ```bash
   pnpm install
   ```

3. Start development:
   ```bash
   pnpm dev:desktop
   ```

## Branch Naming

Use descriptive branch names with prefixes:

- `feat/` - New features (e.g., `feat/add-notifications`)
- `fix/` - Bug fixes (e.g., `fix/login-error`)
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `chore/` - Maintenance tasks

## Pull Request Workflow

1. Create a feature branch from `main`
2. Make your changes
3. Run quality checks locally:
   ```bash
   pnpm run check        # Biome lint
   pnpm run check-types  # TypeScript
   pnpm -F desktop test:run  # Tests
   ```
4. For Rust changes:
   ```bash
   cd apps/desktop/src-tauri
   cargo fmt --check           # Check code formatting (use `cargo fmt` to auto-fix)
   cargo clippy -- -D warnings # Run linter - treats all warnings as errors
   cargo test                  # Run unit and integration tests
   ```
   To check code coverage locally (requires `cargo install cargo-tarpaulin`):
   ```bash
   cargo tarpaulin --ignore-tests --fail-under 50
   ```
   CI enforces a minimum 50% coverage threshold.

   > **Note (Windows users):** `cargo-tarpaulin` has limited Windows support and may fail with parser errors. Use `cargo-llvm-cov` as an alternative:
   > ```bash
   > cargo install cargo-llvm-cov
   > cargo llvm-cov --fail-under-lines 50
   > ```
   > CI runs on Linux where tarpaulin works reliably.
5. Push your branch and open a PR against `main`
6. All CI checks must pass before merging

## Testing

### Coverage Requirements

| Area       | Threshold |
|------------|-----------|
| Statements | 80%       |
| Branches   | 80%       |
| Functions  | 80%       |
| Lines      | 80%       |

### Running Tests Locally

**Frontend (Vitest):**
```bash
pnpm -F desktop test:run       # Run tests
pnpm -F desktop test:coverage  # Run with coverage report
pnpm -F desktop test:unit      # Run unit tests only
pnpm -F desktop test:integration  # Run integration tests only
```

**Rust:**
```bash
cd apps/desktop/src-tauri
cargo test                     # Run tests
cargo tarpaulin --ignore-tests --fail-under 50  # Coverage (50% threshold)
```

> **Windows users:** If tarpaulin fails, use `cargo-llvm-cov` instead:
> ```bash
> cargo llvm-cov --fail-under-lines 50
> ```

## Release Process (Maintainers)

Releases are created through the GitHub Actions workflow:

1. Go to Actions > "Create Release"
2. Click "Run workflow"
3. **Ensure "Branch: main" is selected** (the workflow only runs from main)
4. Enter the version number (e.g., `0.2.0`)
5. The workflow will:
   - Update version in `tauri.conf.json` and `Cargo.toml`
   - Commit the changes
   - Create and push a version tag
6. The tag triggers the Release workflow which builds for all platforms
7. Review and publish the draft release on GitHub

### Signing Keys Setup

For maintainers setting up a new repository:

1. Generate signing keys:
   ```bash
   npx tauri signer generate -w ~/.tauri/product-stalker.key
   ```

2. Add secrets to GitHub (Settings > Secrets > Actions):
   - `TAURI_SIGNING_PRIVATE_KEY`: Contents of the private key file
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: Password used during generation

3. Update `apps/desktop/src-tauri/tauri.conf.json` with the public key

See `docs/MAINTAINER.md` for detailed signing key management.
