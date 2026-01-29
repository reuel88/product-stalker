use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

use crate::error::AppError;

const TRAY_ICON: &[u8] = include_bytes!("../../icons/icon.png");

/// Initialize the system tray with Show/Hide/Quit menu
/// Returns the TrayIcon so it can be stored in app state for later manipulation
pub fn init<R: Runtime>(app: &AppHandle<R>, visible: bool) -> Result<TrayIcon<R>, AppError> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let menu = Menu::with_items(app, &[&show, &hide, &quit])
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let icon = Image::from_bytes(TRAY_ICON).map_err(|e| AppError::Validation(format!("{e}")))?;

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                log::info!("Quit requested from tray menu");
                app.exit(0);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                    log::debug!("Window shown from tray menu");
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                    log::debug!("Window hidden from tray menu");
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // Left-click toggles window visibility
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                        log::debug!("Window hidden via tray click");
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                        log::debug!("Window shown via tray click");
                    }
                }
            }
        })
        .build(app)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Set initial visibility after building
    tray.set_visible(visible)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    log::info!("System tray initialized (visible: {})", visible);
    Ok(tray)
}
