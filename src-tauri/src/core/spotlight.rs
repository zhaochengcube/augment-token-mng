use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

static CURRENT_SHORTCUT: Mutex<Option<String>> = Mutex::new(None);

const SPOTLIGHT_LABEL: &str = "spotlight";
const SPOTLIGHT_WIDTH: f64 = 680.0;
const SPOTLIGHT_HEIGHT: f64 = 480.0;

/// 创建或显示 Spotlight 搜索窗口
fn show_spotlight_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SPOTLIGHT_LABEL) {
        // 窗口已存在，显示并聚焦
        let _ = window.show();
        let _ = window.set_focus();
        // 通知前端重新聚焦搜索框
        let _ = app.emit_to(SPOTLIGHT_LABEL, "spotlight-focus", ());
        return Ok(());
    }

    // 创建新窗口
    let url = WebviewUrl::App("index.html?window=spotlight".into());

    let window = WebviewWindowBuilder::new(app, SPOTLIGHT_LABEL, url)
        .title("Quick Search")
        .inner_size(SPOTLIGHT_WIDTH, SPOTLIGHT_HEIGHT)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .center()
        .skip_taskbar(true)
        .visible(true)
        .focused(true)
        .build()
        .map_err(|e| format!("Failed to create spotlight window: {}", e))?;

    // 窗口失焦时自动隐藏
    let app_handle = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            if let Some(w) = app_handle.get_webview_window(SPOTLIGHT_LABEL) {
                let _ = w.hide();
            }
        }
    });

    Ok(())
}

/// 隐藏 Spotlight 搜索窗口
fn hide_spotlight_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SPOTLIGHT_LABEL) {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 切换 Spotlight 窗口显示状态
#[tauri::command]
pub fn toggle_spotlight(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SPOTLIGHT_LABEL) {
        if window.is_visible().unwrap_or(false) {
            hide_spotlight_window(&app)
        } else {
            show_spotlight_window(&app)
        }
    } else {
        show_spotlight_window(&app)
    }
}

/// 注册全局快捷键
#[tauri::command]
pub fn register_spotlight_shortcut(app: AppHandle, shortcut: String) -> Result<(), String> {
    let shortcut_parsed: Shortcut = shortcut
        .parse()
        .map_err(|e| format!("Invalid shortcut '{}': {}", shortcut, e))?;

    let gsm = app.global_shortcut();

    // 先取消旧的 spotlight 快捷键（不影响其他全局快捷键）
    if let Ok(mut current) = CURRENT_SHORTCUT.lock() {
        if let Some(old_str) = current.take() {
            if let Ok(old_shortcut) = old_str.parse::<Shortcut>() {
                let _ = gsm.unregister(old_shortcut);
            }
        }
    }

    let app_clone = app.clone();
    gsm.on_shortcut(shortcut_parsed, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            let _ = toggle_spotlight(app_clone.clone());
        }
    })
    .map_err(|e| format!("Failed to register shortcut: {}", e))?;

    // 记录当前注册的快捷键
    if let Ok(mut current) = CURRENT_SHORTCUT.lock() {
        *current = Some(shortcut);
    }

    Ok(())
}

/// 取消注册全局快捷键
#[tauri::command]
pub fn unregister_spotlight_shortcut(app: AppHandle) -> Result<(), String> {
    let gsm = app.global_shortcut();

    if let Ok(mut current) = CURRENT_SHORTCUT.lock() {
        if let Some(old_str) = current.take() {
            if let Ok(old_shortcut) = old_str.parse::<Shortcut>() {
                gsm.unregister(old_shortcut)
                    .map_err(|e| format!("Failed to unregister shortcut: {}", e))?;
            }
        }
    }

    Ok(())
}

/// 获取当前快捷键注册状态
#[tauri::command]
pub fn is_spotlight_shortcut_registered(app: AppHandle, shortcut: String) -> Result<bool, String> {
    let shortcut_parsed: Shortcut = shortcut
        .parse()
        .map_err(|e| format!("Invalid shortcut '{}': {}", shortcut, e))?;

    Ok(app.global_shortcut().is_registered(shortcut_parsed))
}
