//! Cursor 机器ID管理模块
//! 用于重置遥测数据，避免设备限制，修改 main.js

use super::db;
use crate::cursor::models::MachineInfo;
use rand::RngCore;
use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};
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
    pub service_machine_id: String, // state.vscdb 中的 storage.serviceMachineId
}

impl TelemetryIds {
    /// 生成新的遥测 ID
    pub fn generate() -> Self {
        let mut random_bytes_32 = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut random_bytes_32);

        Self {
            machine_id: format!("{:x}", Sha256::digest(&random_bytes_32)), // SHA256 哈希 (64字符)
            mac_machine_id: Uuid::new_v4().to_string(),                    // UUID 格式
            dev_device_id: Uuid::new_v4().to_string(),
            sqm_id: format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase()),
            service_machine_id: Uuid::new_v4().to_string(), // UUID 格式
        }
    }
}

/// 重置结果消息
#[derive(Debug, Clone)]
pub struct ResetResult {
    pub messages: Vec<String>,
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
        serde_json::from_str::<Value>(&content)
            .unwrap_or_else(|_| Value::Object(Default::default()))
    } else {
        Value::Object(Default::default())
    };

    // 生成新的遥测 ID
    let ids = TelemetryIds::generate();

    // 更新 storage.json 中的遥测字段
    if let Some(obj) = storage_data.as_object_mut() {
        obj.insert(
            "telemetry.machineId".to_string(),
            Value::String(ids.machine_id.clone()),
        );
        obj.insert(
            "telemetry.macMachineId".to_string(),
            Value::String(ids.mac_machine_id.clone()),
        );
        obj.insert(
            "telemetry.devDeviceId".to_string(),
            Value::String(ids.dev_device_id.clone()),
        );
        obj.insert(
            "telemetry.sqmId".to_string(),
            Value::String(ids.sqm_id.clone()),
        );

        // Windows 平台：重置 system.machineGuid
        #[cfg(windows)]
        {
            let new_machine_guid = Uuid::new_v4().to_string();
            obj.insert(
                "system.machineGuid".to_string(),
                Value::String(new_machine_guid),
            );
        }
    }

    // 写回文件
    let json = serde_json::to_string_pretty(&storage_data)
        .map_err(|e| format!("Failed to serialize storage.json: {}", e))?;
    fs::write(&storage_json_path, json)
        .map_err(|e| format!("Failed to write storage.json: {}", e))?;

    // 重置 state.vscdb 中的机器标识符 (serviceMachineId 和 stableID)
    db::reset_machine_ids_in_db(&ids.machine_id, Some(&ids.service_machine_id))
        .map_err(|e| format!("Failed to reset machine IDs in state.vscdb: {}", e))?;

    // Linux 平台：写入 machineid 文件并设为只读
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = write_linux_machineid_file(&ids.dev_device_id) {
            eprintln!("Warning: Failed to write Linux machineid file: {}", e);
        }
    }

    Ok(ids)
}

/*
/// Windows: 重置注册表 MachineGuid（已停用，不执行）
#[cfg(windows)]
fn reset_windows_registry_machine_guid() -> Result<(), String> {
    use winreg::RegKey;
    use winreg::enums::*;

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
*/

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
    #[cfg(target_os = "macos")]
    let mut exe_path = if let Some(path) = custom_exe_path {
        PathBuf::from(path)
    } else {
        get_cursor_executable_path()?
    };

    #[cfg(not(target_os = "macos"))]
    let exe_path = if let Some(path) = custom_exe_path {
        PathBuf::from(path)
    } else {
        get_cursor_executable_path()?
    };

    #[cfg(target_os = "macos")]
    if let Some(path) = custom_exe_path {
        let custom_path = PathBuf::from(path);
        let is_app_bundle = custom_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("app"))
            .unwrap_or(false);

        if is_app_bundle {
            let bundled_main_js = custom_path
                .join("Contents")
                .join("Resources")
                .join("app")
                .join("out")
                .join("main.js");
            if bundled_main_js.exists() {
                return Ok(bundled_main_js);
            }

            let bundled_exe = custom_path
                .join("Contents")
                .join("MacOS")
                .join("Cursor");
            if bundled_exe.exists() {
                exe_path = bundled_exe;
            }
        }
    }

    // 从可执行文件路径推断 main.js 路径
    if let Some(parent) = exe_path.parent() {
        // Windows: Cursor.exe 在根目录，resources/app 在同级
        // macOS: Cursor.app/Contents/MacOS/Cursor -> Cursor.app/Contents/Resources/app
        let main_js = if cfg!(target_os = "macos") {
            parent
                .parent()
                .map(|p| p.join("Resources").join("app").join("out").join("main.js"))
                .unwrap_or_else(|| {
                    parent
                        .join("resources")
                        .join("app")
                        .join("out")
                        .join("main.js")
                })
        } else {
            parent
                .join("resources")
                .join("app")
                .join("out")
                .join("main.js")
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
/// 任一步骤失败立即返回 Err，不继续后续操作
pub fn complete_reset(custom_exe_path: Option<&str>) -> Result<ResetResult, String> {
    reset_machine_id()?;
    modify_main_js(custom_exe_path)?;

    Ok(ResetResult {
        messages: vec!["Machine IDs reset and main.js modified successfully".to_string()],
    })
}

/// 写入指定的机器码到 storage.json
pub fn write_machine_ids(machine_info: &MachineInfo) -> Result<(), String> {
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
        serde_json::from_str::<Value>(&content)
            .unwrap_or_else(|_| Value::Object(Default::default()))
    } else {
        Value::Object(Default::default())
    };

    // 写入指定的机器码
    if let Some(obj) = storage_data.as_object_mut() {
        if let Some(ref machine_id) = machine_info.machine_id {
            obj.insert(
                "telemetry.machineId".to_string(),
                Value::String(machine_id.clone()),
            );
        }
        if let Some(ref mac_machine_id) = machine_info.mac_machine_id {
            obj.insert(
                "telemetry.macMachineId".to_string(),
                Value::String(mac_machine_id.clone()),
            );
        }
        if let Some(ref dev_device_id) = machine_info.dev_device_id {
            obj.insert(
                "telemetry.devDeviceId".to_string(),
                Value::String(dev_device_id.clone()),
            );
        }
        if let Some(ref sqm_id) = machine_info.sqm_id {
            obj.insert("telemetry.sqmId".to_string(), Value::String(sqm_id.clone()));
        }

        // Windows 平台：写入 system.machineGuid
        #[cfg(windows)]
        {
            if let Some(ref system_machine_guid) = machine_info.system_machine_guid {
                obj.insert(
                    "system.machineGuid".to_string(),
                    Value::String(system_machine_guid.clone()),
                );
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

    // 重置 state.vscdb 中的机器标识符 (serviceMachineId 和 stableID)
    let machine_id_for_db = if let Some(ref machine_id) = machine_info.machine_id {
        machine_id.clone()
    } else {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        format!("{:x}", Sha256::digest(&bytes))
    };
    let service_machine_id_for_db = machine_info.storage_service_machine_id.as_deref();
    db::reset_machine_ids_in_db(&machine_id_for_db, service_machine_id_for_db)
        .map_err(|e| format!("Failed to reset machine IDs in state.vscdb: {}", e))?;

    // Linux 平台：写入 machineid 文件并设为只读
    #[cfg(target_os = "linux")]
    {
        let dev_device_id = machine_info
            .dev_device_id
            .as_deref()
            .unwrap_or(&machine_id_for_db);
        if let Err(e) = write_linux_machineid_file(dev_device_id) {
            eprintln!("Warning: Failed to write Linux machineid file: {}", e);
        }
    }

    Ok(())
}

/// Linux 平台：写入 ~/.config/Cursor/machineid 文件并设为只读 (444)
#[cfg(target_os = "linux")]
fn write_linux_machineid_file(dev_device_id: &str) -> Result<(), String> {
    let home = dirs::home_dir().ok_or("Cannot get home directory")?;
    let cursor_config_dir = home.join(".config/Cursor");

    if !cursor_config_dir.exists() {
        return Ok(());
    }

    let machineid_path = cursor_config_dir.join("machineid");

    // 如果文件存在且只读，先恢复写权限
    if machineid_path.exists() {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o644);
        fs::set_permissions(&machineid_path, perms)
            .map_err(|e| format!("Failed to set machineid writable: {}", e))?;
    }

    fs::write(&machineid_path, dev_device_id)
        .map_err(|e| format!("Failed to write machineid: {}", e))?;

    // 设为只读 (444)，防止 Cursor 启动时覆盖
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o444);
    fs::set_permissions(&machineid_path, perms)
        .map_err(|e| format!("Failed to set machineid read-only: {}", e))?;

    Ok(())
}

/*
#[cfg(windows)]
fn write_windows_registry_machine_guid(guid: &str) -> Result<(), String> {
    use winreg::RegKey;
    use winreg::enums::*;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let crypto_key = hklm
        .open_subkey_with_flags(r"SOFTWARE\Microsoft\Cryptography", KEY_WRITE)
        .map_err(|e| format!("Failed to open Cryptography key (admin required): {}", e))?;

    crypto_key
        .set_value("MachineGuid", &guid)
        .map_err(|e| format!("Failed to set MachineGuid: {}", e))?;

    Ok(())
}
*/

/// 自动更新相关的常量
const AUTO_UPDATE_ORIGINAL_STR: &str = r#"!!this.args["disable-updates"]"#;
const AUTO_UPDATE_REPLACED_STR: &str = "true";

/// 检查 Cursor 自动更新是否已被禁用
/// 通过检测 main.js 中是否包含 `!!this.args["disable-updates"]` 来判断
/// 如果不包含该字符串，说明已被替换为 `true`，即已禁用
pub fn check_auto_update_disabled(custom_exe_path: Option<&str>) -> Result<bool, String> {
    let main_js_path = get_main_js_path(custom_exe_path)?;

    if !main_js_path.exists() {
        return Err(format!("main.js file not found: {}", main_js_path.display()));
    }

    let content =
        fs::read_to_string(&main_js_path).map_err(|e| format!("Failed to read main.js: {}", e))?;

    // 如果文件中不包含该字符串，说明已被替换，即已禁用自动更新
    Ok(!content.contains(AUTO_UPDATE_ORIGINAL_STR))
}

/// 禁用 Cursor 自动更新
/// 将 `!!this.args["disable-updates"]` 替换为 `true`
pub fn disable_auto_update(custom_exe_path: Option<&str>) -> Result<(), String> {
    // 检查 Cursor 是否正在运行
    if super::process::is_cursor_running() {
        return Err("请先关闭 Cursor 再执行此操作".to_string());
    }

    let main_js_path = get_main_js_path(custom_exe_path)?;

    if !main_js_path.exists() {
        return Err(format!("main.js file not found: {}", main_js_path.display()));
    }

    let content =
        fs::read_to_string(&main_js_path).map_err(|e| format!("Failed to read main.js: {}", e))?;

    // 检查是否已经禁用
    if !content.contains(AUTO_UPDATE_ORIGINAL_STR) {
        return Ok(()); // 已经禁用，无需操作
    }

    // 执行替换
    let modified_content = content.replace(AUTO_UPDATE_ORIGINAL_STR, AUTO_UPDATE_REPLACED_STR);

    // 写回文件
    fs::write(&main_js_path, modified_content)
        .map_err(|e| format!("Failed to write main.js: {}", e))?;

    Ok(())
}

/// 启用 Cursor 自动更新（恢复）
/// 由于原文件中该位置上下文为 `return!!this.args["disable-updates"]`，
/// 禁用后变为 `returntrue`，恢复时做反向替换即可
pub fn enable_auto_update(custom_exe_path: Option<&str>) -> Result<(), String> {
    // 检查 Cursor 是否正在运行
    if super::process::is_cursor_running() {
        return Err("请先关闭 Cursor 再执行此操作".to_string());
    }

    let main_js_path = get_main_js_path(custom_exe_path)?;

    if !main_js_path.exists() {
        return Err(format!("main.js file not found: {}", main_js_path.display()));
    }

    let content =
        fs::read_to_string(&main_js_path).map_err(|e| format!("Failed to read main.js: {}", e))?;

    // 检查是否已经禁用（如果原始字符串存在，说明未禁用，无需恢复）
    if content.contains(AUTO_UPDATE_ORIGINAL_STR) {
        return Ok(());
    }

    // 反向替换：用 `return` 前缀定位，将 `returntrue` 还原为 `return!!this.args["disable-updates"]`
    // 由于 minified JS 中 `return` 后面直接跟 `!!this.args[...]`，禁用后变为 `returntrue`
    let disabled_pattern = format!("return{}", AUTO_UPDATE_REPLACED_STR);
    let original_pattern = format!("return{}", AUTO_UPDATE_ORIGINAL_STR);

    if !content.contains(&disabled_pattern) {
        return Err("Cannot find the disabled auto-update pattern to restore".to_string());
    }

    let restored_content = content.replacen(&disabled_pattern, &original_pattern, 1);

    // 写回文件
    fs::write(&main_js_path, restored_content)
        .map_err(|e| format!("Failed to write main.js: {}", e))?;

    Ok(())
}

/// 检查 main.js 文件是否可写（macOS 需要 App Management 权限）
pub fn check_main_js_writable(custom_exe_path: Option<&str>) -> Result<bool, String> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = custom_exe_path;
        return Ok(true);
    }

    #[cfg(target_os = "macos")]
    {
        let main_js_path = get_main_js_path(custom_exe_path)?;
        if !main_js_path.exists() {
            return Err(format!("main.js not found: {}", main_js_path.display()));
        }

        match fs::OpenOptions::new().append(true).open(&main_js_path) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Ok(false),
            Err(e) => Err(format!("Failed to check main.js permission: {}", e)),
        }
    }
}

/// 使用绑定的机器码完成重置流程
/// 任一步骤失败立即返回 Err，不继续后续操作
pub fn complete_reset_with_machine_info(
    custom_exe_path: Option<&str>,
    machine_info: &MachineInfo,
) -> Result<ResetResult, String> {
    write_machine_ids(machine_info)?;
    modify_main_js(custom_exe_path)?;

    Ok(ResetResult {
        messages: vec!["Bound machine IDs written and main.js modified successfully".to_string()],
    })
}
