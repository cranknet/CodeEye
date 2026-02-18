mod capture;
mod cleaner;
mod compression;
mod git;
mod logging;
mod storage;

use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

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
            list_monitors,
            capture_screen,
            capture_region,
            check_capture_permission,
        ])
        .setup(|app| {
            // Tray icon setup
            let _tray = TrayIconBuilder::new()
                .tooltip("CodeEye")
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

            // Initialize storage
            let base = storage::default_base_path();
            storage::init_storage(&base).map_err(|e| {
                log::error!("Failed to initialize storage: {}", e);
                e
            })?;
            log::info!("Storage initialized at {:?}", base);

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
