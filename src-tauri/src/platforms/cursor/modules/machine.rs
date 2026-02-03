//! Cursor 机器ID管理模块
//! 用于重置遥测数据，避免设备限制，修改 main.js

use super::db;
use crate::cursor::models::MachineInfo;
use regex::Regex;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use rand::RngCore;
use sha2::{Digest, Sha256, Sha512};

/// 遥测 ID 结构
#[derive(Debug, Clone)]
pub struct TelemetryIds {
    pub machine_id: String,
    pub mac_machine_id: String,
    pub dev_device_id: String,
    pub sqm_id: String,
}

impl TelemetryIds {
    /// 生成新的遥测 ID（与 Windsurf 一致的生成逻辑）
    pub fn generate() -> Self {
        let mut random_bytes_32 = [0u8; 32];
        let mut random_bytes_64 = [0u8; 64];
        rand::thread_rng().fill_bytes(&mut random_bytes_32);
        rand::thread_rng().fill_bytes(&mut random_bytes_64);

        Self {
            machine_id: format!("{:x}", Sha256::digest(&random_bytes_32)),  // SHA256 哈希 (64字符)
            mac_machine_id: format!("{:x}", Sha512::digest(&random_bytes_64)), // SHA512 哈希 (128字符)
            dev_device_id: Uuid::new_v4().to_string(),
            sqm_id: format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase()),
        }
    }
}

/// 重置结果，包含是否需要管理员权限的信息
#[derive(Debug, Clone)]
pub struct ResetResult {
    pub messages: Vec<String>,
    pub needs_admin: bool,
}

/// 重置机器 ID（写入 storage.json）
/// 返回 (TelemetryIds, needs_admin) - needs_admin 表示 Windows 平台是否需要管理员权限
pub fn reset_machine_id() -> Result<(TelemetryIds, bool), String> {
    let storage_json_path = db::get_storage_json_path()?;
    #[allow(unused_mut)]
    let mut needs_admin = false;

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

        // Windows 平台：重置 system.machineGuid
        #[cfg(windows)]
        {
            let new_machine_guid = Uuid::new_v4().to_string();
            obj.insert("system.machineGuid".to_string(), Value::String(new_machine_guid));
        }
    }

    // 写回文件
    let json = serde_json::to_string_pretty(&storage_data)
        .map_err(|e| format!("Failed to serialize storage.json: {}", e))?;
    fs::write(&storage_json_path, json)
        .map_err(|e| format!("Failed to write storage.json: {}", e))?;

    // Windows 平台：尝试重置注册表 MachineGuid（需要管理员权限）
    #[cfg(windows)]
    {
        if let Err(_) = reset_windows_registry_machine_guid() {
            needs_admin = true;
        }
    }

    Ok((ids, needs_admin))
}

/// Windows: 重置注册表 MachineGuid
/// 需要管理员权限才能修改 HKEY_LOCAL_MACHINE
#[cfg(windows)]
fn reset_windows_registry_machine_guid() -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let crypto_key = hklm
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Cryptography", KEY_WRITE)
        .map_err(|e| format!("Failed to open Cryptography key (admin required): {}", e))?;

    let new_guid = Uuid::new_v4().to_string();
    crypto_key
        .set_value("MachineGuid", &new_guid)
        .map_err(|e| format!("Failed to set MachineGuid: {}", e))?;

    Ok(())
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
/// 优化：检查是否已经修改过，避免重复备份和修改
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

    // 定义需要替换的模式
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

    // 先检查是否需要修改（是否有匹配的模式）
    let mut needs_modification = false;
    for (pattern, _) in &patterns {
        let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;
        if re.is_match(&content) {
            needs_modification = true;
            break;
        }
    }

    // 如果已经修改过，直接返回
    if !needs_modification {
        return Ok(());
    }

    // 只在首次修改时创建备份（检查是否存在 .backup.original 文件）
    let original_backup_path = format!("{}.backup.original", main_js_path.display());
    if !std::path::Path::new(&original_backup_path).exists() {
        fs::copy(&main_js_path, &original_backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
    }

    // 应用正则表达式替换
    let mut modified_content = content;
    for (pattern, replacement) in &patterns {
        let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;
        if re.is_match(&modified_content) {
            modified_content = re.replace_all(&modified_content, *replacement).to_string();
        }
    }

    // 写回文件
    fs::write(&main_js_path, modified_content)
        .map_err(|e| format!("Failed to write main.js: {}", e))?;

    Ok(())
}

/// 完整重置流程：重置机器ID + 修改 main.js
/// 返回 ResetResult，包含操作结果消息和是否需要管理员权限
pub fn complete_reset(custom_exe_path: Option<&str>) -> Result<ResetResult, String> {
    let mut messages = Vec::new();
    let mut needs_admin = false;

    // 1. 重置机器ID
    match reset_machine_id() {
        Ok((_ids, admin_needed)) => {
            messages.push("✓ Machine IDs reset successfully".to_string());
            if admin_needed {
                needs_admin = true;
                #[cfg(windows)]
                messages.push("⚠ Windows registry MachineGuid requires admin privileges".to_string());
            }
        }
        Err(e) => messages.push(format!("✗ Failed to reset machine IDs: {}", e)),
    }

    // 2. 修改 main.js
    match modify_main_js(custom_exe_path) {
        Ok(()) => messages.push("✓ main.js modified successfully".to_string()),
        Err(e) => messages.push(format!("⚠ Failed to modify main.js: {}", e)),
    }

    Ok(ResetResult { messages, needs_admin })
}

/// 写入指定的机器码到 storage.json
/// 返回 (success, needs_admin) - needs_admin 表示 Windows 平台是否需要管理员权限
pub fn write_machine_ids(machine_info: &MachineInfo) -> Result<bool, String> {
    let storage_json_path = db::get_storage_json_path()?;
    #[allow(unused_mut)]
    let mut needs_admin = false;

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

    // 写入指定的机器码
    if let Some(obj) = storage_data.as_object_mut() {
        if let Some(ref machine_id) = machine_info.machine_id {
            obj.insert("telemetry.machineId".to_string(), Value::String(machine_id.clone()));
        }
        if let Some(ref mac_machine_id) = machine_info.mac_machine_id {
            obj.insert("telemetry.macMachineId".to_string(), Value::String(mac_machine_id.clone()));
        }
        if let Some(ref dev_device_id) = machine_info.dev_device_id {
            obj.insert("telemetry.devDeviceId".to_string(), Value::String(dev_device_id.clone()));
        }
        if let Some(ref sqm_id) = machine_info.sqm_id {
            obj.insert("telemetry.sqmId".to_string(), Value::String(sqm_id.clone()));
        }

        // Windows 平台：写入 system.machineGuid
        #[cfg(windows)]
        {
            if let Some(ref system_machine_guid) = machine_info.system_machine_guid {
                obj.insert("system.machineGuid".to_string(), Value::String(system_machine_guid.clone()));
            }
        }
    }

    // 写回文件 - 使用 OpenOptions 确保写入完成
    let json = serde_json::to_string_pretty(&storage_data)
        .map_err(|e| format!("Failed to serialize storage.json: {}", e))?;
    
    // 使用 File::create 和 sync_all 确保数据写入磁盘
    use std::io::Write;
    let mut file = fs::File::create(&storage_json_path)
        .map_err(|e| format!("Failed to create storage.json: {}", e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write storage.json: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync storage.json: {}", e))?;
    drop(file); // 显式关闭文件

    // Windows 平台：如果提供了 system.machineGuid，尝试写入注册表
    #[cfg(windows)]
    {
        if machine_info.system_machine_guid.is_some() {
            if let Err(_) = write_windows_registry_machine_guid(machine_info.system_machine_guid.as_ref().unwrap()) {
                needs_admin = true;
            }
        }
    }

    Ok(needs_admin)
}

/// Windows: 写入指定的 MachineGuid 到注册表
/// 需要管理员权限才能修改 HKEY_LOCAL_MACHINE
#[cfg(windows)]
fn write_windows_registry_machine_guid(guid: &str) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let crypto_key = hklm
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Cryptography", KEY_WRITE)
        .map_err(|e| format!("Failed to open Cryptography key (admin required): {}", e))?;

    crypto_key
        .set_value("MachineGuid", &guid)
        .map_err(|e| format!("Failed to set MachineGuid: {}", e))?;

    Ok(())
}

/// 使用绑定的机器码完成重置流程
/// 返回 ResetResult，包含操作结果消息和是否需要管理员权限
pub fn complete_reset_with_machine_info(custom_exe_path: Option<&str>, machine_info: &MachineInfo) -> Result<ResetResult, String> {
    let mut messages = Vec::new();
    let mut needs_admin = false;

    // 1. 写入绑定的机器码
    match write_machine_ids(machine_info) {
        Ok(admin_needed) => {
            messages.push("✓ Bound machine IDs written successfully".to_string());
            if admin_needed {
                needs_admin = true;
                #[cfg(windows)]
                messages.push("⚠ Windows registry MachineGuid requires admin privileges".to_string());
            }
        }
        Err(e) => messages.push(format!("✗ Failed to write bound machine IDs: {}", e)),
    }

    // 2. 修改 main.js
    match modify_main_js(custom_exe_path) {
        Ok(()) => messages.push("✓ main.js modified successfully".to_string()),
        Err(e) => messages.push(format!("⚠ Failed to modify main.js: {}", e)),
    }

    Ok(ResetResult { messages, needs_admin })
}

