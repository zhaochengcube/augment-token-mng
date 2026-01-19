//! 机器 ID 重置模块
//! 用于重置 Windsurf 的设备标识

use std::path::PathBuf;
use std::fs;
use uuid::Uuid;
use rand::RngCore;
use sha2::{Digest, Sha256, Sha512};
use serde_json::Value;

/// 获取 Windsurf 机器 ID 文件路径
pub fn get_machine_id_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join("Library/Application Support/Windsurf/machineid"))
    }
    
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Cannot get APPDATA environment variable".to_string())?;
        Ok(PathBuf::from(appdata).join("Windsurf\\machineid"))
    }
    
    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join(".config/Windsurf/machineid"))
    }
}

/// 获取 Windsurf 存储目录
pub fn get_storage_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join("Library/Application Support/Windsurf"))
    }
    
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Cannot get APPDATA environment variable".to_string())?;
        Ok(PathBuf::from(appdata).join("Windsurf"))
    }
    
    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join(".config/Windsurf"))
    }
}

/// 获取 Windsurf storage.json 路径
pub fn get_storage_json_path() -> Result<PathBuf, String> {
    let storage_path = get_storage_path()?;
    Ok(storage_path.join("User/globalStorage/storage.json"))
}

/// 读取当前机器 ID
pub fn read_machine_id() -> Result<Option<String>, String> {
    let path = get_machine_id_path()?;
    if !path.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read machine ID: {}", e))?;
    
    Ok(Some(content.trim().to_string()))
}

/// 生成新的机器 ID
pub fn generate_machine_id() -> String {
    Uuid::new_v4().to_string()
}

/// 重置机器 ID
pub fn reset_machine_id() -> Result<String, String> {
    let path = get_machine_id_path()?;
    
    // 备份旧的机器 ID
    if path.exists() {
        let backup_path = path.with_extension("bak");
        let _ = fs::copy(&path, &backup_path);
    }
    
    // 生成并写入新的机器 ID
    let new_id = generate_machine_id();
    
    // 确保目录存在
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }
    
    fs::write(&path, &new_id)
        .map_err(|e| format!("Failed to write machine ID: {}", e))?;

    // 同步更新 storage.json 遥测字段
    if let Err(e) = reset_telemetry_ids() {
        eprintln!("Failed to reset telemetry IDs: {}", e);
    }
    
    Ok(new_id)
}

/// 生成完整的遥测 ID 集合（与 Windsurf-Tool 一致）
pub struct TelemetryIds {
    pub machine_id: String,      // SHA256(32 随机字节)
    pub mac_machine_id: String,  // SHA512(64 随机字节) - macOS 专用
    pub sqm_id: String,          // {UUID-UPPERCASE}
    pub dev_device_id: String,   // UUID v4
    pub service_machine_id: String, // UUID v4
}

impl TelemetryIds {
    pub fn generate() -> Self {
        let mut random_bytes_32 = [0u8; 32];
        let mut random_bytes_64 = [0u8; 64];
        rand::thread_rng().fill_bytes(&mut random_bytes_32);
        rand::thread_rng().fill_bytes(&mut random_bytes_64);

        Self {
            machine_id: format!("{:x}", Sha256::digest(&random_bytes_32)),
            mac_machine_id: format!("{:x}", Sha512::digest(&random_bytes_64)),
            sqm_id: format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase()),
            dev_device_id: Uuid::new_v4().to_string(),
            service_machine_id: Uuid::new_v4().to_string(),
        }
    }
}

fn reset_telemetry_ids() -> Result<TelemetryIds, String> {
    let storage_json_path = get_storage_json_path()?;
    if let Some(parent) = storage_json_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create storage.json directory: {}", e))?;
        }
    }

    let mut storage_data = if storage_json_path.exists() {
        let content = fs::read_to_string(&storage_json_path)
            .map_err(|e| format!("Failed to read storage.json: {}", e))?;
        serde_json::from_str::<Value>(&content).unwrap_or_else(|_| Value::Object(Default::default()))
    } else {
        Value::Object(Default::default())
    };

    let ids = TelemetryIds::generate();

    if let Value::Object(ref mut map) = storage_data {
        map.insert("telemetry.machineId".to_string(), Value::String(ids.machine_id.clone()));
        map.insert("telemetry.sqmId".to_string(), Value::String(ids.sqm_id.clone()));
        map.insert("telemetry.devDeviceId".to_string(), Value::String(ids.dev_device_id.clone()));
        map.insert("telemetry.serviceMachineId".to_string(), Value::String(ids.service_machine_id.clone()));

        // macOS 专用字段
        #[cfg(target_os = "macos")]
        {
            map.insert("telemetry.macMachineId".to_string(), Value::String(ids.mac_machine_id.clone()));
        }
    }

    let json = serde_json::to_string_pretty(&storage_data)
        .map_err(|e| format!("Failed to serialize storage.json: {}", e))?;
    fs::write(&storage_json_path, json)
        .map_err(|e| format!("Failed to write storage.json: {}", e))?;

    // Windows: 尝试重置注册表 MachineGuid（需要管理员权限，失败不阻断）
    #[cfg(windows)]
    {
        if let Err(e) = reset_windows_machine_guid() {
            eprintln!("Warning: Failed to reset Windows MachineGuid (admin required): {}", e);
        }
    }

    Ok(ids)
}

/// Windows: 尝试重置注册表 MachineGuid
#[cfg(windows)]
fn reset_windows_machine_guid() -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let crypto_key = hklm
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Cryptography", KEY_WRITE)
        .map_err(|e| format!("Failed to open Cryptography key: {}", e))?;

    let new_guid = Uuid::new_v4().to_string();
    crypto_key
        .set_value("MachineGuid", &new_guid)
        .map_err(|e| format!("Failed to set MachineGuid: {}", e))?;

    Ok(())
}

/// 设置指定的机器 ID
pub fn set_machine_id(machine_id: &str) -> Result<(), String> {
    let path = get_machine_id_path()?;
    
    // 确保目录存在
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }
    
    fs::write(&path, machine_id)
        .map_err(|e| format!("Failed to write machine ID: {}", e))?;
    
    Ok(())
}

/// 清理 Windsurf 缓存（用于完全重置）
pub fn clear_cache() -> Result<(), String> {
    let storage_path = get_storage_path()?;
    
    // 需要清理的文件/目录列表
    let items_to_clear = [
        "User/globalStorage/state.vscdb",
        "User/globalStorage/state.vscdb.backup",
        "Cache",
        "CachedData",
        "GPUCache",
        "logs",
    ];
    
    for item in items_to_clear {
        let path = storage_path.join(item);
        if path.exists() {
            if path.is_dir() {
                let _ = fs::remove_dir_all(&path);
            } else {
                let _ = fs::remove_file(&path);
            }
        }
    }
    
    Ok(())
}
