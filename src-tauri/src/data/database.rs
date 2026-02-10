pub mod antigravity;
pub mod augment;
pub mod claude;
pub mod config;
pub mod connection;
pub mod cursor;
pub mod openai;
pub mod windsurf;

pub use config::*;
pub use connection::*;

use crate::AppState;
use crate::storage::{
    initialize_antigravity_storage_manager, initialize_claude_storage_manager,
    initialize_cursor_storage_manager, initialize_openai_storage_manager,
    initialize_storage_manager,
};
use std::sync::Arc;
use tauri::{AppHandle, State};

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

    config_manager
        .save_config(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    let mut db_manager = DatabaseManager::new(config);
    match db_manager.initialize().await {
        Ok(_) => {
            if let Some(pool) = db_manager.get_pool() {
                let client = pool
                    .get()
                    .await
                    .map_err(|e| format!("Failed to get database client: {}", e))?;

                let augment_tables_exist = augment::check_tables_exist(&client)
                    .await
                    .map_err(|e| format!("Failed to check Augment tables: {}", e))?;

                if !augment_tables_exist {
                    augment::create_tables(&client)
                        .await
                        .map_err(|e| format!("Failed to create Augment tables: {}", e))?;
                } else {
                    println!("Augment tables already exist, checking for new fields");
                    augment::add_new_fields_if_not_exist(&client)
                        .await
                        .map_err(|e| {
                            format!("Failed to add new fields to Augment tables: {}", e)
                        })?;

                    augment::remove_updated_at_trigger(&client)
                        .await
                        .map_err(|e| format!("Failed to remove Augment trigger: {}", e))?;
                }

                let antigravity_tables_exist = antigravity::check_tables_exist(&client)
                    .await
                    .map_err(|e| format!("Failed to check Antigravity tables: {}", e))?;

                if !antigravity_tables_exist {
                    antigravity::create_tables(&client)
                        .await
                        .map_err(|e| format!("Failed to create Antigravity tables: {}", e))?;
                } else {
                    antigravity::add_new_fields_if_not_exist(&client)
                        .await
                        .map_err(|e| {
                            format!("Failed to add new fields to Antigravity tables: {}", e)
                        })?;
                }

                let windsurf_tables_exist = windsurf::check_tables_exist(&client)
                    .await
                    .map_err(|e| format!("Failed to check Windsurf tables: {}", e))?;

                if !windsurf_tables_exist {
                    windsurf::create_tables(&client)
                        .await
                        .map_err(|e| format!("Failed to create Windsurf tables: {}", e))?;
                } else {
                    windsurf::add_new_fields_if_not_exist(&client)
                        .await
                        .map_err(|e| format!("Failed to update Windsurf tables: {}", e))?;
                }

                let cursor_tables_exist = cursor::check_tables_exist(&client)
                    .await
                    .map_err(|e| format!("Failed to check Cursor tables: {}", e))?;

                if !cursor_tables_exist {
                    cursor::create_tables(&client)
                        .await
                        .map_err(|e| format!("Failed to create Cursor tables: {}", e))?;
                } else {
                    cursor::add_new_fields_if_not_exist(&client)
                        .await
                        .map_err(|e| format!("Failed to update Cursor tables: {}", e))?;
                }

                let openai_tables_exist = openai::check_tables_exist(&client)
                    .await
                    .map_err(|e| format!("Failed to check OpenAI tables: {}", e))?;

                if !openai_tables_exist {
                    openai::create_tables(&client)
                        .await
                        .map_err(|e| format!("Failed to create OpenAI tables: {}", e))?;
                } else {
                    openai::add_new_fields_if_not_exist(&client)
                        .await
                        .map_err(|e| format!("Failed to update OpenAI tables: {}", e))?;
                }

                let claude_tables_exist = claude::check_tables_exist(&client)
                    .await
                    .map_err(|e| format!("Failed to check Claude tables: {}", e))?;

                if !claude_tables_exist {
                    claude::create_tables(&client)
                        .await
                        .map_err(|e| format!("Failed to create Claude tables: {}", e))?;
                } else {
                    claude::add_new_fields_if_not_exist(&client)
                        .await
                        .map_err(|e| format!("Failed to update Claude tables: {}", e))?;
                }
            }

            *state.database_manager.lock().unwrap() = Some(Arc::new(db_manager));

            initialize_storage_manager(&app, &state)
                .await
                .map_err(|e| format!("Failed to initialize storage: {}", e))?;
            initialize_antigravity_storage_manager(&app, &state)
                .await
                .map_err(|e| format!("Failed to initialize Antigravity storage: {}", e))?;
            initialize_cursor_storage_manager(&app, &state)
                .await
                .map_err(|e| format!("Failed to initialize Cursor storage: {}", e))?;
            initialize_openai_storage_manager(&app, &state)
                .await
                .map_err(|e| format!("Failed to initialize OpenAI storage: {}", e))?;
            initialize_claude_storage_manager(&app, &state)
                .await
                .map_err(|e| format!("Failed to initialize Claude storage: {}", e))?;

            Ok(())
        }
        Err(e) => Err(format!("Failed to connect to database: {}", e)),
    }
}

#[tauri::command]
pub async fn load_database_config(app: AppHandle) -> Result<DatabaseConfig, String> {
    let config_manager = DatabaseConfigManager::new(&app)
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    config_manager
        .load_config()
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

    connection::test_database_connection(&config)
        .await
        .map_err(|e| format!("Connection test failed: {}", e))
}

#[tauri::command]
pub async fn delete_database_config(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config_manager = DatabaseConfigManager::new(&app)
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    config_manager
        .delete_config()
        .map_err(|e| format!("Failed to delete config: {}", e))?;

    *state.database_manager.lock().unwrap() = None;

    initialize_storage_manager(&app, &state)
        .await
        .map_err(|e| format!("Failed to reinitialize storage: {}", e))?;
    initialize_antigravity_storage_manager(&app, &state)
        .await
        .map_err(|e| format!("Failed to reinitialize Antigravity storage: {}", e))?;
    initialize_openai_storage_manager(&app, &state)
        .await
        .map_err(|e| format!("Failed to reinitialize OpenAI storage: {}", e))?;
    initialize_claude_storage_manager(&app, &state)
        .await
        .map_err(|e| format!("Failed to reinitialize Claude storage: {}", e))?;

    Ok(())
}
