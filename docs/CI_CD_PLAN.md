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
