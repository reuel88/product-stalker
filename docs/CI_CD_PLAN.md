# CI/CD Strategy for Product Stalker

## Overview
Implement GitHub Flow with full quality checks on PRs and cross-platform builds triggered via manual release workflow.

## Branching Strategy: GitHub Flow

```
main (protected) ─────●─────●─────●─────● (releases tagged here)
                      │     │     │
feature/xyz ──────────┘     │     │
feature/abc ────────────────┘     │
fix/bug-123 ──────────────────────┘
```

**Rules:**
- `main` is always deployable
- Feature branches created from `main`
- PRs required to merge to `main`
- Releases triggered manually via GitHub Actions UI

---

## Files to Create

### 1. `.github/workflows/ci.yml` - Quality Checks
Runs on: All PRs and pushes to main

**Jobs:**
| Job | Steps | Blocks PR |
|-----|-------|-----------|
| lint | Biome check | Yes |
| typecheck | `tsc --noEmit` via turbo | Yes |
| test-js | Vitest run | Yes |
| test-rust | `cargo clippy`, `cargo fmt --check`, `cargo test` | Yes |

### 2. `.github/workflows/release.yml` - Cross-Platform Builds
Runs on: Version tags (`v*`)

**Build Matrix:**
| Platform | Runner | Outputs |
|----------|--------|---------|
| Windows x64 | `windows-latest` | `.exe`, `.msi` |
| macOS Intel | `macos-13` | `.dmg` |
| macOS ARM64 | `macos-latest` | `.dmg` |
| Linux x64 | `ubuntu-22.04` | `.AppImage`, `.deb` |

**Release Job:**
- Creates draft GitHub Release
- Attaches all platform installers
- Auto-generates release notes from commits

### 3. `.github/workflows/create-release.yml` - Manual Release Trigger
Runs on: Manual trigger (workflow_dispatch)

**Inputs:**
- `version`: Version number (e.g., `0.2.0`)
- `release_type`: patch | minor | major (optional, for auto-increment)

**Steps:**
1. Checkout repository
2. Update version in `apps/web/src-tauri/tauri.conf.json`
3. Update version in `apps/web/src-tauri/Cargo.toml`
4. Commit changes with message "Release v{version}"
5. Create and push tag `v{version}`
6. This triggers `release.yml` automatically

### 4. `CONTRIBUTING.md` - Developer Documentation
Beginner-friendly guide covering:
- How to set up the development environment
- How to create a feature branch
- How to make and test changes
- How to submit a pull request
- When and how to trigger a release

---

## CONTRIBUTING.md Content Outline

```markdown
# Contributing to Product Stalker

## Getting Started
- Prerequisites (Node.js, pnpm, Rust)
- Clone and install dependencies
- Run the app locally

## Making Changes

### Step 1: Create a Feature Branch
- Always branch from `main`
- Use descriptive names: `feature/add-dark-mode`, `fix/login-crash`

### Step 2: Make Your Changes
- Write code
- Run local checks before committing
- Commit with clear messages

### Step 3: Open a Pull Request
- Push your branch
- Open PR against `main`
- Wait for CI checks to pass
- Request review

### Step 4: Merge
- Squash and merge when approved
- Delete your feature branch

## Releasing (Maintainers Only)

### When to Release
- After significant features or bug fixes are merged
- Version types explained (patch, minor, major)

### How to Release
- Step-by-step GitHub Actions trigger guide
- What happens after triggering
- Publishing the draft release
```

---

## CI Workflow Details (`ci.yml`)

```yaml
Jobs (run in parallel):
  lint:
    - pnpm install
    - pnpm run check (biome)

  typecheck:
    - pnpm install
    - pnpm run check-types (turbo)

  test-js:
    - pnpm install
    - pnpm -F web test:run (vitest)

  test-rust:
    - Install Rust
    - cargo fmt --check (in apps/web/src-tauri)
    - cargo clippy -- -D warnings
    - cargo test
```

**Caching:**
- pnpm store cached by `pnpm-lock.yaml` hash
- Cargo registry cached by `Cargo.lock` hash

---

## Release Workflow Details (`release.yml`)

```yaml
Trigger: tags matching 'v*'

Jobs:
  build (matrix: 4 platforms):
    - Checkout
    - Install system deps (Linux only)
    - Setup Node.js + pnpm
    - Setup Rust + target
    - pnpm install
    - Build with tauri-action
    - Upload artifacts

  release:
    needs: build
    - Download all artifacts
    - Create draft GitHub Release
    - Attach installers
```

---

## Branch Protection Rules (GitHub Settings)

Configure for `main` branch:
- [x] Require pull request before merging
- [x] Require status checks to pass (lint, typecheck, test-js, test-rust)
- [x] Require branches to be up to date
- [ ] Require signed commits (optional)

---

## Implementation Order

1. **Create `.github/workflows/ci.yml`** - Quality gate for PRs
2. **Create `.github/workflows/release.yml`** - Cross-platform builds on tags
3. **Create `.github/workflows/create-release.yml`** - Manual trigger workflow
4. **Create `CONTRIBUTING.md`** - Developer documentation
5. **Configure branch protection** - Enforce checks on main
6. **Test the workflow** - Run a test release

---

## Verification

1. **Test CI:** Create a feature branch, make a change, open PR
   - Verify all 4 CI jobs run and pass
2. **Test Release:**
   - Go to Actions → Create Release → Run workflow
   - Enter version `0.1.1`
   - Verify version files updated, tag created
   - Verify all 4 platform builds succeed
   - Verify draft release has all installers
3. **Test Installers:** Download and run on each platform
4. **Review docs:** Read CONTRIBUTING.md as a new developer

---

## Files Summary

| File | Purpose |
|------|---------|
| `.github/workflows/ci.yml` | Quality checks on PRs |
| `.github/workflows/release.yml` | Cross-platform builds on tags |
| `.github/workflows/create-release.yml` | Manual button to trigger releases |
| `CONTRIBUTING.md` | Developer guide for contributions and releases |

## Key Project Files (Modified by release workflow)

- `apps/web/src-tauri/tauri.conf.json` - App version
- `apps/web/src-tauri/Cargo.toml` - Rust package version

## Key Project Files (Reference)

- `apps/web/package.json` - Build scripts
- `biome.json` - Linting rules
- `turbo.json` - Task orchestration

---

## Auto-Update Support

The desktop app uses `tauri-plugin-updater` to check for and install updates from GitHub Releases.

### Signing Key Setup (One-time)

Generate a signing key pair locally:

```bash
npx tauri signer generate -w ~/.tauri/product-stalker.key
```

This command will:
1. Create a private key file at `~/.tauri/product-stalker.key`
2. Display the public key in the terminal

**Important:** Copy the displayed public key and update `apps/desktop/src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "pubkey": "PASTE_YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

### Required GitHub Secrets

Add these secrets in your repository settings (Settings > Secrets and variables > Actions):

| Secret | Description |
|--------|-------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Contents of `~/.tauri/product-stalker.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password used during key generation (or empty string if none) |

### Release Workflow Requirements

The `release.yml` workflow must be updated to:

1. **Sign the builds** using the `TAURI_SIGNING_PRIVATE_KEY` secret
2. **Generate the update manifest** (`latest.json`) containing:
   - Version number
   - Platform-specific download URLs
   - Signatures for each platform
3. **Upload `latest.json`** to the release assets

Example tauri-action configuration for signing:

```yaml
- uses: tauri-apps/tauri-action@v0
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
  with:
    tagName: v__VERSION__
    releaseName: 'Product Stalker v__VERSION__'
    releaseBody: 'See the assets to download this version and install.'
    releaseDraft: true
    prerelease: false
```

### Update Manifest Format

The `latest.json` file is automatically generated by `tauri-action` when signing is configured. It follows this format:

```json
{
  "version": "0.2.0",
  "notes": "Release notes here",
  "pub_date": "2024-01-15T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../releases/download/v0.2.0/product-stalker_0.2.0_x64-setup.nsis.zip"
    },
    "darwin-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../releases/download/v0.2.0/product-stalker_0.2.0_x64.app.tar.gz"
    },
    "darwin-aarch64": {
      "signature": "...",
      "url": "https://github.com/.../releases/download/v0.2.0/product-stalker_0.2.0_aarch64.app.tar.gz"
    },
    "linux-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../releases/download/v0.2.0/product-stalker_0.2.0_amd64.AppImage.tar.gz"
    }
  }
}
```

### How Updates Work

1. User clicks "Check for Updates" in Settings
2. App fetches `latest.json` from GitHub Releases
3. If a newer version exists, user sees "Update Now" button
4. User clicks "Update Now" to download and install
5. App restarts with the new version
