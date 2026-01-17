use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

/// Antigravity 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AntigravityConfig {
    /// 是否使用自定义路径
    pub use_custom_path: bool,
    /// 自定义可执行文件路径
    pub custom_executable_path: Option<String>,
}

impl Default for AntigravityConfig {
    fn default() -> Self {
        Self {
            use_custom_path: false,
            custom_executable_path: None,
        }
    }
}

impl AntigravityConfig {
    /// 验证自定义路径是否有效
    pub fn validate_custom_path(&self) -> Result<(), String> {
        if !self.use_custom_path {
            return Ok(());
        }

        let path = self.custom_executable_path
            .as_ref()
            .ok_or("自定义路径未设置")?;

        let path_buf = PathBuf::from(path);
        
        if !path_buf.exists() {
            return Err(format!("路径不存在: {}", path));
        }

        if !path_buf.is_file() {
            return Err(format!("路径不是文件: {}", path));
        }

        // 在 Unix 系统上检查可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&path_buf)
                .map_err(|e| format!("无法读取文件元数据: {}", e))?;
            let permissions = metadata.permissions();
            if permissions.mode() & 0o111 == 0 {
                return Err(format!("文件不可执行: {}", path));
            }
        }

        Ok(())
    }

    /// 获取有效的可执行文件路径（如果配置了自定义路径）
    pub fn get_executable_path(&self) -> Option<PathBuf> {
        if self.use_custom_path {
            self.custom_executable_path
                .as_ref()
                .map(PathBuf::from)
        } else {
            None
        }
    }
}

/// 获取配置文件路径
fn get_config_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法获取应用数据目录: {}", e))?;

    // 确保目录存在
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("无法创建应用数据目录: {}", e))?;

    Ok(app_data_dir.join("antigravity_config.json"))
}

/// 保存 Antigravity 配置到文件
pub fn save_config(app_handle: &tauri::AppHandle, config: &AntigravityConfig) -> Result<(), String> {
    // 先验证配置
    config.validate_custom_path()?;

    let config_path = get_config_path(app_handle)?;

    // 序列化配置
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("配置序列化失败: {}", e))?;

    // 写入文件
    fs::write(&config_path, json)
        .map_err(|e| format!("配置文件写入失败: {}", e))?;

    Ok(())
}

/// 从文件加载 Antigravity 配置
pub fn load_config(app_handle: &tauri::AppHandle) -> Result<AntigravityConfig, String> {
    let config_path = get_config_path(app_handle)?;

    // 如果文件不存在，返回默认配置
    if !config_path.exists() {
        return Ok(AntigravityConfig::default());
    }

    // 读取文件
    let json = fs::read_to_string(&config_path)
        .map_err(|e| format!("配置文件读取失败: {}", e))?;

    // 反序列化配置
    let config: AntigravityConfig = serde_json::from_str(&json)
        .map_err(|e| format!("配置文件解析失败: {}", e))?;

    Ok(config)
}

/// 删除配置文件
pub fn delete_config(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let config_path = get_config_path(app_handle)?;
    
    if config_path.exists() {
        fs::remove_file(&config_path)
            .map_err(|e| format!("配置文件删除失败: {}", e))?;
    }
    
    Ok(())
}

/// 检查配置文件是否存在
pub fn config_exists(app_handle: &tauri::AppHandle) -> bool {
    match get_config_path(app_handle) {
        Ok(path) => path.exists(),
        Err(_) => false,
    }
}

