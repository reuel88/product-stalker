# Comparison: Your Project vs local.ts

## Summary

Your project (`apps/web`) is based on `local.ts` but has diverged with your own product tracking features. Here's what exists in `local.ts` that's missing or different in your project.

---

## Completed Items

- [x] **Testing Infrastructure** - Vitest with @testing-library/react, jsdom, and Tauri API mocks
- [x] **tauri-plugin-opener** - Opens URLs/files in native browser/app
- [x] **Window Close Handler** - `on_window_event()` for cleanup/logging on app close
- [x] **Constants Directory** - Centralized constants in `src/constants/` (api, queryKeys, messages, ui, config)
- [x] **Separate App.tsx** - Extracted App component from main.tsx for cleaner separation of concerns

---

## Missing from Your Project

### 1. Frontend Structure

| Feature | local.ts | Your Project |
|---------|----------|--------------|
| `assets/` directory | Dedicated static assets folder | Missing (uses `public/`) |
| `constants/` directory | Centralized app constants | ✅ Implemented |
| `stores/` directory | State management stores | Missing (uses TanStack Query only) |
| `App.tsx` component | Separate App component file | ✅ Implemented |

### 2. Backend Architecture

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

6. **tauri-plugin-opener** - Open product URLs in native browser

---

## Using tauri-plugin-opener

The opener plugin is now available. Use it in your frontend:

```typescript
import { openUrl, openPath, revealItemInDir } from '@tauri-apps/plugin-opener';

// Open a URL in the default browser
await openUrl('https://example.com');

// Open a file with default app
await openPath('/path/to/file.pdf');

// Reveal a file in the system file explorer
await revealItemInDir('/path/to/file');
```

---

## Specific Missing Items to Consider Adding

### Low Priority

1. **Assets Directory**
   - Move static assets from `public/` to `src/assets/` for better bundling

### Deferred - Not Needed Yet

3. **State Stores (Zustand)**
   - local.ts has a `stores/` directory for UI state management
   - **Current status**: Not needed - only 6 useState calls across the app, no prop drilling, server state handled cleanly by React Query, theme state by next-themes
   - **Revisit when**:
     - Sidebar is implemented and needs cross-component state
     - Global dialog/modal system is added
     - Command palette (Cmd+K) is implemented
     - Table state persistence across navigation is needed

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

1. **Assets directory** - Better bundling for static assets

Your project has already evolved past the starter kit with the products feature and SeaORM choice, which are valid architectural decisions.
