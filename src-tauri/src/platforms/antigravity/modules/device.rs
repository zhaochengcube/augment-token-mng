use crate::antigravity::models::DeviceProfile;
use rand::{distributions::Alphanumeric, Rng};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// 获取 Antigravity storage.json 路径，优先使用 `--user-data-dir` 和便携模式目录。
pub fn get_storage_path(custom_executable_path: Option<&str>) -> Result<PathBuf, String> {
    if let Some(path) = user_data_dir_storage_path() {
        if path.exists() {
            return Ok(path);
        }
    }

    if let Some(path) = portable_storage_path(custom_executable_path) {
        if path.exists() {
            return Ok(path);
        }
    }

    if let Ok(default_exe) = crate::antigravity::modules::process::get_antigravity_executable_path()
    {
        if let Some(path) = portable_storage_path(default_exe.to_str()) {
            if path.exists() {
                return Ok(path);
            }
        }
    }

    default_storage_path()
}

fn user_data_dir_storage_path() -> Option<PathBuf> {
    crate::antigravity::modules::process::get_user_data_dir_from_process()
        .map(|dir| dir.join("User").join("globalStorage").join("storage.json"))
}

fn portable_storage_path(executable_path: Option<&str>) -> Option<PathBuf> {
    let executable_path = executable_path?;
    let path = PathBuf::from(executable_path);

    #[cfg(target_os = "macos")]
    {
        if path.extension().and_then(|ext| ext.to_str()) == Some("app") {
            return path
                .parent()
                .map(|parent| parent.join("data/user-data/User/globalStorage/storage.json"));
        }
    }

    path.parent()
        .map(|parent| parent.join("data/user-data/User/globalStorage/storage.json"))
}

fn default_storage_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join("Library/Application Support/Antigravity/User/globalStorage/storage.json"))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Cannot get APPDATA environment variable".to_string())?;
        Ok(PathBuf::from(appdata).join("Antigravity\\User\\globalStorage\\storage.json"))
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join(".config/Antigravity/User/globalStorage/storage.json"))
    }
}

/// 生成一组与 Antigravity/VS Code telemetry 兼容的设备指纹。
pub fn generate_profile() -> DeviceProfile {
    DeviceProfile {
        machine_id: format!("auth0|user_{}", random_hex(32)),
        mac_machine_id: new_standard_machine_id(),
        dev_device_id: Uuid::new_v4().to_string(),
        sqm_id: format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase()),
    }
}

pub fn write_profile(storage_path: &Path, profile: &DeviceProfile) -> Result<(), String> {
    if !storage_path.exists() {
        return Err(format!("storage.json not found: {:?}", storage_path));
    }

    let content = fs::read_to_string(storage_path)
        .map_err(|e| format!("Failed to read storage.json: {}", e))?;
    let mut json: Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse storage.json: {}", e))?;

    if !json.is_object() {
        return Err("storage.json top-level value is not an object".to_string());
    }

    if !json.get("telemetry").map_or(false, |value| value.is_object()) {
        json["telemetry"] = serde_json::json!({});
    }

    if let Some(telemetry) = json.get_mut("telemetry").and_then(|value| value.as_object_mut()) {
        telemetry.insert(
            "machineId".to_string(),
            Value::String(profile.machine_id.clone()),
        );
        telemetry.insert(
            "macMachineId".to_string(),
            Value::String(profile.mac_machine_id.clone()),
        );
        telemetry.insert(
            "devDeviceId".to_string(),
            Value::String(profile.dev_device_id.clone()),
        );
        telemetry.insert("sqmId".to_string(), Value::String(profile.sqm_id.clone()));
    }

    if let Some(map) = json.as_object_mut() {
        map.insert(
            "telemetry.machineId".to_string(),
            Value::String(profile.machine_id.clone()),
        );
        map.insert(
            "telemetry.macMachineId".to_string(),
            Value::String(profile.mac_machine_id.clone()),
        );
        map.insert(
            "telemetry.devDeviceId".to_string(),
            Value::String(profile.dev_device_id.clone()),
        );
        map.insert(
            "telemetry.sqmId".to_string(),
            Value::String(profile.sqm_id.clone()),
        );
        map.insert(
            "storage.serviceMachineId".to_string(),
            Value::String(profile.dev_device_id.clone()),
        );
    }

    let updated = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize storage.json: {}", e))?;
    fs::write(storage_path, updated).map_err(|e| format!("Failed to write storage.json: {}", e))
}

fn random_hex(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}

fn new_standard_machine_id() -> String {
    let mut rng = rand::thread_rng();
    let mut id = String::with_capacity(36);

    for ch in "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".chars() {
        if ch == '-' || ch == '4' {
            id.push(ch);
        } else if ch == 'x' {
            id.push_str(&format!("{:x}", rng.gen_range(0..16)));
        } else if ch == 'y' {
            id.push_str(&format!("{:x}", rng.gen_range(8..12)));
        }
    }

    id
}
