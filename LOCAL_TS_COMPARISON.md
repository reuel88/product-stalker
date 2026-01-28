# Comparison: Your Project vs local.ts

## Summary

Your project (`apps/web`) is based on `local.ts` but has diverged with your own product tracking features. Here's what exists in `local.ts` that's missing or different in your project.

---

## Completed Items

- [x] **Testing Infrastructure** - Vitest with @testing-library/react, jsdom, and Tauri API mocks

---

## Missing from Your Project

### 1. Frontend Structure

| Feature | local.ts | Your Project |
|---------|----------|--------------|
| `assets/` directory | Dedicated static assets folder | Missing (uses `public/`) |
| `constants/` directory | Centralized app constants | Missing |
| `stores/` directory | State management stores | Missing (uses TanStack Query only) |
| `App.tsx` component | Separate App component file | Missing (inline in main.tsx) |

### 2. Backend/Tauri Plugins

| Plugin | local.ts | Your Project |
|--------|----------|--------------|
| `tauri-plugin-opener` | Has it (opens files/URLs natively) | Missing |
| Window close event handler | `on_close_requested()` for state save | Not implemented |

### 3. Backend Architecture

| Feature | local.ts | Your Project |
|---------|----------|--------------|
| ORM | Diesel (with r2d2 connection pool) | SeaORM |
| Database | SQLite (with Diesel) | SQLite (with SeaORM) |

---

## Features You Have That local.ts Doesn't

Your project has extended beyond the starter kit:

1. **Products CRUD** - Full product tracking entity with:
   - Products table with name, url, description, notes
   - ProductService, ProductRepository layers
   - Frontend products table with pagination

2. **Extended Settings** - More settings than local.ts:
   - `sidebar_expanded`
   - More detailed logging configuration

3. **Test Settings Page** - `/test-settings` route for debugging

4. **SeaORM** - More modern async ORM (vs Diesel's sync approach)

5. **Vitest Testing** - Modern test runner with React Testing Library

---

## Specific Missing Items to Consider Adding

### High Priority

1. **tauri-plugin-opener**
   - Useful for opening product URLs in native browser
   - Add to Cargo.toml: `tauri-plugin-opener = "2.5.2"`
   - Register in `lib.rs` and add to `capabilities/default.json`

### Medium Priority

2. **Constants Directory**
   - Create `src/constants/` for:
     - API endpoints
     - Default values
     - Magic strings/numbers

3. **State Stores**
   - local.ts has a `stores/` directory
   - Could add Zustand for UI state (separate from server state in TanStack Query)

4. **Window Close Handler**
   - Add `on_close_requested()` to properly save state before closing

### Low Priority

5. **Separate App.tsx**
   - Extract App component from `main.tsx` for cleaner structure

6. **Assets Directory**
   - Move static assets from `public/` to `src/assets/` for better bundling

---

## Commands Comparison

| Command | local.ts | Your Project |
|---------|----------|--------------|
| `get_app_settings` | Yes | `get_settings` |
| `update_app_settings` | Yes | `update_settings` |
| `set_tray_visible` | Yes | Handled in `update_settings` |
| `close_splashscreen` | Yes | Yes |
| `are_notifications_enabled` | Yes | Yes |
| Product commands | No | Yes (5 commands) |

---

## Next Steps

Recommended order for remaining items:

1. **Add tauri-plugin-opener** - Most useful for your product tracking app (open product URLs)
2. **Add window close handler** - Proper cleanup ensures settings/state are saved
3. **Create constants directory** - Better code organization as the app grows

Your project has already evolved past the starter kit with the products feature and SeaORM choice, which are valid architectural decisions.
