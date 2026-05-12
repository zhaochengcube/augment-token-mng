use crate::antigravity::utils::protobuf;
use base64::{Engine as _, engine::general_purpose};
use rusqlite::Connection;
use std::path::PathBuf;

const LEGACY_AGENT_STATE_KEY: &str = "jetskiStateSync.agentManagerInitState";
const UNIFIED_OAUTH_STATE_KEY: &str = "antigravityUnifiedStateSync.oauthToken";
const ONBOARDING_KEY: &str = "antigravityOnboarding";

/// 获取 Antigravity 数据库路径（跨平台）
pub fn get_db_path() -> Result<PathBuf, String> {
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
) -> Result<String, String> {
    // 1. 打开数据库
    let conn = Connection::open(db_path).map_err(|e| format!("Failed to open database: {}", e))?;

    if has_item(&conn, LEGACY_AGENT_STATE_KEY)? {
        inject_legacy_agent_state(&conn, access_token, refresh_token, expiry)?;
    } else if has_item(&conn, UNIFIED_OAUTH_STATE_KEY)? {
        inject_unified_oauth_state(&conn, access_token, refresh_token, expiry)?;
    } else {
        return Err(format!(
            "Failed to locate Antigravity auth state. Missing both '{}' and '{}'",
            LEGACY_AGENT_STATE_KEY, UNIFIED_OAUTH_STATE_KEY
        ));
    }

    write_onboarding_flag(&conn)?;

    Ok(format!("Token injected successfully to {:?}", db_path))
}

fn inject_legacy_agent_state(
    conn: &Connection,
    access_token: &str,
    refresh_token: &str,
    expiry: i64,
) -> Result<(), String> {
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

    // 移除旧 Field 6
    let clean_data = protobuf::remove_protobuf_field(&blob, 6)?;

    // 创建新 Field 6
    let new_field = protobuf::create_oauth_field(access_token, refresh_token, expiry);

    // 合并数据
    let final_data = [clean_data, new_field].concat();
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
) -> Result<(), String> {
    let state = protobuf::create_unified_oauth_state(access_token, refresh_token, expiry);
    let encoded_state = general_purpose::STANDARD.encode(state);

    conn.execute(
        "UPDATE ItemTable SET value = ? WHERE key = ?",
        [&encoded_state, UNIFIED_OAUTH_STATE_KEY],
    )
    .map_err(|e| format!("Failed to write unified auth state: {}", e))?;

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
        assert!(decoded_text.contains("authStateWithContextSentinelKey"));
        assert!(decoded_text.contains("oauthTokenInfoSentinelKey"));
        assert!(decoded_text.contains(&general_purpose::STANDARD.encode(
            protobuf::create_oauth_token_info(
                "access-token-new",
                "refresh-token-new",
                1_778_468_407,
            )
        )));

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
