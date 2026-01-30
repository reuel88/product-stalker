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

Releases are created through a three-stage draft-first workflow. The automated stages create a draft release, allowing you to review build artifacts before manually publishing.

### Stage 1: Create Release Tag

1. Go to **Actions** > **"Create Release"** workflow
2. Click **"Run workflow"**
3. **Ensure "Branch: main" is selected** in the dropdown
   - The workflow enforces this and will fail if run from another branch
   - This prevents accidental releases from feature branches
4. Select the version bump type from the dropdown:
   - **patch** - Bug fixes and minor changes (1.2.3 → 1.2.4)
   - **minor** - New features, backwards compatible (1.2.3 → 1.3.0)
   - **major** - Breaking changes (1.2.3 → 2.0.0)
5. Click **"Run workflow"** to start
6. The workflow automatically:
   - Reads the current version from `tauri.conf.json`
   - Calculates the new version based on your selection
   - Updates version in `tauri.conf.json` and `Cargo.toml`
   - Commits the version bump to main
   - Creates and pushes a `v{version}` tag

### Stage 2: Build and Create Draft Release

7. The tag push triggers the **"Release"** workflow, which builds the app for all platforms:
   - Windows (x64)
   - macOS Intel (x64)
   - macOS Apple Silicon (ARM64)
   - Linux (x64)
8. Once all builds complete, a **draft release** is created with all artifacts attached
   - The workflow uses `releaseDraft: true`, so releases are never auto-published
   - Draft releases are only visible to maintainers
   - This gives you a chance to verify builds before publishing

### Stage 3: Review and Publish

9. Go to **Releases** page on GitHub
10. Click on the draft release to review it
11. Verify the assets include all expected installers:
    - `.msi` / `.exe` for Windows
    - `.dmg` for macOS (both Intel and ARM)
    - `.AppImage` / `.deb` for Linux
    - `latest.json` for the auto-updater
12. Optionally edit the release notes to add changelog details
13. Click **"Publish release"** to convert the draft to a public release
    - This is the only way releases become public (drafts are never auto-published)
    - Once published, the release is visible to all users
    - The auto-updater will detect the new version and notify users

### Signing Keys Setup (One-Time)

> **Why signing keys?** The app uses cryptographic signing to enable secure auto-updates. When users update, the app verifies the download hasn't been tampered with. Without valid signing keys, releases cannot be built.

This setup is only needed once when configuring a new repository fork:

1. Generate signing keys:
   ```bash
   npx tauri signer generate -w ~/.tauri/product-stalker.key
   ```

2. Add secrets to GitHub (Settings > Secrets > Actions):
   - `TAURI_SIGNING_PRIVATE_KEY`: Contents of the private key file
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: Password used during generation

3. Update `apps/desktop/src-tauri/tauri.conf.json` with the public key

See `docs/MAINTAINERS.md` for detailed signing key management, troubleshooting, and key rotation procedures.
