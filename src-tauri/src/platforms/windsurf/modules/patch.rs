//! Windsurf extension.js patch management for seamless account switching.

use crate::core::path_manager::{WINDSURF_CONFIG, get_config_path};
use crate::windsurf::modules::process;
use regex::Regex;
use serde::Serialize;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;

const PATTERN_URI_HANDLER_ORIGINAL: &str = r#"this\._uriHandler\.event\((\w+)=>\{"/refresh-authentication-session"===(\w+)\.path&&\(0,(\w+)\.refreshAuthenticationSession\)\(\)\}\)"#;
const PATTERN_URI_HANDLER_CURRENT: &str = r#"this\._uriHandler\.event\((\w+)=>\{"/refresh-authentication-session"===(\w+)\.path\?\(0,(\w+)\.refreshAuthenticationSession\)\(\):this\._loginInProgress\|\|this\.maybeHandleUriWithToken\((\w+)\)\}\)"#;
const PATTERN_TIMEOUT_ORIGINAL: &str =
    r#",new Promise\(\((\w+),(\w+)\)=>setTimeout\(\(\)=>\{(\w+)\(new (\w+)\)\},18e4\)\)"#;
const PATTERN_DIFF_ACCOUNT_PROMPT_ORIGINAL: &str = r#"if\("Yes"===await (\w+)\.window\.showWarningMessage\("Are you sure you want to log in using a different account\?",\{modal:!0\},"Yes"\)\)"#;
const PATTERN_DIFF_ACCOUNT_PROMPT_CURRENT: &str =
    r#"if\("Yes"===await (\w+)\.window\.showWarningMessage\((\w+),\{modal:!0\},"Yes"\)\)"#;
const CURRENT_VERSION_MARKER: &str = "Failed to handle OAuth callback";

#[derive(Debug, Clone, Serialize)]
pub struct PatchStatus {
    pub enabled: bool,
    pub applied: bool,
    pub extension_path: Option<String>,
    pub backup_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatchApplyResult {
    pub applied: bool,
    pub already_patched: bool,
    pub restarted: bool,
    pub extension_path: String,
    pub backup_path: Option<String>,
    pub message: String,
}

pub fn get_patch_status(app: &AppHandle, executable_path: Option<&str>) -> PatchStatus {
    let config = read_config_json(app).unwrap_or_else(|_| json!({}));
    let enabled = config
        .get("seamless_switch_enabled")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let backup_path = config
        .get("patch_backup_path")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    match resolve_extension_file(executable_path) {
        Ok(extension_path) => match fs::read_to_string(&extension_path) {
            Ok(content) => PatchStatus {
                enabled,
                applied: is_patched_content(&content),
                extension_path: Some(extension_path.to_string_lossy().to_string()),
                backup_path,
                error: None,
            },
            Err(e) => PatchStatus {
                enabled,
                applied: false,
                extension_path: Some(extension_path.to_string_lossy().to_string()),
                backup_path,
                error: Some(format!("读取 extension.js 失败: {}", e)),
            },
        },
        Err(e) => PatchStatus {
            enabled,
            applied: false,
            extension_path: None,
            backup_path,
            error: Some(e),
        },
    }
}

pub fn ensure_seamless_patch(
    app: &AppHandle,
    executable_path: Option<&str>,
) -> Result<PatchApplyResult, String> {
    let extension_file = resolve_extension_file(executable_path)?;
    if !extension_file.exists() {
        return Err(format!("extension.js 不存在: {}", extension_file.display()));
    }

    let content = fs::read_to_string(&extension_file)
        .map_err(|e| format!("读取 extension.js 失败: {}", e))?;
    if is_patched_content(&content) {
        update_config_json(app, |json| {
            json["seamless_switch_enabled"] = Value::Bool(true);
        })?;
        return Ok(PatchApplyResult {
            applied: true,
            already_patched: true,
            restarted: false,
            extension_path: extension_file.to_string_lossy().to_string(),
            backup_path: get_config_string(app, "patch_backup_path"),
            message: "无感切号补丁已存在".to_string(),
        });
    }

    let patched = patch_extension_content(&content)?;
    let parent = extension_file
        .parent()
        .ok_or_else(|| "无法获取 extension.js 所在目录".to_string())?;
    rotate_backups(parent)?;
    let backup_file = extension_file.with_file_name(format!(
        "extension.js.backup.{}",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    ));
    fs::copy(&extension_file, &backup_file)
        .map_err(|e| format!("备份 extension.js 失败: {}", e))?;
    fs::write(&extension_file, patched).map_err(|e| format!("写入 extension.js 失败: {}", e))?;

    update_config_json(app, |json| {
        json["seamless_switch_enabled"] = Value::Bool(true);
        json["patch_backup_path"] = Value::String(backup_file.to_string_lossy().to_string());
    })?;

    let restarted = restart_windsurf_if_running(executable_path)?;

    Ok(PatchApplyResult {
        applied: true,
        already_patched: false,
        restarted,
        extension_path: extension_file.to_string_lossy().to_string(),
        backup_path: Some(backup_file.to_string_lossy().to_string()),
        message: if restarted {
            "无感切号补丁已应用，Windsurf 已重启".to_string()
        } else {
            "无感切号补丁已应用".to_string()
        },
    })
}

pub fn restore_seamless_patch(
    app: &AppHandle,
    executable_path: Option<&str>,
) -> Result<PatchApplyResult, String> {
    let extension_file = resolve_extension_file(executable_path)?;
    let backup_file = get_config_string(app, "patch_backup_path")
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .or_else(|| {
            extension_file
                .parent()
                .and_then(|dir| find_latest_backup(dir).ok())
        })
        .ok_or_else(|| "未找到可用的 extension.js 备份".to_string())?;

    fs::copy(&backup_file, &extension_file)
        .map_err(|e| format!("还原 extension.js 失败: {}", e))?;
    update_config_json(app, |json| {
        json["seamless_switch_enabled"] = Value::Bool(false);
    })?;

    let restarted = restart_windsurf_if_running(executable_path)?;

    Ok(PatchApplyResult {
        applied: false,
        already_patched: false,
        restarted,
        extension_path: extension_file.to_string_lossy().to_string(),
        backup_path: Some(backup_file.to_string_lossy().to_string()),
        message: if restarted {
            "无感切号补丁已还原，Windsurf 已重启".to_string()
        } else {
            "无感切号补丁已还原".to_string()
        },
    })
}

fn patch_extension_content(content: &str) -> Result<String, String> {
    let pattern1 = Regex::new(PATTERN_URI_HANDLER_ORIGINAL)
        .map_err(|e| format!("OAuth 回调正则无效: {}", e))?;
    let pattern2 =
        Regex::new(PATTERN_TIMEOUT_ORIGINAL).map_err(|e| format!("超时正则无效: {}", e))?;
    let pattern3 = Regex::new(PATTERN_DIFF_ACCOUNT_PROMPT_ORIGINAL)
        .map_err(|e| format!("切号确认正则无效: {}", e))?;

    let mut modified = content.to_string();

    if let Some(captures) = pattern1.captures(&modified) {
        let var_name1 = &captures[1];
        let var_name2 = &captures[2];
        let module_name = &captures[3];
        if var_name1 == var_name2 {
            let replacement = format!(
                r#"this._uriHandler.event(async {}=>{{if("/refresh-authentication-session"==={}.path){{(0,{}.refreshAuthenticationSession)()}}else{{try{{const t=new URLSearchParams({}.fragment).get("access_token");if(!t)throw new Error("No access_token in URI fragment");await this.handleAuthToken(t)}}catch(e){{console.error("[Windsurf] Failed to handle OAuth callback:",e)}}}}}})"#,
                var_name1, var_name1, module_name, var_name1
            );
            if let Some(matched) = captures.get(0) {
                modified = modified.replace(matched.as_str(), &replacement);
            }
        }
    }

    let pattern1_current = Regex::new(PATTERN_URI_HANDLER_CURRENT)
        .map_err(|e| format!("当前版本 OAuth 回调正则无效: {}", e))?;
    if let Some(captures) = pattern1_current.captures(&modified) {
        let uri_var = &captures[1];
        let path_var = &captures[2];
        let module_name = &captures[3];
        let handler_var = &captures[4];
        if uri_var == path_var && uri_var == handler_var {
            let replacement = format!(
                r#"this._uriHandler.event(async {}=>{{if("/refresh-authentication-session"==={}.path){{(0,{}.refreshAuthenticationSession)()}}else{{try{{const t=new URLSearchParams({}.fragment).get("access_token");if(!t)throw new Error("No access_token in URI fragment");await this.handleAuthToken(t)}}catch(e){{console.error("[Windsurf] Failed to handle OAuth callback:",e)}}}}}})"#,
                uri_var, uri_var, module_name, uri_var
            );
            if let Some(matched) = captures.get(0) {
                modified = modified.replace(matched.as_str(), &replacement);
            }
        }
    }

    if let Some(captures) = pattern2.captures(&modified) {
        let reject_var1 = &captures[2];
        let reject_var2 = &captures[3];
        if reject_var1 == reject_var2 {
            if let Some(matched) = captures.get(0) {
                modified = modified.replace(matched.as_str(), "");
            }
        }
    }

    if let Some(captures) = pattern3.captures(&modified) {
        if let Some(matched) = captures.get(0) {
            modified = modified.replace(matched.as_str(), "if(true)");
        }
    }

    let pattern3_current = Regex::new(PATTERN_DIFF_ACCOUNT_PROMPT_CURRENT)
        .map_err(|e| format!("当前版本切号确认正则无效: {}", e))?;
    if let Some(captures) = pattern3_current.captures(&modified) {
        if let Some(matched) = captures.get(0) {
            modified = modified.replace(matched.as_str(), "if(true)");
        }
    }

    if modified == content {
        return Err("未能匹配当前 Windsurf extension.js，可能版本不兼容".to_string());
    }

    Ok(modified)
}

fn is_patched_content(content: &str) -> bool {
    content.contains(CURRENT_VERSION_MARKER)
}

fn resolve_extension_file(executable_path: Option<&str>) -> Result<PathBuf, String> {
    let executable = if let Some(path) = executable_path.filter(|p| !p.trim().is_empty()) {
        PathBuf::from(path)
    } else {
        process::get_windsurf_executable_path()?
    };
    let root = executable_to_install_root(&executable)?;
    Ok(root.join(get_extension_js_relative_path()))
}

fn executable_to_install_root(path: &Path) -> Result<PathBuf, String> {
    if path.is_dir() {
        return Ok(path.to_path_buf());
    }

    #[cfg(target_os = "macos")]
    {
        let mut current = path.to_path_buf();
        while let Some(parent) = current.parent() {
            if current.extension().and_then(|s| s.to_str()) == Some("app") {
                return Ok(current);
            }
            current = parent.to_path_buf();
        }
    }

    path.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| format!("无法从路径推导 Windsurf 安装目录: {}", path.display()))
}

fn get_extension_js_relative_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("Contents")
            .join("Resources")
            .join("app")
            .join("extensions")
            .join("windsurf")
            .join("dist")
            .join("extension.js")
    }
    #[cfg(not(target_os = "macos"))]
    {
        PathBuf::from("resources")
            .join("app")
            .join("extensions")
            .join("windsurf")
            .join("dist")
            .join("extension.js")
    }
}

fn restart_windsurf_if_running(executable_path: Option<&str>) -> Result<bool, String> {
    if !process::is_windsurf_running() {
        return Ok(false);
    }
    process::close_windsurf(20)?;
    std::thread::sleep(std::time::Duration::from_millis(800));
    process::launch_windsurf_with_path(executable_path)?;
    Ok(true)
}

fn rotate_backups(dir: &Path) -> Result<(), String> {
    let mut backups: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("读取补丁目录失败: {}", e))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("extension.js.backup."))
                .unwrap_or(false)
        })
        .collect();

    backups.sort_by_key(|path| fs::metadata(path).and_then(|meta| meta.modified()).ok());
    while backups.len() >= 3 {
        if let Some(oldest) = backups.first() {
            let _ = fs::remove_file(oldest);
        }
        backups.remove(0);
    }

    Ok(())
}

fn find_latest_backup(dir: &Path) -> Result<PathBuf, String> {
    let mut backups: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("读取补丁目录失败: {}", e))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("extension.js.backup."))
                .unwrap_or(false)
        })
        .collect();

    backups.sort_by_key(|path| fs::metadata(path).and_then(|meta| meta.modified()).ok());
    backups
        .pop()
        .ok_or_else(|| "未找到 extension.js 备份".to_string())
}

fn get_config_string(app: &AppHandle, key: &str) -> Option<String> {
    read_config_json(app).ok().and_then(|json| {
        json.get(key)
            .and_then(Value::as_str)
            .map(ToString::to_string)
    })
}

fn read_config_json(app: &AppHandle) -> Result<Value, String> {
    let config_path = get_config_path(app, &WINDSURF_CONFIG)?;
    if !config_path.exists() {
        return Ok(json!({}));
    }
    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("读取 Windsurf 配置失败: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("解析 Windsurf 配置失败: {}", e))
}

fn update_config_json<F>(app: &AppHandle, update: F) -> Result<(), String>
where
    F: FnOnce(&mut Value),
{
    let config_path = get_config_path(app, &WINDSURF_CONFIG)?;
    let mut json = read_config_json(app).unwrap_or_else(|_| json!({}));
    update(&mut json);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建 Windsurf 配置目录失败: {}", e))?;
    }
    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化 Windsurf 配置失败: {}", e))?;
    fs::write(&config_path, content).map_err(|e| format!("写入 Windsurf 配置失败: {}", e))
}
