pub mod traits;
pub mod local_storage;
pub mod postgres_storage;
pub mod dual_storage;

pub use traits::*;
pub use local_storage::*;
pub use postgres_storage::*;
pub use dual_storage::*;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Manager, State};
use crate::AppState;

#[tauri::command]
pub async fn save_tokens_json(json_string: String, app: tauri::AppHandle) -> Result<(), String> {
    use std::fs;
    use std::io::Write;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    let storage_path = app_data_dir.join("tokens.json");
    let temp_path = storage_path.with_extension("tmp");

    serde_json::from_str::<serde_json::Value>(&json_string)
        .map_err(|e| format!("Invalid JSON format: {}", e))?;

    {
        let mut temp_file = fs::File::create(&temp_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;

        temp_file.write_all(json_string.as_bytes())
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        temp_file.sync_all()
            .map_err(|e| format!("Failed to sync temp file: {}", e))?;
    }

    // 尝试重命名,如果失败则清理临时文件并重试
    match fs::rename(&temp_path, &storage_path) {
        Ok(_) => Ok(()),
        Err(e) => {
            // 清理临时文件
            let _ = fs::remove_file(&temp_path);

            // 检查临时文件是否仍然存在
            if !temp_path.exists() {
                return Err(format!("Failed to rename temp file (temp file was removed): {}", e));
            }

            // 检查父目录是否仍然存在
            if !app_data_dir.exists() {
                return Err(format!("Failed to rename temp file (parent directory disappeared): {}", e));
            }

            // 在Windows上,如果目标文件被占用,尝试先删除再重命名
            #[cfg(target_os = "windows")]
            {
                if storage_path.exists() {
                    if let Err(remove_err) = fs::remove_file(&storage_path) {
                        let _ = fs::remove_file(&temp_path);
                        return Err(format!("Failed to remove existing file before rename: {}", remove_err));
                    }
                }

                // 重新尝试重命名
                match fs::rename(&temp_path, &storage_path) {
                    Ok(_) => return Ok(()),
                    Err(retry_err) => {
                        let _ = fs::remove_file(&temp_path);
                        return Err(format!("Failed to rename temp file after retry: {}", retry_err));
                    }
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                Err(format!("Failed to rename temp file: {}", e))
            }
        }
    }
}

#[tauri::command]
pub async fn load_tokens_json(app: tauri::AppHandle) -> Result<String, String> {
    use std::fs;

    let new_app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let new_storage_path = new_app_data_dir.join("tokens.json");

    println!("尝试读取新文件路径: {:?}", new_storage_path);

    if new_storage_path.exists() {
        let content = fs::read_to_string(&new_storage_path)
            .map_err(|e| format!("Failed to read tokens file: {}", e))?;

        if content.trim().is_empty() {
            return Ok("[]".to_string());
        }

        return process_token_content(content);
    }

    println!("新目录中没有文件，尝试从旧目录读取...");

    let old_app_data_dir = get_old_app_data_dir()?;
    let old_storage_path = old_app_data_dir.join("tokens.json");

    println!("尝试读取旧文件路径: {:?}", old_storage_path);

    if old_storage_path.exists() {
        let content = fs::read_to_string(&old_storage_path)
            .map_err(|e| format!("Failed to read old tokens file: {}", e))?;

        if content.trim().is_empty() {
            return Ok("[]".to_string());
        }

        if let Some(parent) = new_storage_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create new app data directory: {}", e))?;
        }

        fs::copy(&old_storage_path, &new_storage_path)
            .map_err(|e| format!("Failed to migrate tokens file: {}", e))?;

        println!("文件已迁移到新目录: {:?}", new_storage_path);

        return process_token_content(content);
    }

    println!("新旧目录都没有找到 tokens.json 文件");
    Ok("[]".to_string())
}

fn get_old_app_data_dir() -> Result<PathBuf, String> {
    use std::env;

    let home_dir = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map_err(|_| "Failed to get home directory".to_string())?;

    let old_path = if cfg!(target_os = "windows") {
        PathBuf::from(home_dir)
            .join("AppData")
            .join("Roaming")
            .join("com.capslockCube.augment-token-manager")
    } else if cfg!(target_os = "macos") {
        PathBuf::from(home_dir)
            .join("Library")
            .join("Application Support")
            .join("com.capslockCube.augment-token-manager")
    } else {
        PathBuf::from(home_dir)
            .join(".config")
            .join("com.capslockCube.augment-token-manager")
    };

    Ok(old_path)
}

fn process_token_content(content: String) -> Result<String, String> {
    match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(value) => match value {
            serde_json::Value::Array(_) => Ok(content),
            serde_json::Value::Object(ref obj) => {
                if let Some(tokens_array) = obj.get("tokens") {
                    if tokens_array.is_array() {
                        Ok(serde_json::to_string_pretty(tokens_array)
                            .map_err(|e| format!("Failed to serialize tokens: {}", e))?)
                    } else {
                        Ok("[]".to_string())
                    }
                } else {
                    let array = serde_json::Value::Array(vec![value]);
                    Ok(serde_json::to_string_pretty(&array)
                        .map_err(|e| format!("Failed to serialize tokens: {}", e))?)
                }
            }
            _ => Ok("[]".to_string()),
        },
        Err(_) => Ok("[]".to_string()),
    }
}

// 同步相关命令
#[tauri::command]
pub async fn sync_tokens_to_database(
    state: State<'_, AppState>,
) -> Result<SyncStatus, String> {
    let storage_manager = {
        let guard = state.storage_manager.lock().unwrap();
        guard.clone().ok_or("Storage manager not initialized")?
    };

    storage_manager.sync_local_to_remote().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn sync_tokens_from_database(
    state: State<'_, AppState>,
) -> Result<SyncStatus, String> {
    let storage_manager = {
        let guard = state.storage_manager.lock().unwrap();
        guard.clone().ok_or("Storage manager not initialized")?
    };

    storage_manager.sync_remote_to_local().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn delete_token(
    token_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let storage_manager = {
        let guard = state.storage_manager.lock().unwrap();
        guard.clone().ok_or("Storage manager not initialized")?
    };

    storage_manager.delete_token(&token_id).await
        .map_err(|e| format!("Delete failed: {}", e))
}

#[tauri::command]
pub async fn bidirectional_sync_tokens(
    state: State<'_, AppState>,
) -> Result<SyncStatus, String> {
    let storage_manager = {
        let guard = state.storage_manager.lock().unwrap();
        guard.clone().ok_or("Storage manager not initialized")?
    };

    storage_manager.bidirectional_sync().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn bidirectional_sync_tokens_with_data(
    tokens_json: String,
    state: State<'_, AppState>,
) -> Result<SyncStatus, String> {
    let storage_manager = {
        let guard = state.storage_manager.lock().unwrap();
        guard.clone().ok_or("Storage manager not initialized")?
    };

    let tokens: Vec<TokenData> = serde_json::from_str(&tokens_json)
        .map_err(|e| format!("Failed to parse tokens JSON: {}", e))?;

    storage_manager.bidirectional_sync_with_tokens(tokens).await
        .map_err(|e| format!("Sync failed: {}", e))
}

/// 新的基于 version + tombstone 的增量同步命令
#[tauri::command]
pub async fn sync_tokens(
    req_json: String,
    state: State<'_, AppState>,
) -> Result<ServerSyncResponse, String> {
    let storage_manager = {
        let guard = state.storage_manager.lock().unwrap();
        guard.clone().ok_or("Storage manager not initialized")?
    };

    let req: ClientSyncRequest = serde_json::from_str(&req_json)
        .map_err(|e| format!("Failed to parse sync request: {}", e))?;

    storage_manager.sync_tokens(req).await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn get_storage_status(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let storage_manager = {
        let guard = state.storage_manager.lock().unwrap();
        guard.clone()
    };

    if storage_manager.is_none() {
        return Ok(serde_json::json!({
            "is_available": false,
            "storage_type": "initializing",
            "is_database_available": false,
            "is_initializing": true
        }));
    }

    let storage_manager = storage_manager.unwrap();
    let is_available = storage_manager.is_available().await;
    let storage_type = storage_manager.storage_type();
    let is_database_available = storage_manager.is_database_available();

    Ok(serde_json::json!({
        "is_available": is_available,
        "storage_type": storage_type,
        "is_database_available": is_database_available,
        "is_initializing": false
    }))
}

#[tauri::command]
pub async fn get_sync_status(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<SyncStatus>, String> {
    let storage_manager = {
        let manager_option = {
            let guard = state.storage_manager.lock().unwrap();
            guard.clone()
        };

        if let Some(manager) = manager_option {
            manager
        } else {
            if let Err(e) = initialize_storage_manager(&app, &state).await {
                return Err(format!("Failed to initialize storage manager: {}", e));
            }
            let guard = state.storage_manager.lock().unwrap();
            guard.clone().ok_or("Storage manager still not initialized after initialization attempt")?
        }
    };

    storage_manager.get_sync_status().await
        .map_err(|e| format!("Failed to get sync status: {}", e))
}

// 辅助函数：初始化存储管理器
pub async fn initialize_storage_manager(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let local_storage = Arc::new(LocalFileStorage::new(app)?);

    let postgres_storage = {
        let db_manager_guard = state.database_manager.lock().unwrap();
        if let Some(db_manager) = db_manager_guard.as_ref() {
            Some(Arc::new(PostgreSQLStorage::new(db_manager.clone())))
        } else {
            None
        }
    };

    let dual_storage = Arc::new(DualStorage::new(local_storage, postgres_storage));

    *state.storage_manager.lock().unwrap() = Some(dual_storage);

    Ok(())
}
