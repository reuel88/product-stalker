mod commands;
mod db;
mod entities;
mod error;
mod migrations;
mod plugins;
mod repositories;
mod services;

use std::sync::Mutex;

use db::DbState;
use tauri::tray::TrayIcon;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

/// State wrapper for the system tray icon, allowing runtime show/hide
pub struct TrayState(pub Mutex<Option<TrayIcon<tauri::Wry>>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            // Initialize logging plugin
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let handle = app.handle().clone();

            // Initialize database
            let conn = tauri::async_runtime::block_on(async move { db::init_db(&handle).await })?;

            app.manage(DbState::new(conn.clone()));

            // Load settings and apply them
            let settings = tauri::async_runtime::block_on(async {
                services::SettingService::get(&conn).await
            })?;

            log::info!(
                "Settings loaded: theme={}, show_in_tray={}, launch_at_login={}",
                settings.theme,
                settings.show_in_tray,
                settings.launch_at_login
            );

            // Initialize system tray (always create it, visibility based on setting)
            match plugins::system_tray::init(app.handle(), settings.show_in_tray) {
                Ok(tray) => {
                    app.manage(TrayState(Mutex::new(Some(tray))));
                }
                Err(e) => {
                    log::error!("Failed to initialize system tray: {}", e);
                    // Still manage an empty TrayState so commands don't panic
                    app.manage(TrayState(Mutex::new(None)));
                }
            }

            // Configure autostart based on settings
            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::ManagerExt;
                let autostart_manager = app.autolaunch();
                let is_enabled = autostart_manager.is_enabled().unwrap_or(false);

                if settings.launch_at_login && !is_enabled {
                    if let Err(e) = autostart_manager.enable() {
                        log::error!("Failed to enable autostart: {}", e);
                    } else {
                        log::info!("Autostart enabled");
                    }
                } else if !settings.launch_at_login && is_enabled {
                    if let Err(e) = autostart_manager.disable() {
                        log::error!("Failed to disable autostart: {}", e);
                    } else {
                        log::info!("Autostart disabled");
                    }
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Product commands
            commands::get_products,
            commands::get_product,
            commands::create_product,
            commands::update_product,
            commands::delete_product,
            // Settings commands
            commands::get_settings,
            commands::update_settings,
            // Notification commands
            commands::are_notifications_enabled,
            commands::send_notification,
            // Window commands
            commands::close_splashscreen,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
