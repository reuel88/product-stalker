# Implementation Plan: Settings, System Tray, Notifications, Theming, Logging, Window State, Autostart, Splash Screen

**Status:** Planning complete, ready for implementation approval

Inspired by [local.ts](https://github.com/zap-studio/local.ts), adapted for SeaORM + SQLite.

---

## Session Summary (Resume Point)

### What We Did
1. Explored the local.ts GitHub repo for inspiration (structure, patterns, plugins)
2. Analyzed your current Tauri codebase (entities, repos, services, commands pattern)
3. Designed a comprehensive implementation plan using rust-seaorm-sqlite-expert
4. Got your preferences on:
   - **State Management:** TanStack Query (add @tanstack/react-query)
   - **Splash Screen:** Simple gradient design (purple with spinner)
   - **Scope:** All 8 features

### Your Current Codebase Structure
```
apps/web/src-tauri/src/
├── lib.rs              # Entry point with logging + DB init
├── error.rs            # AppError (Database, NotFound, Validation)
├── commands/products.rs # CRUD commands with DTOs
├── db/connection.rs    # SQLite init with WAL mode
├── entities/product.rs # Product entity
├── migrations/         # SeaORM migrations
├── repositories/       # Data access layer
└── services/           # Business logic layer
```

### local.ts Patterns We're Adapting
- System tray with Show/Hide/Quit menu + left-click toggle
- Multi-target logging (stdout, webview, file)
- Single-row settings pattern (id=1)
- Splash screen → main window transition
- Window state persistence via tauri-plugin-window-state
- Autostart via tauri-plugin-autostart

### To Resume
When you come back, just say: **"Continue with the implementation plan"** and I'll proceed with the approved plan below.

## Overview

Add 8 features to the Tauri app following existing layered architecture (Commands → Services → Repositories → Entities).

---

## 1. Dependencies to Add

**File:** `apps/web/src-tauri/Cargo.toml`

```toml
tauri = { version = "2.9.5", features = ["tray-icon"] }  # Add tray-icon feature
tauri-plugin-notification = "2"
tauri-plugin-window-state = "2"
tauri-plugin-autostart = "2"
```

---

## 2. New Files to Create

### Settings Entity
**`src/entities/setting.rs`**
- Single-row settings (id=1) with fields: theme, show_in_tray, launch_at_login, enable_logging, log_level, enable_notifications, sidebar_expanded, updated_at
- Implements `Default` for auto-initialization

### Settings Repository
**`src/repositories/setting_repository.rs`**
- `get_or_create()` - returns settings or creates default
- `update()` - partial update with timestamp

### Settings Service
**`src/services/setting_service.rs`**
- `get()` and `update()` with validation for theme/log_level values

### Settings Commands
**`src/commands/settings.rs`**
- `get_settings` - returns SettingsResponse DTO
- `update_settings` - accepts UpdateSettingsInput DTO

### Settings Migration
**`src/migrations/m20240102_000001_create_settings_table.rs`**
- Creates settings table with proper defaults

### Notification Commands
**`src/commands/notifications.rs`**
- `are_notifications_enabled` - checks settings
- `send_notification` - respects enable_notifications

### Plugins Module
**`src/plugins/mod.rs`**
```rust
pub mod system_tray;
pub mod window_state;
```

### System Tray Plugin
**`src/plugins/system_tray.rs`**
- `init()` - creates tray with Show/Hide/Quit menu
- Left-click toggles window visibility
- `apply_settings_visibility()` - respects show_in_tray setting

### Window State Plugin
**`src/plugins/window_state.rs`**
- `init()` - restores window state
- `on_close_requested()` - saves state

### Splash Screen
**`apps/web/splash.html`**
- Centered loading screen with spinner
- App name and "Initializing..." text

---

## 3. Files to Modify

### `src/entities/mod.rs`
Add: `pub mod setting;`

### `src/entities/prelude.rs`
Add: `pub use super::setting::{ActiveModel as SettingActiveModel, Entity as Setting, Model as SettingModel};`

### `src/repositories/mod.rs`
Add: `pub mod setting_repository; pub use setting_repository::SettingRepository;`

### `src/services/mod.rs`
Add: `pub mod setting_service; pub use setting_service::SettingService;`

### `src/commands/mod.rs`
Add: `pub mod notifications; pub mod settings;` and re-exports

### `src/migrations/mod.rs`
Add migration module

### `src/migrations/migrator.rs`
Add settings migration to vec

### `src/lib.rs` (Major Changes)
```rust
mod plugins;  // Add this module

// In run():
.plugin(tauri_plugin_notification::init())
.plugin(tauri_plugin_window_state::Builder::default().build())
.plugin(tauri_plugin_autostart::init(
    tauri_plugin_autostart::MacosLauncher::LaunchAgent,
    Some(vec![]),
))
.setup(|app| {
    // Enhanced logging with multiple targets
    // Init database
    // Load settings
    // Init system tray
    // Apply visibility settings
    // Restore window state
    // Configure autostart based on settings
    // Close splash, show main window
})
.on_window_event(|window, event| {
    // Save window state on close
})
.invoke_handler(tauri::generate_handler![
    // Existing product commands...
    commands::get_settings,
    commands::update_settings,
    commands::are_notifications_enabled,
    commands::send_notification,
])
```

### `apps/web/src-tauri/tauri.conf.json`
```json
{
  "app": {
    "windows": [
      {
        "label": "splash",
        "url": "splash.html",
        "width": 400,
        "height": 300,
        "decorations": false,
        "center": true,
        "visible": true
      },
      {
        "label": "main",
        "width": 1200,
        "height": 800,
        "visible": false,
        "center": true
      }
    ],
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  }
}
```

---

## 4. Frontend Changes (Minimal)

### Add TanStack Query Dependency
```bash
pnpm add @tanstack/react-query
```

### `apps/web/src/hooks/useSettings.ts`
- React hook using TanStack Query
- `useSettings()` returns settings + updateSettings mutation

### `apps/web/src/main.tsx` (or App wrapper)
- Wrap app with `<QueryClientProvider>`

### Theme Sync in Root Component
```typescript
// Sync backend theme to next-themes on app load
useEffect(() => {
  if (settings?.theme) setTheme(settings.theme);
}, [settings?.theme]);
```

---

## 5. Initialization Flow

1. App starts → Splash visible, main hidden
2. Logging plugin initialized
3. Database initialized + migrations run
4. Settings loaded (created with defaults if first run)
5. System tray initialized
6. Window state restored
7. Autostart configured based on settings
8. Splash closed, main window shown
9. Frontend syncs theme from backend

---

## 6. Verification Steps

### Build Test
```bash
cd apps/web/src-tauri
cargo build
```

### Runtime Tests
- [ ] App starts with splash screen
- [ ] Splash closes, main window appears
- [ ] System tray icon visible with menu (Show/Hide/Quit)
- [ ] Tray left-click toggles window
- [ ] Window size/position persists after restart
- [ ] Settings persist in database
- [ ] Theme changes sync frontend ↔ backend
- [ ] Notifications respect enable_notifications setting
- [ ] Autostart setting toggles OS startup behavior

### Database Verification
```sql
SELECT * FROM settings;  -- Should return 1 row with id=1
```

### Frontend Verification
```typescript
await window.__TAURI__.invoke('get_settings');
await window.__TAURI__.invoke('update_settings', { input: { theme: 'dark' } });
```

---

## 7. Implementation Order

1. **Settings Foundation** - entity, repo, service, commands, migration
2. **System Tray** - plugin module, tray init
3. **Notifications** - commands, integration
4. **Window State** - plugin, save/restore
5. **Autostart** - plugin configuration
6. **Splash Screen** - HTML, tauri.conf.json, close logic
7. **Frontend** - useSettings hook, theme sync
8. **Testing** - verify all features work together

---

## 8. Detailed Code Reference

### Settings Entity (`src/entities/setting.rs`)
```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "settings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub theme: String,
    pub show_in_tray: bool,
    pub launch_at_login: bool,
    pub enable_logging: bool,
    pub log_level: String,
    pub enable_notifications: bool,
    pub sidebar_expanded: bool,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Default for Model {
    fn default() -> Self {
        Self {
            id: 1,
            theme: "system".to_string(),
            show_in_tray: true,
            launch_at_login: false,
            enable_logging: true,
            log_level: "info".to_string(),
            enable_notifications: true,
            sidebar_expanded: true,
            updated_at: chrono::Utc::now(),
        }
    }
}
```

### Settings Migration (`src/migrations/m20240102_000001_create_settings_table.rs`)
```rust
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(integer(Settings::Id).primary_key())
                    .col(string(Settings::Theme).default("system"))
                    .col(boolean(Settings::ShowInTray).default(true))
                    .col(boolean(Settings::LaunchAtLogin).default(false))
                    .col(boolean(Settings::EnableLogging).default(true))
                    .col(string(Settings::LogLevel).default("info"))
                    .col(boolean(Settings::EnableNotifications).default(true))
                    .col(boolean(Settings::SidebarExpanded).default(true))
                    .col(timestamp_with_time_zone(Settings::UpdatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Settings {
    Table, Id, Theme, ShowInTray, LaunchAtLogin, EnableLogging,
    LogLevel, EnableNotifications, SidebarExpanded, UpdatedAt,
}
```

### System Tray Plugin (`src/plugins/system_tray.rs`)
```rust
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use crate::error::AppError;

pub fn init<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| AppError::Database(sea_orm::DbErr::Custom(e.to_string())))?;
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)
        .map_err(|e| AppError::Database(sea_orm::DbErr::Custom(e.to_string())))?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)
        .map_err(|e| AppError::Database(sea_orm::DbErr::Custom(e.to_string())))?;

    let menu = Menu::with_items(app, &[&show, &hide, &quit])
        .map_err(|e| AppError::Database(sea_orm::DbErr::Custom(e.to_string())))?;

    TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "quit" => app.exit(0),
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "hide" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                if let Some(w) = tray.app_handle().get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }
        })
        .build(app)
        .map_err(|e| AppError::Database(sea_orm::DbErr::Custom(e.to_string())))?;

    log::info!("System tray initialized");
    Ok(())
}
```

### Splash Screen (`apps/web/splash.html`)
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Loading...</title>
  <style>
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      display: flex; align-items: center; justify-content: center;
      height: 100vh;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      color: white;
    }
    .container { text-align: center; }
    .logo { font-size: 48px; font-weight: 700; margin-bottom: 16px; }
    .text { font-size: 16px; opacity: 0.9; margin-bottom: 24px; }
    .spinner {
      width: 40px; height: 40px; margin: 0 auto;
      border: 3px solid rgba(255,255,255,0.3);
      border-top-color: white; border-radius: 50%;
      animation: spin 1s linear infinite;
    }
    @keyframes spin { to { transform: rotate(360deg); } }
  </style>
</head>
<body>
  <div class="container">
    <div class="logo">Product Stalker</div>
    <div class="text">Initializing...</div>
    <div class="spinner"></div>
  </div>
</body>
</html>
```

### Frontend Hook (`apps/web/src/hooks/useSettings.ts`)
```typescript
import { invoke } from '@tauri-apps/api/core';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

export interface Settings {
  theme: 'light' | 'dark' | 'system';
  show_in_tray: boolean;
  launch_at_login: boolean;
  enable_logging: boolean;
  log_level: string;
  enable_notifications: boolean;
  sidebar_expanded: boolean;
  updated_at: string;
}

export function useSettings() {
  const queryClient = useQueryClient();

  const { data: settings, isLoading } = useQuery({
    queryKey: ['settings'],
    queryFn: () => invoke<Settings>('get_settings'),
  });

  const updateSettings = useMutation({
    mutationFn: (input: Partial<Settings>) =>
      invoke<Settings>('update_settings', { input }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['settings'] }),
  });

  return { settings, isLoading, updateSettings: updateSettings.mutate };
}
```

---

## 9. tauri.conf.json Changes

```json
{
  "app": {
    "windows": [
      {
        "label": "splash",
        "url": "splash.html",
        "width": 400,
        "height": 300,
        "decorations": false,
        "center": true,
        "visible": true,
        "skipTaskbar": true
      },
      {
        "label": "main",
        "title": "Product Stalker",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "center": true,
        "visible": false
      }
    ],
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  }
}
```
