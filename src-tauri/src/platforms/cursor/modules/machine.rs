//! Cursor 机器ID管理模块
//! 用于重置遥测数据，避免设备限制，修改 main.js

use super::db;
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// 遥测 ID 结构
#[derive(Debug, Clone)]
pub struct TelemetryIds {
    pub machine_id: String,
    pub mac_machine_id: String,
    pub dev_device_id: String,
    pub sqm_id: String,
}

impl TelemetryIds {
    /// 生成新的遥测 ID
    pub fn generate() -> Self {
        Self {
            machine_id: Uuid::new_v4().to_string(),
            mac_machine_id: Uuid::new_v4().to_string(),
            dev_device_id: Uuid::new_v4().to_string(),
            sqm_id: format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase()),
        }
    }
}

/// 重置机器 ID（写入 storage.json）
pub fn reset_machine_id() -> Result<TelemetryIds, String> {
    let storage_json_path = db::get_storage_json_path()?;

    // 确保目录存在
    if let Some(parent) = storage_json_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create storage.json directory: {}", e))?;
        }
    }

    // 读取现有内容或创建新的
    let mut storage_data = if storage_json_path.exists() {
        let content = fs::read_to_string(&storage_json_path)
            .map_err(|e| format!("Failed to read storage.json: {}", e))?;
        serde_json::from_str::<Value>(&content).unwrap_or_else(|_| Value::Object(Default::default()))
    } else {
        Value::Object(Default::default())
    };

    // 生成新的遥测 ID
    let ids = TelemetryIds::generate();

    // 更新 storage.json 中的遥测字段
    if let Some(obj) = storage_data.as_object_mut() {
        obj.insert("telemetry.machineId".to_string(), Value::String(ids.machine_id.clone()));
        obj.insert("telemetry.macMachineId".to_string(), Value::String(ids.mac_machine_id.clone()));
        obj.insert("telemetry.devDeviceId".to_string(), Value::String(ids.dev_device_id.clone()));
        obj.insert("telemetry.sqmId".to_string(), Value::String(ids.sqm_id.clone()));
    }

    // 写回文件
    let json = serde_json::to_string_pretty(&storage_data)
        .map_err(|e| format!("Failed to serialize storage.json: {}", e))?;
    fs::write(&storage_json_path, json)
        .map_err(|e| format!("Failed to write storage.json: {}", e))?;

    println!("Reset Cursor machine IDs in storage.json successfully");
    Ok(ids)
}

/// 获取当前机器 ID（从 storage.json 读取）
pub fn get_machine_id() -> Result<Option<String>, String> {
    let storage_json_path = db::get_storage_json_path()?;

    if !storage_json_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&storage_json_path)
        .map_err(|e| format!("Failed to read storage.json: {}", e))?;

    let storage_data: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse storage.json: {}", e))?;

    let machine_id = storage_data
        .get("telemetry.machineId")
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok(machine_id)
}

/// 根据可执行文件路径获取 main.js 路径
/// 支持自定义路径，如果提供了自定义可执行文件路径，会推断出 resources/app/out/main.js 路径
/// 如果没有提供自定义路径，则使用 get_cursor_executable_path() 获取默认路径
pub fn get_main_js_path(custom_exe_path: Option<&str>) -> Result<PathBuf, String> {
    use super::process::get_cursor_executable_path;

    // 获取可执行文件路径：优先使用自定义路径，否则使用默认路径
    let exe_path = if let Some(path) = custom_exe_path {
        PathBuf::from(path)
    } else {
        get_cursor_executable_path()?
    };

    // 从可执行文件路径推断 main.js 路径
    if let Some(parent) = exe_path.parent() {
        // Windows: Cursor.exe 在根目录，resources/app 在同级
        // macOS: Cursor.app/Contents/MacOS/Cursor -> Cursor.app/Contents/Resources/app
        let main_js = if cfg!(target_os = "macos") {
            parent
                .parent()
                .map(|p| p.join("Resources").join("app").join("out").join("main.js"))
                .unwrap_or_else(|| parent.join("resources").join("app").join("out").join("main.js"))
        } else {
            parent.join("resources").join("app").join("out").join("main.js")
        };

        if main_js.exists() {
            return Ok(main_js);
        }

        return Err(format!("main.js not found at: {}", main_js.display()));
    }

    Err(format!("Invalid executable path: {}", exe_path.display()))
}

/// 修改 main.js
pub fn modify_main_js(custom_exe_path: Option<&str>) -> Result<(), String> {
    let main_js_path = get_main_js_path(custom_exe_path)?;

    if !main_js_path.exists() {
        return Err(format!(
            "main.js file not found: {}",
            main_js_path.display()
        ));
    }

    // 读取文件内容
    let content =
        fs::read_to_string(&main_js_path).map_err(|e| format!("Failed to read main.js: {}", e))?;

    // 创建备份
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_path = format!("{}.backup.{}", main_js_path.display(), timestamp);
    fs::copy(&main_js_path, &backup_path)
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    // 应用正则表达式替换 - 绕过 getMachineId 检测
    // 匹配 async getMachineId(){return xxx??yyy} 替换为 async getMachineId(){return yyy}
    let patterns = vec![
        (
            r"async getMachineId\(\)\{return [^?]+\?\?([^}]+)\}",
            r"async getMachineId(){return $1}",
        ),
        (
            r"async getMacMachineId\(\)\{return [^?]+\?\?([^}]+)\}",
            r"async getMacMachineId(){return $1}",
        ),
    ];

    let mut modified_content = content.clone();
    let mut patterns_applied = 0;

    for (pattern, replacement) in patterns {
        let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;
        // 使用 is_match 先检测是否存在匹配
        if re.is_match(&modified_content) {
            modified_content = re.replace_all(&modified_content, replacement).to_string();
            patterns_applied += 1;
        }
    }

    // 写回文件
    fs::write(&main_js_path, modified_content)
        .map_err(|e| format!("Failed to write main.js: {}", e))?;

    println!(
        "Modified main.js successfully, {} patterns applied",
        patterns_applied
    );
    Ok(())
}

/// 完整重置流程：重置机器ID + 修改 main.js
pub fn complete_reset(custom_exe_path: Option<&str>) -> Result<Vec<String>, String> {
    let mut results = Vec::new();

    // 1. 重置机器ID
    match reset_machine_id() {
        Ok(_ids) => results.push("✓ Machine IDs reset successfully".to_string()),
        Err(e) => results.push(format!("✗ Failed to reset machine IDs: {}", e)),
    }

    // 2. 修改 main.js
    match modify_main_js(custom_exe_path) {
        Ok(()) => results.push("✓ main.js modified successfully".to_string()),
        Err(e) => results.push(format!("⚠ Failed to modify main.js: {}", e)),
    }

    Ok(results)
}

