mod commands;
mod db;
mod entities;
mod error;
mod migrations;
mod repositories;
mod services;

use db::DbState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let handle = app.handle().clone();
            let conn = tauri::async_runtime::block_on(async move {
                db::init_db(&handle).await
            })?;

            app.manage(DbState::new(conn));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_products,
            commands::get_product,
            commands::create_product,
            commands::update_product,
            commands::delete_product,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
