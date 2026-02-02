use tauri::{
    AppHandle, Manager, Emitter,
    tray::{TrayIcon, TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    menu::{Menu, MenuItem},
};
use std::sync::Mutex;

/// The fixed tray icon ID
const TRAY_ID: &str = "atm-tray";

/// Tray icon state manager - stores the TrayIcon to control its lifecycle
pub struct TrayState {
    pub tray_icon: Mutex<Option<TrayIcon>>,
}

impl TrayState {
    pub fn new() -> Self {
        Self {
            tray_icon: Mutex::new(None),
        }
    }
}

impl Default for TrayState {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create the tray menu
fn create_tray_menu(app: &AppHandle) -> Result<Menu<tauri::Wry>, String> {
    let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let platforms_i = MenuItem::with_id(app, "platforms", "平台选择", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let subscriptions_i = MenuItem::with_id(app, "subscriptions", "订阅管理", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    Menu::with_items(app, &[&show_i, &platforms_i, &subscriptions_i, &quit_i])
        .map_err(|e| e.to_string())
}

/// Create the system tray with menu
#[tauri::command]
pub fn create_tray(app: AppHandle) -> Result<bool, String> {
    let state = app.state::<TrayState>();
    let mut tray_guard = state.tray_icon.lock().map_err(|e| e.to_string())?;

    // If tray already exists in our state, re-enable it
    if let Some(ref tray) = *tray_guard {
        // Re-enable the existing tray with icon and menu
        let menu = create_tray_menu(&app)?;
        let _ = tray.set_menu(Some(menu));
        let _ = tray.set_icon(Some(app.default_window_icon().unwrap().clone()));
        let _ = tray.set_visible(true);
        return Ok(true);
    }

    // Check if tray exists by ID (in case of stale state) and re-enable it
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let menu = create_tray_menu(&app)?;
        let _ = tray.set_menu(Some(menu));
        let _ = tray.set_icon(Some(app.default_window_icon().unwrap().clone()));
        let _ = tray.set_visible(true);
        *tray_guard = Some(tray);
        return Ok(true);
    }

    // Create new tray - create menu
    let menu = create_tray_menu(&app)?;

    // Build tray icon with fixed ID
    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("ATM - Account Token Manager")
        .on_tray_icon_event(|tray, event| {
            // Handle left click - show window
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
                    let _ = window.unminimize();
                }
            }
        })
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.unminimize();
                    }
                }
                "platforms" => {
                    // Show window and emit event for frontend to navigate
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.unminimize();
                    }
                    let _ = app.emit("tray-menu-clicked", serde_json::json!({
                        "action": "platforms"
                    }));
                }
                "subscriptions" => {
                    // Show window and emit event for frontend to navigate
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                        let _ = window.unminimize();
                    }
                    let _ = app.emit("tray-menu-clicked", serde_json::json!({
                        "action": "subscriptions"
                    }));
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(&app)
        .map_err(|e| e.to_string())?;

    // Store the tray icon in state to control its lifecycle
    *tray_guard = Some(tray);
    Ok(true)
}

/// Destroy the system tray
#[tauri::command]
pub fn destroy_tray(app: AppHandle) -> Result<bool, String> {
    let state = app.state::<TrayState>();
    let mut tray_guard = state.tray_icon.lock().map_err(|e| e.to_string())?;

    // Take the tray out of state
    if let Some(tray) = tray_guard.take() {
        // First hide the tray icon
        let _ = tray.set_visible(false);
        // Remove the menu to clean up
        let _ = tray.set_menu::<tauri::menu::Menu<tauri::Wry>>(None);
        // Remove the icon
        let _ = tray.set_icon(None);
        // Then drop it
        drop(tray);
        Ok(true)
    } else {
        Ok(false) // Tray didn't exist
    }
}

/// Toggle the system tray on or off
#[tauri::command]
pub fn toggle_tray(app: AppHandle, enabled: bool) -> Result<bool, String> {
    if enabled {
        create_tray(app)
    } else {
        destroy_tray(app)
    }
}

/// Get the current tray status
#[tauri::command]
pub fn get_tray_status(app: AppHandle) -> Result<bool, String> {
    let state = app.state::<TrayState>();
    let tray_guard = state.tray_icon.lock().map_err(|e| e.to_string())?;
    Ok(tray_guard.is_some())
}
