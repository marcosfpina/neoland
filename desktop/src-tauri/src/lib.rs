//! Neoland Desktop — Tauri-native AI console.
//!
//! This crate provides the native desktop shell that wraps the Leptos WASM
//! Web Console.  It adds system-tray integration, native OS notifications,
//! and (in future releases) an embedded Neoland server for full offline mode.

mod tray;

use tauri::Manager;

/// Tauri command: returns the Neoland server base URL.
///
/// In dev mode the Leptos frontend proxies through Trunk (localhost:8080).
/// In production the desktop app connects to the local Neoland server on
/// the default REST port (3001), or a user-configured endpoint.
#[tauri::command]
fn get_server_url() -> String {
    std::env::var("NEOLAND_SERVER_URL").unwrap_or_else(|_| "http://127.0.0.1:3001".into())
}

/// Tauri command: show a native OS notification.
#[tauri::command]
async fn notify(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| e.to_string())
}

/// Tauri command: check whether the Neoland server is reachable.
#[tauri::command]
async fn check_server_health(url: String) -> Result<String, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/health", url))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| format!("server unreachable: {e}"))?;

    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();

    Ok(format!("{{\"status\":{status},\"body\":{body:?}}}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Build the system tray
            let _tray = tray::create_tray(app.handle())?;

            // Log startup
            let window = app.get_webview_window("main").expect("main window not found");
            window
                .eval("console.log('[Neoland Desktop] Native shell ready')")
                .ok();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_server_url,
            notify,
            check_server_health,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Neoland desktop");
}
