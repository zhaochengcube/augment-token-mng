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
