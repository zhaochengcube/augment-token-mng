use crate::antigravity::utils::protobuf;
use base64::{Engine as _, engine::general_purpose};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

const LEGACY_AGENT_STATE_KEY: &str = "jetskiStateSync.agentManagerInitState";
const UNIFIED_OAUTH_STATE_KEY: &str = "antigravityUnifiedStateSync.oauthToken";
const UNIFIED_USER_STATUS_KEY: &str = "antigravityUnifiedStateSync.userStatus";
const UNIFIED_ENTERPRISE_PREFS_KEY: &str = "antigravityUnifiedStateSync.enterprisePreferences";
const ONBOARDING_KEY: &str = "antigravityOnboarding";
const SERVICE_MACHINE_ID_KEY: &str = "telemetry.serviceMachineId";

/// 获取 Antigravity 数据库路径（跨平台）
pub fn get_db_path() -> Result<PathBuf, String> {
    get_db_path_for_executable(None)
}

pub fn get_db_path_for_storage(storage_path: &Path) -> PathBuf {
    storage_path
        .parent()
        .map(|parent| parent.join("state.vscdb"))
        .unwrap_or_else(|| storage_path.with_file_name("state.vscdb"))
}

pub fn get_db_path_for_executable(custom_executable_path: Option<&str>) -> Result<PathBuf, String> {
    if let Some(path) = user_data_dir_db_path() {
        if path.exists() {
            return Ok(path);
        }
    }

    if let Some(path) = portable_db_path(custom_executable_path) {
        if path.exists() {
            return Ok(path);
        }
    }

    if let Ok(default_exe) = crate::antigravity::modules::process::get_antigravity_executable_path()
    {
        if let Some(path) = portable_db_path(default_exe.to_str()) {
            if path.exists() {
                return Ok(path);
            }
        }
    }

    default_db_path()
}

fn user_data_dir_db_path() -> Option<PathBuf> {
    crate::antigravity::modules::process::get_user_data_dir_from_process()
        .map(|dir| dir.join("User").join("globalStorage").join("state.vscdb"))
}

fn portable_db_path(executable_path: Option<&str>) -> Option<PathBuf> {
    let executable_path = executable_path?;
    let path = PathBuf::from(executable_path);

    #[cfg(target_os = "macos")]
    {
        if path.extension().and_then(|ext| ext.to_str()) == Some("app") {
            return path
                .parent()
                .map(|parent| parent.join("data/user-data/User/globalStorage/state.vscdb"));
        }
    }

    path.parent()
        .map(|parent| parent.join("data/user-data/User/globalStorage/state.vscdb"))
}

fn default_db_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join("Library/Application Support/Antigravity/User/globalStorage/state.vscdb"))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Cannot get APPDATA environment variable".to_string())?;
        Ok(PathBuf::from(appdata).join("Antigravity\\User\\globalStorage\\state.vscdb"))
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join(".config/Antigravity/User/globalStorage/state.vscdb"))
    }
}

/// 注入 Token 到 Antigravity 数据库
pub fn inject_token(
    db_path: &PathBuf,
    access_token: &str,
    refresh_token: &str,
    expiry: i64,
    email: &str,
    mut is_gcp_tos: bool,
    project_id: Option<&str>,
    id_token: Option<&str>,
    oauth_client_key: Option<&str>,
    custom_executable_path: Option<&str>,
) -> Result<String, String> {
    let conn = Connection::open(db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    if matches!(oauth_client_key, Some("antigravity_enterprise")) {
        is_gcp_tos = false;
    }

    match crate::antigravity::modules::version::get_antigravity_version_for_path(custom_executable_path) {
        Ok(version) => {
            if crate::antigravity::modules::version::is_new_version(&version) {
                inject_unified_oauth_state(
                    &conn,
                    access_token,
                    refresh_token,
                    expiry,
                    email,
                    is_gcp_tos,
                    project_id,
                    id_token,
                )?;
            } else {
                inject_legacy_agent_state(&conn, access_token, refresh_token, expiry, email)?;
            }
        }
        Err(_) => {
            let new_result = inject_unified_oauth_state(
                &conn,
                access_token,
                refresh_token,
                expiry,
                email,
                is_gcp_tos,
                project_id,
                id_token,
            );
            let old_result =
                inject_legacy_agent_state(&conn, access_token, refresh_token, expiry, email);

            if new_result.is_err() && old_result.is_err() {
                return Err(format!(
                    "Both injection formats failed - New: {:?}, Old: {:?}",
                    new_result.err(),
                    old_result.err()
                ));
            }
        }
    }

    write_onboarding_flag(&conn)?;

    Ok(format!("Token injected successfully to {:?}", db_path))
}

fn inject_legacy_agent_state(
    conn: &Connection,
    access_token: &str,
    refresh_token: &str,
    expiry: i64,
    email: &str,
) -> Result<(), String> {
    if !has_item(conn, LEGACY_AGENT_STATE_KEY)? {
        return Err("Legacy auth state key does not exist".to_string());
    }

    // 读取当前数据
    let current_data: String = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?",
            [LEGACY_AGENT_STATE_KEY],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to read data: {}", e))?;

    // Base64 解码
    let blob = general_purpose::STANDARD
        .decode(&current_data)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    // 移除旧 UserID/Email/OAuth 字段，重新写入当前账号。
    let clean_data = protobuf::remove_protobuf_field(&blob, 1)?;
    let clean_data = protobuf::remove_protobuf_field(&clean_data, 2)?;
    let clean_data = protobuf::remove_protobuf_field(&clean_data, 6)?;

    let email_field = protobuf::create_email_field(email);
    let new_field = protobuf::create_oauth_field(access_token, refresh_token, expiry);

    // 合并数据
    let final_data = [clean_data, email_field, new_field].concat();
    let final_b64 = general_purpose::STANDARD.encode(&final_data);

    // 写入数据库
    conn.execute(
        "UPDATE ItemTable SET value = ? WHERE key = ?",
        [&final_b64, LEGACY_AGENT_STATE_KEY],
    )
    .map_err(|e| format!("Failed to write data: {}", e))?;

    Ok(())
}

fn inject_unified_oauth_state(
    conn: &Connection,
    access_token: &str,
    refresh_token: &str,
    expiry: i64,
    email: &str,
    is_gcp_tos: bool,
    project_id: Option<&str>,
    id_token: Option<&str>,
) -> Result<(), String> {
    let oauth_info = protobuf::create_oauth_info(
        access_token,
        refresh_token,
        expiry,
        is_gcp_tos,
        id_token,
        Some(email),
    );
    let encoded_state = protobuf::create_unified_state_entry("oauthTokenInfoSentinelKey", &oauth_info);

    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?, ?)",
        [UNIFIED_OAUTH_STATE_KEY, &encoded_state],
    )
    .map_err(|e| format!("Failed to write unified auth state: {}", e))?;

    inject_user_status(conn, email)?;

    if let Some(project_id) = project_id.map(str::trim).filter(|value| !value.is_empty()) {
        inject_enterprise_project_preference(conn, project_id)?;
    } else {
        clear_enterprise_project_preference(conn)?;
    }

    Ok(())
}

fn inject_user_status(conn: &Connection, email: &str) -> Result<(), String> {
    let payload = protobuf::create_minimal_user_status_payload(email);
    let encoded_state = protobuf::create_unified_state_entry("userStatusSentinelKey", &payload);

    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?, ?)",
        [UNIFIED_USER_STATUS_KEY, &encoded_state],
    )
    .map_err(|e| format!("Failed to write user status: {}", e))?;

    Ok(())
}

fn inject_enterprise_project_preference(conn: &Connection, project_id: &str) -> Result<(), String> {
    let payload = protobuf::create_string_value_payload(project_id);
    let encoded_state = protobuf::create_unified_state_entry("enterpriseGcpProjectId", &payload);

    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?, ?)",
        [UNIFIED_ENTERPRISE_PREFS_KEY, &encoded_state],
    )
    .map_err(|e| format!("Failed to write enterprise preferences: {}", e))?;

    Ok(())
}

fn clear_enterprise_project_preference(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "DELETE FROM ItemTable WHERE key = ?",
        [UNIFIED_ENTERPRISE_PREFS_KEY],
    )
    .map_err(|e| format!("Failed to clear enterprise preferences: {}", e))?;

    Ok(())
}

fn write_onboarding_flag(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?, ?)",
        [ONBOARDING_KEY, "true"],
    )
    .map_err(|e| format!("Failed to write onboarding flag: {}", e))?;

    Ok(())
}

pub fn write_service_machine_id(db_path: &Path, service_machine_id: &str) -> Result<(), String> {
    let conn = Connection::open(db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    conn.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?, ?)",
        [SERVICE_MACHINE_ID_KEY, service_machine_id],
    )
    .map_err(|e| format!("Failed to write serviceMachineId: {}", e))?;

    Ok(())
}

fn has_item(conn: &Connection, key: &str) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM ItemTable WHERE key = ?",
            [key],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to check auth state key '{}': {}", key, e))?;
    Ok(count > 0)
}

/// 检查数据库是否存在
pub fn check_db_exists() -> bool {
    match get_db_path() {
        Ok(path) => path.exists(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose;
    use tempfile::tempdir;

    #[test]
    fn inject_token_supports_unified_oauth_state_without_legacy_key() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("state.vscdb");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute(
            "CREATE TABLE ItemTable (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO ItemTable (key, value) VALUES (?, ?)",
            ["antigravityUnifiedStateSync.oauthToken", ""],
        )
        .unwrap();
        drop(conn);

        let result = inject_token(
            &db_path,
            "access-token-new",
            "refresh-token-new",
            1_778_468_407,
            "user@example.com",
            false,
            Some("project-id"),
            Some("id-token"),
            None,
            None,
        );

        assert!(result.is_ok(), "{result:?}");
        let conn = Connection::open(&db_path).unwrap();
        let encoded: String = conn
            .query_row(
                "SELECT value FROM ItemTable WHERE key = ?",
                ["antigravityUnifiedStateSync.oauthToken"],
                |row| row.get(0),
            )
            .unwrap();
        let decoded = general_purpose::STANDARD.decode(encoded).unwrap();
        let decoded_text = String::from_utf8_lossy(&decoded);
        assert!(decoded_text.contains("oauthTokenInfoSentinelKey"));

        let user_status: String = conn
            .query_row(
                "SELECT value FROM ItemTable WHERE key = ?",
                ["antigravityUnifiedStateSync.userStatus"],
                |row| row.get(0),
            )
            .unwrap();
        let decoded_user_status = general_purpose::STANDARD.decode(user_status).unwrap();
        assert!(String::from_utf8_lossy(&decoded_user_status).contains("userStatusSentinelKey"));

        let enterprise_prefs: String = conn
            .query_row(
                "SELECT value FROM ItemTable WHERE key = ?",
                ["antigravityUnifiedStateSync.enterprisePreferences"],
                |row| row.get(0),
            )
            .unwrap();
        let decoded_enterprise_prefs = general_purpose::STANDARD.decode(enterprise_prefs).unwrap();
        assert!(String::from_utf8_lossy(&decoded_enterprise_prefs).contains("enterpriseGcpProjectId"));

        let onboarding: String = conn
            .query_row(
                "SELECT value FROM ItemTable WHERE key = ?",
                ["antigravityOnboarding"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(onboarding, "true");
    }
}
