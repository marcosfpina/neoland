//! System tray integration for Neoland Desktop.
//!
//! Provides a tray icon with context menu: Show/Hide window and Quit.

use tauri::{
    AppHandle,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

pub fn create_tray(app: &AppHandle) -> Result<tauri::tray::TrayIcon, tauri::Error> {
    let show_item = MenuItemBuilder::with_id("show", "Show Neoland").build(app)?;
    let hide_item = MenuItemBuilder::with_id("hide", "Hide Neoland").build(app)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit Neoland").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .item(&hide_item)
        .item(&separator)
        .item(&quit_item)
        .build()?;

    let tray = TrayIconBuilder::with_id("neoland-tray")
        .menu(&menu)
        .tooltip("Neoland AI Console")
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            },
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            },
            "quit" => {
                app.exit(0);
            },
            _ => {},
        })
        .on_tray_icon_event(|tray, event| {
            // Toggle window visibility on tray icon click (left mouse button)
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(tray)
}
