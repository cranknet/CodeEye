mod capture;
mod cleaner;
mod compression;
mod git;
mod logging;
pub mod mcp;
mod storage;
mod uninstall;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[tauri::command]
fn get_git_context(path: String) -> Result<git::GitContext, String> {
    let dir = std::path::Path::new(&path);
    let mut ctx = git::detect_git_context(dir)?;
    ctx.recent_diff = git::get_recent_diff(dir, 200);
    Ok(ctx)
}

#[tauri::command]
fn create_new_session(page_name: String, project: String) -> Result<String, String> {
    let base = storage::default_base_path();
    storage::create_session(&base, &page_name, &project)
}

#[tauri::command]
fn list_all_sessions() -> Result<Vec<storage::SessionSummary>, String> {
    let base = storage::default_base_path();
    storage::list_sessions(&base)
}

#[tauri::command]
fn delete_session_by_id(session_id: String) -> Result<(), String> {
    let base = storage::default_base_path();
    storage::delete_session(&base, &session_id)
}

#[tauri::command]
fn load_app_config() -> Result<storage::AppConfig, String> {
    let base = storage::default_base_path();
    storage::load_config(&base)
}

#[tauri::command]
fn save_app_config(config: storage::AppConfig) -> Result<(), String> {
    let base = storage::default_base_path();
    storage::save_config(&base, &config)
}

#[tauri::command]
fn load_session_meta(session_id: String) -> Result<serde_json::Value, String> {
    let base = storage::default_base_path();
    storage::load_session_meta(&base, &session_id)
}

#[tauri::command]
fn save_session_meta(session_id: String, meta: serde_json::Value) -> Result<(), String> {
    let base = storage::default_base_path();
    storage::save_session_meta(&base, &session_id, &meta)
}

#[tauri::command]
fn list_monitors() -> Result<Vec<capture::MonitorInfo>, String> {
    capture::list_monitors()
}

#[tauri::command]
fn capture_screen(monitor_id: u32) -> Result<String, String> {
    let png_bytes = capture::capture_monitor(monitor_id)?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &png_bytes,
    ))
}

#[tauri::command]
fn save_session_capture(session_id: String, image_base64: String) -> Result<(), String> {
    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD
        .decode(&image_base64)
        .map_err(|e| format!("Failed to decode image: {e}"))?;
    let base = storage::default_base_path();
    let session_dir = base.join("sessions").join(&session_id);
    if !session_dir.exists() {
        return Err(format!("Session not found: {session_id}"));
    }
    let config = storage::load_config(&base).unwrap_or_default();
    compression::save_capture(
        &session_dir,
        &data,
        config.compression_max_resolution,
        config.compression_quality,
    )?;
    Ok(())
}

#[tauri::command]
fn capture_region(
    monitor_id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<String, String> {
    let png_bytes = capture::capture_region(monitor_id, x, y, width, height)?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &png_bytes,
    ))
}

#[tauri::command]
fn check_capture_permission() -> bool {
    capture::check_capture_permission()
}

#[tauri::command]
fn scan_integrations() -> Vec<mcp::adapters::AdapterStatus> {
    let binary = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "codeeye".into());
    mcp::adapters::scan_all(&binary)
}

#[tauri::command]
fn connect_integration(tool_name: String) -> Result<String, String> {
    let binary = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "codeeye".into());
    let adapters = mcp::adapters::all_adapters();
    let adapter = adapters
        .iter()
        .find(|a| a.name() == tool_name)
        .ok_or_else(|| format!("Unknown tool: {tool_name}"))?;
    adapter.connect(&binary)?;
    Ok(format!("{tool_name} connected"))
}

#[tauri::command]
fn disconnect_integration(tool_name: String) -> Result<String, String> {
    let adapters = mcp::adapters::all_adapters();
    let adapter = adapters
        .iter()
        .find(|a| a.name() == tool_name)
        .ok_or_else(|| format!("Unknown tool: {tool_name}"))?;
    adapter.disconnect()?;
    Ok(format!("{tool_name} disconnected"))
}

#[tauri::command]
fn run_uninstall(backup: bool) -> Result<String, String> {
    uninstall::run_uninstall(backup)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus existing window when second instance launched
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(logging::build_plugin().build())
        .invoke_handler(tauri::generate_handler![
            get_git_context,
            create_new_session,
            list_all_sessions,
            delete_session_by_id,
            load_app_config,
            save_app_config,
            load_session_meta,
            save_session_meta,
            list_monitors,
            capture_screen,
            capture_region,
            save_session_capture,
            check_capture_permission,
            scan_integrations,
            connect_integration,
            disconnect_integration,
            run_uninstall,
        ])
        .setup(|app| {
            // ── Tray menu ───────────────────────────────────────────────
            let shortcut_label = if cfg!(target_os = "macos") {
                "Cmd+Shift+E"
            } else {
                "Ctrl+Shift+E"
            };
            let capture_item = MenuItem::with_id(
                app,
                "capture",
                &format!("New Capture  {shortcut_label}"),
                true,
                None::<&str>,
            )?;
            let open_item =
                MenuItem::with_id(app, "open", "Open CodeEye", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&capture_item, &open_item, &separator, &quit_item])?;

            let tray_icon = app
                .default_window_icon()
                .cloned()
                .expect("default window icon must be set in tauri.conf.json");

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .tooltip("CodeEye — click to capture")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "capture" => {
                        let _ = app.emit("tray-capture", ());
                    }
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── Global shortcut (Ctrl+Shift+E / Cmd+Shift+E) ────────────
            let modifiers = if cfg!(target_os = "macos") {
                Modifiers::META | Modifiers::SHIFT
            } else {
                Modifiers::CONTROL | Modifiers::SHIFT
            };
            let capture_shortcut = Shortcut::new(Some(modifiers), Code::KeyE);
            match app.global_shortcut().on_shortcut(capture_shortcut, |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    let _ = app.emit("capture-hotkey", ());
                }
            }) {
                Ok(_) => log::info!("Global shortcut registered: Ctrl+Shift+E"),
                Err(e) => {
                    log::warn!("Failed to register global shortcut: {e}");
                    let _ = app.emit("hotkey-conflict", e.to_string());
                }
            }

            // Initialize storage
            let base = storage::default_base_path();
            storage::init_storage(&base).map_err(|e| {
                log::error!("Failed to initialize storage: {}", e);
                e
            })?;
            log::info!("Storage initialized at {:?}", base);

            // Scale UI for standard-DPI displays (e.g. 1080p on Linux)
            if let Some(webview) = app.get_webview_window("main") {
                let _ = webview.set_zoom(1.2);
            }

            // Run auto-cleaner
            let config = storage::load_config(&base).unwrap_or_default();
            match cleaner::run_cleanup(
                &base,
                config.session_limit as usize,
                config.session_retention_days,
            ) {
                Ok(n) if n > 0 => log::info!("Auto-cleaner removed {} sessions", n),
                Ok(_) => {}
                Err(e) => log::warn!("Auto-cleaner failed: {}", e),
            }

            log::info!("CodeEye starting up");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
