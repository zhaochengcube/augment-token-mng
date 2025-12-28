pub mod connection;
pub mod config;
pub mod migrations;

pub use config::*;
pub use connection::*;
pub use migrations::*;

use std::sync::Arc;
use tauri::{AppHandle, State};
use crate::AppState;
use crate::storage::initialize_storage_manager;

// 数据库配置相关命令
#[tauri::command]
pub async fn save_database_config(
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    ssl_mode: Option<String>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config_manager = DatabaseConfigManager::new(&app)
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    let ssl_mode = match ssl_mode.as_deref() {
        Some("disable") => SslMode::Disable,
        Some("require") => SslMode::Require,
        _ => SslMode::Prefer,
    };

    let config = DatabaseConfig::new_with_ssl(host, port, database, username, password, ssl_mode);

    config_manager.save_config(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    let mut db_manager = DatabaseManager::new(config);
    match db_manager.initialize().await {
        Ok(_) => {
            if let Some(pool) = db_manager.get_pool() {
                let client = pool.get().await
                    .map_err(|e| format!("Failed to get database client: {}", e))?;

                let tables_exist = check_tables_exist(&client).await
                    .map_err(|e| format!("Failed to check tables: {}", e))?;

                if !tables_exist {
                    create_tables(&client).await
                        .map_err(|e| format!("Failed to create tables: {}", e))?;
                } else {
                    println!("Database tables already exist, checking for new fields");
                    add_new_fields_if_not_exist(&client).await
                        .map_err(|e| format!("Failed to add new fields: {}", e))?;

                    remove_updated_at_trigger(&client).await
                        .map_err(|e| format!("Failed to remove trigger: {}", e))?;
                }
            }

            *state.database_manager.lock().unwrap() = Some(Arc::new(db_manager));

            initialize_storage_manager(&app, &state).await
                .map_err(|e| format!("Failed to initialize storage: {}", e))?;

            Ok(())
        }
        Err(e) => Err(format!("Failed to connect to database: {}", e))
    }
}

#[tauri::command]
pub async fn load_database_config(
    app: AppHandle,
) -> Result<DatabaseConfig, String> {
    let config_manager = DatabaseConfigManager::new(&app)
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    config_manager.load_config()
        .map_err(|e| format!("Failed to load config: {}", e))
}

#[tauri::command(rename = "test_database_connection")]
pub async fn test_database_connection_cmd(
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    ssl_mode: Option<String>,
) -> Result<(), String> {
    let ssl_mode = match ssl_mode.as_deref() {
        Some("disable") => SslMode::Disable,
        Some("require") => SslMode::Require,
        _ => SslMode::Prefer,
    };

    let config = DatabaseConfig::new_with_ssl(host, port, database, username, password, ssl_mode);

    connection::test_database_connection(&config).await
        .map_err(|e| format!("Connection test failed: {}", e))
}

#[tauri::command]
pub async fn delete_database_config(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config_manager = DatabaseConfigManager::new(&app)
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    config_manager.delete_config()
        .map_err(|e| format!("Failed to delete config: {}", e))?;

    *state.database_manager.lock().unwrap() = None;

    initialize_storage_manager(&app, &state).await
        .map_err(|e| format!("Failed to reinitialize storage: {}", e))?;

    Ok(())
}
