# Phased Dependency Updates

**Date:** 2026-05-26
**Status:** Draft
**Branch:** `chore/dependency-updates` (create from `main`)

## Context

`pnpm outdated -r` reports ~30 outdated packages spanning patch bumps, minor bumps, and several majors (TypeScript 5→6, Vite 6→8, Vitest 3→4, jsdom 26→29, `@types/node` 22→25, `lucide-react` 0.x→1.x, `shadcn` 3→4, `lint-staged` 16→17, `@vitejs/plugin-react` 4→6). Doing this in one commit risks tangling unrelated breakages and makes any rollback an all-or-nothing affair. This plan splits the work into ~11 small, independently testable phases, each producing its own commit so any regression can be bisected to a single ecosystem.

Versions referenced below come from the `pnpm outdated -r` snapshot taken at the start of this task; re-run the command at the start of each phase to pick up anything published since.

## Strategy

- **One phase = one commit = one PR-worthy change.** Each phase must pass the full local verification gate (Verification Gate below) before the next phase begins.
- **Group by ecosystem, not by file.** TanStack, Tauri, Tailwind, Vite/Vitest are each upgraded together because their internal types and plugins are tightly coupled.
- **Patches first, majors last.** Safe wins ship early so the diff for risky phases stays small and easy to review.
- **Catalog-managed deps update in `pnpm-workspace.yaml`.** `dotenv`, `zod`, and `typescript` are in the catalog — bump them there, not in individual `package.json` files.
- **Don't change unrelated config.** Each phase should touch lockfile + the relevant `package.json` / catalog entry only. The two exceptions are documented (Biome → `.vscode/settings.json`; Tailwind → potential config audit).
- **`tauri.conf.json` + `Cargo.toml` are out of scope.** This plan covers Node packages only. Rust crate updates are tracked separately.

## Branch Setup

```bash
git checkout main
git pull
git checkout -b chore/dependency-updates
```

Each phase below is a separate commit on this branch. PR can be opened after Phase 1 lands and updated as phases stack, or each phase can be its own PR — caller's choice.

## Verification Gate (run after every phase)

```bash
pnpm install                           # Refresh lockfile + node_modules
pnpm run check                         # Biome lint/format
pnpm run check-types                   # tsc --noEmit across workspace
pnpm -F desktop test:run               # Vitest (unit + lib + integration)
pnpm -F desktop build                  # Vite production build
pnpm dev:desktop                       # Smoke-test the running app — close after verifying it boots and a route renders
```

For Rust-touching phases (only Tauri, Phase 5), also run from `apps/desktop/src-tauri`:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

If any step fails, **stop and fix in the same phase** — do not roll forward into the next phase with a broken gate.

## Phases

### Phase 1 — Patch bumps (zero-risk)

Bumps within the same minor; no API surface changes expected.

| Package | From | To | Where |
|---|---|---|---|
| `react` | 19.2.3 | 19.2.6 | `apps/desktop` |
| `react-dom` | 19.2.3 | 19.2.6 | `apps/desktop` |
| `@types/react` (dev) | 19.2.7 | 19.2.15 | `apps/desktop` |
| `postcss` (dev) | 8.5.6 | 8.5.15 | `apps/desktop` |
| `@tauri-apps/plugin-opener` | 2.5.3 | 2.5.4 | `apps/desktop` |
| `@t3-oss/env-core` | 0.13.10 | 0.13.11 | `packages/env` |

**Commit message:** `chore(deps): bump patch versions across workspace`

### Phase 2 — Low-risk minor bumps (utility libs)

Libraries with small, additive minor releases.

| Package | From | To | Where |
|---|---|---|---|
| `dotenv` | 17.2.3 | 17.4.2 | catalog (`pnpm-workspace.yaml`) |
| `zod` | 4.3.6 | 4.4.3 | catalog |
| `tailwind-merge` | 3.4.0 | 3.6.0 | `apps/desktop` |
| `@hookform/resolvers` | 5.2.2 | 5.4.0 | `apps/desktop` |
| `@base-ui/react` | 1.1.0 | 1.5.0 | `apps/desktop` |
| `recharts` | 3.7.0 | 3.8.1 | `apps/desktop` |

After bumping, skim `@base-ui/react` and `recharts` changelogs for any deprecation warnings — those two render UI so any breakage will show up in the dev-server smoke test.

**Commit message:** `chore(deps): bump utility libraries to latest minor`

### Phase 3 — Build tooling minors

| Package | From | To | Where |
|---|---|---|---|
| `@biomejs/biome` (dev) | 2.3.13 | 2.4.15 | root |
| `turbo` (dev) | 2.7.6 | 2.9.14 | root |

**Must also do (per CLAUDE.md):** update the `@biomejs+biome@X.Y.Z` segment in `.vscode/settings.json` (biome.lsp.bin paths) to match the new version. After bump, run `pnpm run check` — if Biome 2.4 introduced new lint rules, fix or disable them in `biome.json` *in this same commit* so the gate stays green.

**Commit message:** `chore(deps): bump Biome to 2.4 and Turbo to 2.9`

### Phase 4 — TanStack ecosystem

These versions move in lockstep; mixing major-version-skewed TanStack packages is the most common source of breakage. Upgrade all together.

| Package | From | To |
|---|---|---|
| `@tanstack/react-form` | 1.28.0 | 1.32.0 |
| `@tanstack/react-query` | 5.90.20 | 5.100.14 |
| `@tanstack/react-router` | 1.157.16 | 1.170.8 |
| `@tanstack/react-router-devtools` (dev) | 1.157.16 | 1.167.0 |
| `@tanstack/router-plugin` (dev) | 1.157.16 | 1.168.11 |

After install, regenerate the route tree by running `pnpm dev:desktop` once (router-plugin writes `routeTree.gen.ts`); commit that file if it changed.

**Commit message:** `chore(deps): bump TanStack form/query/router ecosystem`

### Phase 5 — Tauri JS bindings

| Package | From | To |
|---|---|---|
| `@tauri-apps/api` | 2.9.1 | 2.11.0 |
| `@tauri-apps/cli` (dev) | 2.9.6 | 2.11.2 |

`plugin-opener` was already moved in Phase 1. Run the Rust gate (cargo fmt/clippy/test) in addition to the JS gate, because the CLI bump can shift Tauri's generated bindings.

**Commit message:** `chore(deps): bump @tauri-apps/api and cli to 2.11`

### Phase 6 — Tailwind 4.1 → 4.3

| Package | From | To |
|---|---|---|
| `tailwindcss` (dev) | 4.1.18 | 4.3.0 |
| `@tailwindcss/vite` | 4.1.18 | 4.3.0 |

Bump together. After install, build the app once and skim a few pages — Tailwind minor releases occasionally adjust default theme values. No config changes expected.

**Commit message:** `chore(deps): bump Tailwind to 4.3`

### Phase 7 — Vite 6 → 8 + @vitejs/plugin-react 4 → 6 (MAJOR)

| Package | From | To |
|---|---|---|
| `vite` (dev) | 6.4.1 | 8.0.14 |
| `@vitejs/plugin-react` (dev) | 4.7.0 | 6.0.2 |

This skips Vite 7 entirely; review the Vite 7 *and* 8 migration notes before starting. Watch for:

- Node version floor — Vite 7+ requires Node ≥ 20.19 / 22.12. Confirm CI and `engines` match.
- Changes to the `defineConfig` signature or plugin order in `apps/desktop/vite.config.ts`.
- Any deprecated options surfacing as build warnings.

Run the full gate including the production `vite build` — this is the phase most likely to surface bundler regressions.

**Commit message:** `chore(deps): upgrade Vite to 8 and plugin-react to 6`

### Phase 8 — Vitest 3 → 4 + jsdom 26 → 29 (MAJOR)

| Package | From | To |
|---|---|---|
| `vitest` (dev) | 3.2.4 | 4.1.7 |
| `@vitest/coverage-v8` (dev) | 3.2.4 | 4.1.7 |
| `@vitest/ui` (dev) | 3.2.4 | 4.1.7 |
| `jsdom` (dev) | 26.1.0 | 29.1.1 |

Vitest 4 has changes around the `projects` config and coverage thresholds; `apps/desktop/vitest.config.ts` uses both `defineConfig` + `defineProject` with three projects — read the Vitest 4 migration guide before bumping and adjust the config in the same commit if needed. Coverage thresholds (80% unit, 60% integration per CLAUDE.md) must still pass after the bump.

**Commit message:** `chore(deps): upgrade Vitest to 4 and jsdom to 29`

### Phase 9 — TypeScript 5 → 6 + @types/node 22 → 25 (MAJOR)

| Package | From | To | Where |
|---|---|---|---|
| `typescript` (dev) | 5.9.3 | 6.0.3 | catalog |
| `@types/node` (dev) | 22.19.7 | 25.9.1 | `apps/desktop` |

TypeScript 6 will introduce new errors. Expect to fix a handful of inference / library typing issues in the same commit. `@types/node` 25 matches Node 25; if the local/CI Node runtime is on 22 LTS, prefer pinning `@types/node` to a 22.x latest instead (e.g. `^22.19.7` stays put). Confirm Node runtime first; bump `@types/node` to match the runtime, not blindly to latest.

Run `pnpm check-types` across the whole workspace (`turbo check-types` runs it for every package). Fix any new errors before moving on.

**Commit message:** `chore(deps): upgrade TypeScript to 6 and @types/node to match runtime`

### Phase 10 — lucide-react 0.473 → 1.x (effectively MAJOR)

| Package | From | To |
|---|---|---|
| `lucide-react` | 0.473.0 | 1.16.0 |

This is the 0.x → 1.0 stabilization. Some icons were renamed between 0.x releases. The codebase imports lucide-react from ~18 files (UI primitives in `src/components/ui/` and several module components under `src/modules/products/ui/`). After bumping:

1. Run `pnpm run check-types` — missing icon exports will surface as type errors.
2. For each missing icon, look it up in the lucide-react changelog/migration table and rename the import + JSX usage.
3. Smoke-test the dev server and visually confirm icons render in product list, product detail, and dialogs.

**Commit message:** `chore(deps): upgrade lucide-react to 1.x and rename moved icons`

### Phase 11 — Tooling majors (shadcn + lint-staged)

| Package | From | To |
|---|---|---|
| `shadcn` | 3.7.0 | 4.8.0 |
| `lint-staged` (dev) | 16.2.7 | 17.0.5 |

`shadcn` here is the CLI tool; the bump shouldn't change any already-generated UI components. Verify by running the CLI once (`pnpm dlx shadcn --version`) and confirming no existing component file changes are required by the generator.

`lint-staged` 17 may tighten its config schema — the root `package.json` `lint-staged` block is small (one glob) so adjustments are unlikely but possible. Run `pnpm install && git add . && git commit` on a no-op change to confirm Husky + lint-staged still wires through.

**Commit message:** `chore(deps): upgrade shadcn CLI to 4 and lint-staged to 17`

## Critical Files

- `pnpm-workspace.yaml` — catalog entries for `dotenv`, `zod`, `typescript` (Phases 1/2/9)
- `package.json` (root) — Biome, Turbo, lint-staged, husky (Phases 3, 11)
- `apps/desktop/package.json` — most app-level deps (every phase touches this)
- `packages/env/package.json` — `@t3-oss/env-core` (Phase 1)
- `apps/desktop/vite.config.ts` — review for Phase 7
- `apps/desktop/vitest.config.ts` — review for Phase 8
- `.vscode/settings.json` — update Biome LSP path in Phase 3
- `apps/desktop/src/routeTree.gen.ts` — regenerated in Phase 4
- Files importing `lucide-react` (~18 files under `src/components/ui/` and `src/modules/products/ui/`) — touched in Phase 10

## Rollback

Because each phase is one commit, rolling back any single phase is `git revert <sha>` on that commit. The phases were ordered so reverting any one of them does not require reverting later ones, with one exception: if Phase 7 (Vite) is reverted, Phase 8 (Vitest) is unaffected, but if Phase 8 is reverted while Phase 7 stays, double-check that Vitest 3 is still compatible with Vite 8 — if not, both must revert together.

## Out of Scope

- Rust crate updates (`Cargo.toml` dependencies) — covered by a separate plan.
- Node runtime / `engines` field changes — only revisit if Phase 7 or 9 forces it.
- `pnpm` itself (`packageManager` field) — currently pinned to `pnpm@10.17.0`; bump separately if desired.
