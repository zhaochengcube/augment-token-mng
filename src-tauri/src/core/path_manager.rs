//! 公用路径管理模块
//!
//! 提供跨平台的自定义路径管理功能，包括：
//! - 获取/设置自定义路径
//! - 路径验证
//! - 文件选择对话框

use std::path::PathBuf;
use tauri::AppHandle;

/// 平台配置，用于定义不同平台的特定行为
pub struct PlatformPathConfig {
    /// 配置文件名（如 "windsurf_config.json"）
    pub config_filename: &'static str,
    /// 应用名称（用于文件对话框标题）
    pub app_name: &'static str,
    /// Windows 可执行文件名（如 "Windsurf.exe"）
    pub windows_exe_name: &'static str,
    /// 验证路径时检查的关键字（如 "windsurf"）
    pub path_keyword: &'static str,
}

/// Windsurf 平台配置
pub const WINDSURF_CONFIG: PlatformPathConfig = PlatformPathConfig {
    config_filename: "windsurf_config.json",
    app_name: "Windsurf",
    windows_exe_name: "Windsurf.exe",
    path_keyword: "windsurf",
};

/// Antigravity 平台配置
pub const ANTIGRAVITY_CONFIG: PlatformPathConfig = PlatformPathConfig {
    config_filename: "antigravity_config.json",
    app_name: "Antigravity",
    windows_exe_name: "Antigravity.exe",
    path_keyword: "antigravity",
};

/// Cursor 平台配置
pub const CURSOR_CONFIG: PlatformPathConfig = PlatformPathConfig {
    config_filename: "cursor_config.json",
    app_name: "Cursor",
    windows_exe_name: "Cursor.exe",
    path_keyword: "cursor",
};

/// 获取配置文件路径
pub fn get_config_path(app: &AppHandle, config: &PlatformPathConfig) -> Result<PathBuf, String> {
    use tauri::Manager;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    Ok(app_data_dir.join(config.config_filename))
}

/// 从配置文件中获取自定义路径
pub fn get_custom_path(
    app: &AppHandle,
    config: &PlatformPathConfig,
) -> Result<Option<String>, String> {
    let config_path = get_config_path(app, config)?;

    if !config_path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

    Ok(json
        .get("custom_path")
        .and_then(|v| v.as_str())
        .map(String::from))
}

/// 设置自定义路径到配置文件
///
/// # 参数
/// - `app`: Tauri 应用句柄
/// - `config`: 平台配置
/// - `path`: 可选的自定义路径，None 表示删除自定义路径
/// - `validate_fn`: 路径验证函数
pub fn set_custom_path<F>(
    app: &AppHandle,
    config: &PlatformPathConfig,
    path: Option<String>,
    validate_fn: F,
) -> Result<(), String>
where
    F: Fn(&str) -> Result<bool, String>,
{
    let config_path = get_config_path(app, config)?;

    // 如果提供了路径，验证它
    if let Some(ref p) = path {
        if !validate_fn(p)? {
            return Err(format!("Invalid {} executable path", config.app_name));
        }
    }

    // 读取现有配置或创建新配置
    let mut json: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // 更新或删除 custom_path
    if let Some(p) = path {
        json["custom_path"] = serde_json::Value::String(p);
    } else {
        json.as_object_mut().map(|obj| obj.remove("custom_path"));
    }

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    // 写入配置
    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&config_path, content).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

/// 验证可执行文件路径
pub fn validate_executable_path(path: &str, config: &PlatformPathConfig) -> Result<bool, String> {
    let path = PathBuf::from(path);

    if !path.exists() {
        return Ok(false);
    }

    #[cfg(target_os = "windows")]
    {
        let is_exe = path
            .extension()
            .map(|ext| ext.eq_ignore_ascii_case("exe"))
            .unwrap_or(false);
        let name_matches = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_lowercase().contains(config.path_keyword))
            .unwrap_or(false);
        Ok(is_exe && name_matches)
    }

    #[cfg(target_os = "macos")]
    {
        let is_app = path.extension().map(|ext| ext == "app").unwrap_or(false);
        let name_matches = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_lowercase().contains(config.path_keyword))
            .unwrap_or(false);
        Ok((is_app || path.is_file()) && name_matches)
    }

    #[cfg(target_os = "linux")]
    {
        let is_executable = path.is_file();
        let name_matches = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_lowercase().contains(config.path_keyword))
            .unwrap_or(false);
        Ok(is_executable && name_matches)
    }
}

/// 打开文件选择对话框
pub fn select_executable_path(config: &PlatformPathConfig) -> Result<Option<String>, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        let script = format!(
            r#"
            Add-Type -AssemblyName System.Windows.Forms
            $dialog = New-Object System.Windows.Forms.OpenFileDialog
            $dialog.Filter = "{}|{}.exe|All executables|*.exe"
            $dialog.Title = "Select {} Executable"
            if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{
                Write-Output $dialog.FileName
            }}
            "#,
            config.app_name, config.app_name, config.app_name
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .map_err(|e| format!("Failed to open file dialog: {}", e))?;

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() {
            return Err("User cancelled".to_string());
        }

        return Ok(Some(path));
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let script = format!(
            r#"
            tell application "System Events"
                activate
                set selectedApp to choose file of type {{"app"}} with prompt "Select {} Application"
                return POSIX path of selectedApp
            end tell
            "#,
            config.app_name
        );

        let output = Command::new("osascript")
            .args(["-e", &script])
            .output()
            .map_err(|e| format!("Failed to open file dialog: {}", e))?;

        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() || !output.status.success() {
            return Err("User cancelled".to_string());
        }

        return Ok(Some(path));
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        let title = format!("Select {} Executable", config.app_name);

        // 尝试使用 zenity
        let output = Command::new("zenity")
            .args(["--file-selection", &format!("--title={}", title)])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(Some(path));
                }
            }
        }

        // 尝试使用 kdialog
        let output = Command::new("kdialog")
            .args(["--getopenfilename", ".", "Executable files (*)"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(Some(path));
                }
            }
        }

        Err("User cancelled or no file dialog available".to_string())
    }
}

/// 从配置文件中读取自定义路径（非命令版本，用于内部调用）
pub fn read_custom_path_from_config(
    app: &AppHandle,
    config: &PlatformPathConfig,
) -> Option<String> {
    get_config_path(app, config)
        .ok()
        .and_then(|config_path| {
            if config_path.exists() {
                std::fs::read_to_string(&config_path).ok()
            } else {
                None
            }
        })
        .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
        .and_then(|json| {
            json.get("custom_path")
                .and_then(|v| v.as_str())
                .map(String::from)
        })
}
