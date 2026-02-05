//! Cursor state.vscdb 数据库操作模块

use rusqlite::Connection;
use std::path::PathBuf;

/// 获取 Cursor 数据库路径（跨平台）
pub fn get_db_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join("Library/Application Support/Cursor/User/globalStorage/state.vscdb"))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Cannot get APPDATA environment variable".to_string())?;
        Ok(PathBuf::from(appdata).join("Cursor\\User\\globalStorage\\state.vscdb"))
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join(".config/Cursor/User/globalStorage/state.vscdb"))
    }
}

/// 获取 Cursor storage.json 路径
pub fn get_storage_json_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join("Library/Application Support/Cursor/User/globalStorage/storage.json"))
    }

    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Cannot get APPDATA environment variable".to_string())?;
        Ok(PathBuf::from(appdata).join("Cursor\\User\\globalStorage\\storage.json"))
    }

    #[cfg(target_os = "linux")]
    {
        let home = dirs::home_dir().ok_or("Cannot get home directory")?;
        Ok(home.join(".config/Cursor/User/globalStorage/storage.json"))
    }
}

/// 检查数据库是否存在
pub fn check_db_exists() -> bool {
    match get_db_path() {
        Ok(path) => path.exists(),
        Err(_) => false,
    }
}

fn write_item(tx: &rusqlite::Transaction<'_>, key: &str, value: &str) -> Result<(), String> {
    tx.execute(
        "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?, ?)",
        [key, value],
    )
    .map_err(|e| format!("Failed to write key {}: {}", key, e))?;
    Ok(())
}

/// 写入 Cursor 认证状态到数据库
/// Cursor 使用 cursorAuth/accessToken 和 cursorAuth/refreshToken 存储认证信息
pub fn write_cursor_auth_state(
    db_path: &PathBuf,
    access_token: &str,
    refresh_token: &str,
    email: &str,
    _user_id: &str,
) -> Result<(), String> {
    println!("Writing auth state to database: {}", db_path.display());

    let mut conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // 清理旧的认证状态
    tx.execute("DELETE FROM ItemTable WHERE key LIKE 'cursorAuth/%'", [])
        .map_err(|e| format!("Failed to clear cursorAuth entries: {}", e))?;
    println!("Cleared old cursorAuth entries");

    // 写入新的认证信息
    // Cursor 存储格式: cursorAuth/accessToken, cursorAuth/refreshToken
    write_item(&tx, "cursorAuth/accessToken", access_token)?;
    write_item(&tx, "cursorAuth/refreshToken", refresh_token)?;
    println!("Written access_token and refresh_token");

    // 写入邮箱缓存
    write_item(&tx, "cursorAuth/cachedEmail", email)?;
    println!("Written user info and email: {}", email);

    tx.commit()
        .map_err(|e| format!("Failed to commit auth updates: {}", e))?;
    println!("Auth state written successfully for: {}", email);
    Ok(())
}

/// 重置 state.vscdb 中的机器标识符
/// - storage.serviceMachineId: 如果提供了 service_machine_id 则使用，否则生成新的 UUID
/// - workbench.experiments.statsigBootstrap.customIDs.stableID: 使用 machine_id (64字符十六进制)
pub fn reset_machine_ids_in_db(machine_id: &str, service_machine_id: Option<&str>) -> Result<(), String> {
    let db_path = get_db_path()?;

    if !db_path.exists() {
        // 数据库不存在，静默返回（Cursor 可能未运行过）
        return Ok(());
    }

    println!("Resetting machine IDs in database: {}", db_path.display());

    let mut conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // 1. 重置 storage.serviceMachineId (使用提供的值或生成新 UUID)
    let new_service_id = if let Some(service_id) = service_machine_id {
        service_id.to_string()
    } else {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    };
    write_item(&tx, "storage.serviceMachineId", &new_service_id)?;
    println!("Written storage.serviceMachineId: {}", new_service_id);

    // 2. 重置 statsigBootstrap.customIDs.stableID (使用与 machineId 相同的值)
    // 先读取现有的 statsigBootstrap 数据
    match tx.query_row(
        "SELECT value FROM ItemTable WHERE key = 'workbench.experiments.statsigBootstrap'",
        [],
        |row| row.get::<_, String>(0),
    ) {
        Ok(existing_value) => {
            // 解析并更新 JSON
            if let Ok(mut statsig_data) = serde_json::from_str::<serde_json::Value>(&existing_value) {
                if let Some(user_obj) = statsig_data.get_mut("user") {
                    if let Some(custom_ids) = user_obj.get_mut("customIDs") {
                        if let Some(id_map) = custom_ids.as_object_mut() {
                            id_map.insert("stableID".to_string(), serde_json::Value::String(machine_id.to_string()));
                        }
                    } else if let Some(user_obj) = statsig_data["user"].as_object_mut() {
                        // customIDs 不存在，创建它
                        user_obj.insert("customIDs".to_string(), serde_json::json!({
                            "stableID": machine_id
                        }));
                    }
                }
                // 写回更新后的 JSON
                let updated_json = serde_json::to_string(&statsig_data)
                    .map_err(|e| format!("Failed to serialize statsigBootstrap: {}", e))?;
                tx.execute(
                    "UPDATE ItemTable SET value = ? WHERE key = 'workbench.experiments.statsigBootstrap'",
                    [&updated_json],
                ).map_err(|e| format!("Failed to update statsigBootstrap: {}", e))?;
                println!("Written statsigBootstrap.customIDs.stableID: {}", machine_id);
            }
        }
        Err(_) => {
            // statsigBootstrap 不存在，创建新的
            let new_statsig_data = serde_json::json!({
                "user": {
                    "customIDs": {
                        "stableID": machine_id
                    }
                }
            });
            let new_statsig_json = serde_json::to_string(&new_statsig_data)
                .map_err(|e| format!("Failed to serialize statsigBootstrap: {}", e))?;
            tx.execute(
                "INSERT OR REPLACE INTO ItemTable (key, value) VALUES ('workbench.experiments.statsigBootstrap', ?)",
                [&new_statsig_json],
            ).map_err(|e| format!("Failed to insert statsigBootstrap: {}", e))?;
            println!("Created new statsigBootstrap.customIDs.stableID: {}", machine_id);
        }
    }

    tx.commit()
        .map_err(|e| format!("Failed to commit machine ID updates: {}", e))?;
    println!("Machine IDs in database reset successfully");
    Ok(())
}
