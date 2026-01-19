//! Windsurf state.vscdb 数据库操作模块

use rusqlite::Connection;
use base64::{Engine as _, engine::general_purpose};
use std::path::PathBuf;
use super::crypto::platform::{encrypt_for_secret_storage, encrypt_for_windsurf};
use serde::Serialize;
use uuid::Uuid;

/// 获取 Windsurf 数据库路径（跨平台）
pub fn get_db_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join("Library/Application Support/Windsurf/User/globalStorage/state.vscdb"))
    }
    
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Cannot get APPDATA environment variable".to_string())?;
        Ok(PathBuf::from(appdata).join("Windsurf\\User\\globalStorage\\state.vscdb"))
    }
    
    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join(".config/Windsurf/User/globalStorage/state.vscdb"))
    }
}

/// 检查数据库是否存在
pub fn check_db_exists() -> bool {
    match get_db_path() {
        Ok(path) => path.exists(),
        Err(_) => false,
    }
}

/// 注入 Token 到 Windsurf 数据库
pub fn inject_token(
    db_path: &PathBuf,
    id_token: &str,
    refresh_token: &str,
    expiry: i64,
    user_id: &str,
    email: &str,
) -> Result<String, String> {
    // 1. 打开数据库
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // 2. 构造 Firebase 用户数据 JSON
    let user_data = serde_json::json!({
        "uid": user_id,
        "email": email,
        "emailVerified": true,
        "isAnonymous": false,
        "spiVersion": "2.0.0",
        "stsTokenManager": {
            "refreshToken": refresh_token,
            "accessToken": id_token,
            "expirationTime": expiry * 1000  // 转换为毫秒
        },
        "createdAt": chrono::Utc::now().timestamp_millis().to_string(),
        "lastLoginAt": chrono::Utc::now().timestamp_millis().to_string(),
        "apiKey": "AIzaSyB7yJsv44i4dKTdzGJC1O8x3vjfrOOkero",
        "appName": "[DEFAULT]"
    });

    // 3. 加密数据
    let json_bytes = user_data.to_string().into_bytes();
    let encrypted = encrypt_for_windsurf(&json_bytes)?;
    
    // 4. Base64 编码
    let encrypted_b64 = general_purpose::STANDARD.encode(&encrypted);

    // 5. 写入数据库 - 更新 Firebase 用户状态
    let key = format!("firebase:authUser:{}:[DEFAULT]", 
        "AIzaSyB7yJsv44i4dKTdzGJC1O8x3vjfrOOkero");
    
    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?, ?)",
        [&key, &encrypted_b64],
    )
    .map_err(|e| format!("Failed to write auth data: {}", e))?;

    // 6. 清理可能的旧登录状态
    let _ = conn.execute(
        "DELETE FROM ItemTable WHERE key LIKE 'codeium.%' AND key LIKE '%logout%'",
        [],
    );

    Ok(format!("Token injected successfully to {:?}", db_path))
}

/// 清除 Windsurf 登录状态
pub fn clear_auth_state(db_path: &PathBuf) -> Result<(), String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // 删除 Firebase 认证数据
    conn.execute(
        "DELETE FROM ItemTable WHERE key LIKE 'firebase:authUser:%'",
        [],
    )
    .map_err(|e| format!("Failed to clear auth state: {}", e))?;

    Ok(())
}

#[derive(Serialize)]
struct BufferJson<'a> {
    #[serde(rename = "type")]
    type_field: &'a str,
    data: Vec<u8>,
}

#[derive(Serialize)]
struct SessionAccount {
    label: String,
    id: String,
}

#[derive(Serialize)]
struct SessionEntry {
    id: String,
    #[serde(rename = "accessToken")]
    access_token: String,
    account: SessionAccount,
    scopes: Vec<String>,
}

#[derive(Serialize)]
struct AuthStatus {
    name: String,
    #[serde(rename = "apiKey")]
    api_key: String,
    email: String,
    #[serde(rename = "teamId")]
    team_id: String,
    #[serde(rename = "planName")]
    plan_name: String,
}

#[derive(Serialize)]
struct CodeiumConfig {
    #[serde(rename = "codeium.installationId")]
    installation_id: String,
    #[serde(rename = "codeium.apiKey")]
    api_key: String,
    #[serde(rename = "apiServerUrl")]
    api_server_url: String,
    #[serde(rename = "codeium.hasOneTimeUpdatedUnspecifiedMode")]
    has_one_time_updated_unspecified_mode: bool,
}

fn buffer_json(data: &[u8]) -> Result<String, String> {
    let payload = BufferJson {
        type_field: "Buffer",
        data: data.to_vec(),
    };
    serde_json::to_string(&payload)
        .map_err(|e| format!("Failed to serialize encrypted buffer: {}", e))
}

fn write_item(tx: &rusqlite::Transaction<'_>, key: &str, value: &str) -> Result<(), String> {
    tx.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?, ?)",
        [key, value],
    )
    .map_err(|e| format!("Failed to write key {}: {}", key, e))?;
    Ok(())
}

pub fn write_windsurf_auth_state(
    db_path: &PathBuf,
    api_key: &str,
    api_server_url: &str,
    name: &str,
    email: &str,
) -> Result<(), String> {
    let mut conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // 清理旧登录状态
    tx.execute("DELETE FROM ItemTable WHERE key LIKE 'windsurf_auth-%'", [])
        .map_err(|e| format!("Failed to clear windsurf_auth entries: {}", e))?;
    tx.execute("DELETE FROM ItemTable WHERE key LIKE 'secret://%'", [])
        .map_err(|e| format!("Failed to clear secret entries: {}", e))?;
    tx.execute("DELETE FROM ItemTable WHERE key = 'windsurfAuthStatus'", [])
        .map_err(|e| format!("Failed to clear windsurfAuthStatus: {}", e))?;

    // 构建 sessions 数据并加密
    let session_id = Uuid::new_v4().to_string();
    let sessions = vec![SessionEntry {
        id: session_id,
        access_token: api_key.to_string(),
        account: SessionAccount {
            label: name.to_string(),
            id: name.to_string(),
        },
        scopes: Vec::new(),
    }];
    let sessions_json = serde_json::to_string(&sessions)
        .map_err(|e| format!("Failed to serialize sessions: {}", e))?;
    let encrypted = encrypt_for_secret_storage(sessions_json.as_bytes())?;

    let encrypted_json = buffer_json(&encrypted)?;

    let sessions_key = r#"secret://{"extensionId":"codeium.windsurf","key":"windsurf_auth.sessions"}"#;
    write_item(&tx, sessions_key, &encrypted_json)?;

    // windsurfAuthStatus 使用旧版格式
    let legacy = AuthStatus {
        name: name.to_string(),
        api_key: api_key.to_string(),
        email: email.to_string(),
        team_id: Uuid::new_v4().to_string(),
        plan_name: "Pro".to_string(),
    };
    let auth_status_json = serde_json::to_string(&legacy)
        .map_err(|e| format!("Failed to serialize windsurfAuthStatus: {}", e))?;

    // 验证序列化结果
    if auth_status_json == "null" || auth_status_json.is_empty() {
        return Err(format!(
            "AuthStatus serialization produced invalid value: '{}', params=({}, {}, {})",
            auth_status_json, name, api_key.len(), email
        ));
    }

    write_item(&tx, "windsurfAuthStatus", &auth_status_json)?;

    // codeium.windsurf
    let config = CodeiumConfig {
        installation_id: Uuid::new_v4().to_string(),
        api_key: api_key.to_string(),
        api_server_url: api_server_url.to_string(),
        has_one_time_updated_unspecified_mode: true,
    };
    let config_json = serde_json::to_string(&config)
        .map_err(|e| format!("Failed to serialize codeium config: {}", e))?;
    write_item(&tx, "codeium.windsurf", &config_json)?;
    write_item(&tx, "codeium.windsurf-windsurf_auth", name)?;

    tx.commit()
        .map_err(|e| format!("Failed to commit auth updates: {}", e))?;
    Ok(())
}
