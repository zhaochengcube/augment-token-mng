// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod augment_oauth;
mod bookmarks;
mod http_server;
mod outlook_manager;
mod database;
mod storage;

use augment_oauth::{create_augment_oauth_state, generate_augment_authorize_url, complete_augment_oauth_flow, check_account_ban_status, AugmentOAuthState, AugmentTokenResponse, AccountStatus};
use bookmarks::{BookmarkManager, Bookmark};
use http_server::HttpServer;
use outlook_manager::{OutlookManager, OutlookCredentials, EmailListResponse, EmailDetailsResponse, AccountStatus as OutlookAccountStatus};
use database::{DatabaseConfig, DatabaseConfigManager, DatabaseManager};
use storage::{DualStorage, LocalFileStorage, PostgreSQLStorage, TokenStorage, SyncManager};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use tauri::{State, Manager, WebviewWindowBuilder, WebviewUrl};
use chrono;

// Global state to store OAuth state and storage managers
struct AppState {
    augment_oauth_state: Mutex<Option<AugmentOAuthState>>,
    http_server: Mutex<Option<HttpServer>>,
    outlook_manager: Mutex<OutlookManager>,
    storage_manager: Arc<Mutex<Option<Arc<DualStorage>>>>,
    database_manager: Arc<Mutex<Option<Arc<DatabaseManager>>>>,
}

#[tauri::command]
async fn generate_auth_url(state: State<'_, AppState>) -> Result<String, String> {
    let augment_oauth_state = create_augment_oauth_state();
    let auth_url = generate_augment_authorize_url(&augment_oauth_state)
        .map_err(|e| format!("Failed to generate auth URL: {}", e))?;
    
    // Store the Augment OAuth state
    *state.augment_oauth_state.lock().unwrap() = Some(augment_oauth_state);
    
    Ok(auth_url)
}

#[tauri::command]
async fn generate_augment_auth_url(state: State<'_, AppState>) -> Result<String, String> {
    let augment_oauth_state = create_augment_oauth_state();
    let auth_url = generate_augment_authorize_url(&augment_oauth_state)
        .map_err(|e| format!("Failed to generate Augment auth URL: {}", e))?;
    
    // Store the Augment OAuth state
    *state.augment_oauth_state.lock().unwrap() = Some(augment_oauth_state);
    
    Ok(auth_url)
}



#[tauri::command]
async fn get_token(code: String, state: State<'_, AppState>) -> Result<AugmentTokenResponse, String> {
    let augment_oauth_state = {
        let guard = state.augment_oauth_state.lock().unwrap();
        guard.clone()
            .ok_or("No Augment OAuth state found. Please generate auth URL first.")?
    };

    complete_augment_oauth_flow(&augment_oauth_state, &code)
        .await
        .map_err(|e| format!("Failed to complete OAuth flow: {}", e))
}

#[tauri::command]
async fn get_augment_token(code: String, state: State<'_, AppState>) -> Result<AugmentTokenResponse, String> {
    let augment_oauth_state = {
        let guard = state.augment_oauth_state.lock().unwrap();
        guard.clone()
            .ok_or("No Augment OAuth state found. Please generate auth URL first.")?
    };

    complete_augment_oauth_flow(&augment_oauth_state, &code)
        .await
        .map_err(|e| format!("Failed to complete Augment OAuth flow: {}", e))
}

#[tauri::command]
async fn check_account_status(token: String, tenant_url: String) -> Result<AccountStatus, String> {
    check_account_ban_status(&token, &tenant_url)
        .await
        .map_err(|e| format!("Failed to check account status: {}", e))
}

#[tauri::command]
async fn open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener().open_url(url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))
}

#[tauri::command]
async fn save_tokens_json(json_string: String, app: tauri::AppHandle) -> Result<(), String> {
    use std::fs;
    use std::io::Write;

    // 获取应用数据目录
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // 确保目录存在
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    let storage_path = app_data_dir.join("tokens.json");
    let temp_path = storage_path.with_extension("tmp");

    // 基本的 JSON 格式验证
    serde_json::from_str::<serde_json::Value>(&json_string)
        .map_err(|e| format!("Invalid JSON format: {}", e))?;

    // 原子性写入：先写临时文件，再重命名
    {
        let mut temp_file = fs::File::create(&temp_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;

        temp_file.write_all(json_string.as_bytes())
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        temp_file.sync_all()
            .map_err(|e| format!("Failed to sync temp file: {}", e))?;
    }

    // 原子性重命名
    fs::rename(&temp_path, &storage_path)
        .map_err(|e| format!("Failed to rename temp file: {}", e))?;

    Ok(())
}


#[tauri::command]
async fn load_tokens_json(app: tauri::AppHandle) -> Result<String, String> {
    use std::fs;

    // 获取新的应用数据目录
    let new_app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let new_storage_path = new_app_data_dir.join("tokens.json");

    println!("尝试读取新文件路径: {:?}", new_storage_path);

    // 首先尝试从新目录读取
    if new_storage_path.exists() {
        let content = fs::read_to_string(&new_storage_path)
            .map_err(|e| format!("Failed to read tokens file: {}", e))?;

        println!("从新目录读取到的文件内容: {}", content);

        // 如果文件为空，返回空数组的 JSON
        if content.trim().is_empty() {
            return Ok("[]".to_string());
        }

        return process_token_content(content);
    }

    // 如果新目录没有文件，尝试从旧目录读取
    println!("新目录中没有文件，尝试从旧目录读取...");

    // 构造旧的应用数据目录路径
    let old_app_data_dir = get_old_app_data_dir()?;
    let old_storage_path = old_app_data_dir.join("tokens.json");

    println!("尝试读取旧文件路径: {:?}", old_storage_path);

    if old_storage_path.exists() {
        let content = fs::read_to_string(&old_storage_path)
            .map_err(|e| format!("Failed to read old tokens file: {}", e))?;

        println!("从旧目录读取到的文件内容: {}", content);

        // 如果文件为空，返回空数组的 JSON
        if content.trim().is_empty() {
            return Ok("[]".to_string());
        }

        // 创建新目录（如果不存在）
        if let Some(parent) = new_storage_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create new app data directory: {}", e))?;
        }

        // 将文件迁移到新目录
        fs::copy(&old_storage_path, &new_storage_path)
            .map_err(|e| format!("Failed to migrate tokens file: {}", e))?;

        println!("文件已迁移到新目录: {:?}", new_storage_path);

        return process_token_content(content);
    }

    // 两个目录都没有文件
    println!("新旧目录都没有找到 tokens.json 文件");
    Ok("[]".to_string())
}

// 获取旧的应用数据目录
fn get_old_app_data_dir() -> Result<PathBuf, String> {
    use std::env;
    use std::path::PathBuf;

    let home_dir = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map_err(|_| "Failed to get home directory".to_string())?;

    // 旧的 identifier: com.capslockCube.augment-token-manager
    let old_path = if cfg!(target_os = "windows") {
        // Windows: %APPDATA%\com.capslockCube.augment-token-manager
        PathBuf::from(home_dir)
            .join("AppData")
            .join("Roaming")
            .join("com.capslockCube.augment-token-manager")
    } else if cfg!(target_os = "macos") {
        // macOS: ~/Library/Application Support/com.capslockCube.augment-token-manager
        PathBuf::from(home_dir)
            .join("Library")
            .join("Application Support")
            .join("com.capslockCube.augment-token-manager")
    } else {
        // Linux: ~/.config/com.capslockCube.augment-token-manager
        PathBuf::from(home_dir)
            .join(".config")
            .join("com.capslockCube.augment-token-manager")
    };

    Ok(old_path)
}

// 处理 token 内容的通用函数
fn process_token_content(content: String) -> Result<String, String> {
    // 尝试解析 JSON 内容
    match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(value) => {
            // 如果解析成功，检查是否需要转换格式
            match value {
                serde_json::Value::Array(_) => {
                    // 如果已经是数组格式，直接返回原内容
                    Ok(content)
                }
                serde_json::Value::Object(ref obj) => {
                    // 检查是否是旧格式 {tokens: [...]}
                    if let Some(tokens_array) = obj.get("tokens") {
                        if tokens_array.is_array() {
                            // 旧格式，提取 tokens 数组
                            Ok(serde_json::to_string_pretty(tokens_array)
                                .map_err(|e| format!("Failed to serialize tokens: {}", e))?)
                        } else {
                            Ok("[]".to_string())
                        }
                    } else {
                        // 如果是单个对象格式，包装成数组
                        let array = serde_json::Value::Array(vec![value]);
                        Ok(serde_json::to_string_pretty(&array)
                            .map_err(|e| format!("Failed to serialize tokens: {}", e))?)
                    }
                }
                _ => {
                    // 其他格式，返回空数组
                    Ok("[]".to_string())
                }
            }
        }
        Err(_) => {
            // 如果 JSON 解析失败，可能是其他格式的旧数据，返回空数组
            Ok("[]".to_string())
        }
    }
}



// Bookmark management commands
#[tauri::command]
async fn add_bookmark(
    name: String,
    url: String,
    description: Option<String>,
    category: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let bookmark_manager = BookmarkManager::new(&app)
        .map_err(|e| format!("Failed to initialize bookmark manager: {}", e))?;

    bookmark_manager.add_bookmark(name, url, description, category)
        .map_err(|e| format!("Failed to add bookmark: {}", e))
}

#[tauri::command]
async fn update_bookmark(
    id: String,
    name: String,
    url: String,
    description: Option<String>,
    app: tauri::AppHandle,
) -> Result<bool, String> {
    let bookmark_manager = BookmarkManager::new(&app)
        .map_err(|e| format!("Failed to initialize bookmark manager: {}", e))?;

    bookmark_manager.update_bookmark(&id, name, url, description)
        .map_err(|e| format!("Failed to update bookmark: {}", e))
}

#[tauri::command]
async fn delete_bookmark(
    id: String,
    app: tauri::AppHandle,
) -> Result<bool, String> {
    let bookmark_manager = BookmarkManager::new(&app)
        .map_err(|e| format!("Failed to initialize bookmark manager: {}", e))?;

    bookmark_manager.remove_bookmark(&id)
        .map_err(|e| format!("Failed to delete bookmark: {}", e))
}

#[tauri::command]
async fn get_bookmarks(
    category: String,
    app: tauri::AppHandle,
) -> Result<Vec<Bookmark>, String> {
    let bookmark_manager = BookmarkManager::new(&app)
        .map_err(|e| format!("Failed to initialize bookmark manager: {}", e))?;

    bookmark_manager.get_bookmarks_by_category(&category)
        .map_err(|e| format!("Failed to get bookmarks: {}", e))
}

#[tauri::command]
async fn get_all_bookmarks(
    app: tauri::AppHandle,
) -> Result<Vec<Bookmark>, String> {
    let bookmark_manager = BookmarkManager::new(&app)
        .map_err(|e| format!("Failed to initialize bookmark manager: {}", e))?;

    bookmark_manager.get_all_bookmarks()
        .map_err(|e| format!("Failed to get all bookmarks: {}", e))
}







#[tauri::command]
async fn open_internal_browser(
    app: tauri::AppHandle,
    url: String,
    title: Option<String>
) -> Result<String, String> {
    let window_label = format!("browser_{}", chrono::Utc::now().timestamp());

    let _window = WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::External(url.parse().map_err(|e| format!("Invalid URL: {}", e))?)
    )
    .title(&title.unwrap_or_else(|| "内置浏览器".to_string()))
    .inner_size(1000.0, 700.0)
    .center()
    .resizable(true)
    .build()
    .map_err(|e| format!("Failed to create browser window: {}", e))?;

    Ok(window_label)
}

#[tauri::command]
async fn close_window(app: tauri::AppHandle, window_label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&window_label) {
        window.close().map_err(|e| format!("Failed to close window: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
async fn get_customer_info(token: String) -> Result<String, String> {
    let url = format!("https://portal.withorb.com/api/v1/customer_from_link?token={}", token);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Accept-Charset", "utf-8")
        .header("Connection", "keep-alive")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-origin")
        .send()
        .await
        .map_err(|e| format!("Failed to make API request: {}", e))?;

    let status = response.status();

    if status.is_success() {
        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response bytes: {}", e))?;

        let response_text = String::from_utf8_lossy(&bytes).to_string();

        match serde_json::from_str::<serde_json::Value>(&response_text) {
            Ok(json_value) => {
                match serde_json::to_string_pretty(&json_value) {
                    Ok(formatted) => Ok(formatted),
                    Err(_) => Ok(response_text),
                }
            }
            Err(_) => Ok(response_text),
        }
    } else {
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;
        Err(format!("API request failed with status {}: {}", status, response_text))
    }
}

#[tauri::command]
async fn get_ledger_summary(customer_id: String, pricing_unit_id: String, token: String) -> Result<String, String> {
    let url = format!("https://portal.withorb.com/api/v1/customers/{}/ledger_summary?pricing_unit_id={}&token={}",
                     customer_id, pricing_unit_id, token);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Accept-Charset", "utf-8")
        .header("Connection", "keep-alive")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-origin")
        .send()
        .await
        .map_err(|e| format!("Failed to make API request: {}", e))?;

    let status = response.status();

    if status.is_success() {
        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response bytes: {}", e))?;

        let response_text = String::from_utf8_lossy(&bytes).to_string();

        match serde_json::from_str::<serde_json::Value>(&response_text) {
            Ok(json_value) => {
                match serde_json::to_string_pretty(&json_value) {
                    Ok(formatted) => Ok(formatted),
                    Err(_) => Ok(response_text),
                }
            }
            Err(_) => Ok(response_text),
        }
    } else {
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;
        Err(format!("API request failed with status {}: {}", status, response_text))
    }
}

#[tauri::command]
async fn test_api_call() -> Result<String, String> {
    let url = "https://portal.withorb.com/api/v1/customer_from_link?token=ImRhUHFhU3ZtelpKdEJrUVci.1konHDs_4UqVUJWcxaZpKV4nQik";

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Accept-Charset", "utf-8")
        .header("Connection", "keep-alive")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-origin")
        .send()
        .await
        .map_err(|e| format!("Failed to make API request: {}", e))?;

    let status = response.status();

    if status.is_success() {
        // 尝试获取JSON并格式化
        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response bytes: {}", e))?;

        // 确保使用UTF-8解码
        let response_text = String::from_utf8_lossy(&bytes).to_string();

        // 尝试解析并格式化JSON
        match serde_json::from_str::<serde_json::Value>(&response_text) {
            Ok(json_value) => {
                // 格式化JSON输出
                match serde_json::to_string_pretty(&json_value) {
                    Ok(formatted) => Ok(formatted),
                    Err(_) => Ok(response_text), // 如果格式化失败，返回原始文本
                }
            }
            Err(_) => Ok(response_text), // 如果不是有效JSON，返回原始文本
        }
    } else {
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;
        Err(format!("API request failed with status {}: {}", status, response_text))
    }
}

#[tauri::command]
async fn open_data_folder(
    app: tauri::AppHandle,
) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    // Open folder using system default file manager
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&app_data_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&app_data_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&app_data_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
async fn create_jetbrains_token_file(
    editor_type: String,
    token_data: String,
) -> Result<String, String> {
    use std::fs;
    use std::env;
    use std::path::PathBuf;

    // 获取用户主目录
    let home_dir = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map_err(|_| "Failed to get home directory".to_string())?;

    let augment_dir = PathBuf::from(&home_dir).join(".augment");

    // 确保 .augment 目录存在
    fs::create_dir_all(&augment_dir)
        .map_err(|e| format!("Failed to create .augment directory: {}", e))?;

    // 创建文件路径
    let file_name = format!("{}_token.json", editor_type);
    let file_path = augment_dir.join(&file_name);

    // 写入文件
    fs::write(&file_path, token_data)
        .map_err(|e| format!("Failed to write token file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn open_editor_with_protocol(
    app: tauri::AppHandle,
    protocol_url: String,
) -> Result<(), String> {
    println!("Opening editor with protocol URL: {}", protocol_url);

    use tauri_plugin_opener::OpenerExt;
    app.opener().open_url(protocol_url, None::<&str>)
        .map_err(|e| format!("Failed to open editor with protocol: {}", e))
}

// Outlook 邮箱管理命令
#[tauri::command]
async fn outlook_save_credentials(
    email: String,
    refresh_token: String,
    client_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let credentials = OutlookCredentials {
        email,
        refresh_token,
        client_id,
    };

    let mut manager = state.outlook_manager.lock().unwrap();
    manager.save_credentials(credentials)
}

#[tauri::command]
async fn outlook_get_all_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let manager = state.outlook_manager.lock().unwrap();
    manager.get_all_accounts()
}

#[tauri::command]
async fn outlook_delete_account(
    email: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let mut manager = state.outlook_manager.lock().unwrap();
    manager.delete_account(&email)
}

#[tauri::command]
async fn outlook_check_account_status(
    email: String,
    state: State<'_, AppState>,
) -> Result<OutlookAccountStatus, String> {
    // 克隆必要的数据以避免跨 await 持有锁
    let credentials = {
        let manager = state.outlook_manager.lock().unwrap();
        manager.get_credentials(&email)?
    };

    // 创建临时管理器实例进行状态检查
    let temp_manager = OutlookManager::new();
    temp_manager.check_account_status_with_credentials(&credentials).await
}

#[tauri::command]
async fn outlook_get_emails(
    email: String,
    folder: String,
    page: i32,
    page_size: i32,
    state: State<'_, AppState>,
) -> Result<EmailListResponse, String> {
    // 克隆必要的数据以避免跨 await 持有锁
    let credentials = {
        let manager = state.outlook_manager.lock().unwrap();
        manager.get_credentials(&email)?
    };

    // 创建临时管理器实例进行邮件获取
    let temp_manager = OutlookManager::new();
    temp_manager.get_emails_with_credentials(&credentials, &folder, page, page_size).await
}

#[tauri::command]
async fn outlook_get_email_details(
    email: String,
    message_id: String,
    state: State<'_, AppState>,
) -> Result<EmailDetailsResponse, String> {
    // 克隆必要的数据以避免跨 await 持有锁
    let credentials = {
        let manager = state.outlook_manager.lock().unwrap();
        manager.get_credentials(&email)?
    };

    // 创建临时管理器实例进行邮件详情获取
    let temp_manager = OutlookManager::new();
    temp_manager.get_email_details_with_credentials(&credentials, &message_id).await
}

// 数据库配置相关命令
#[tauri::command]
async fn save_database_config(
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config_manager = DatabaseConfigManager::new(&app)
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    let config = DatabaseConfig::new(host, port, database, username, password);

    config_manager.save_config(&config)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    // 尝试初始化数据库连接
    let mut db_manager = DatabaseManager::new(config);
    match db_manager.initialize().await {
        Ok(_) => {
            // 检查数据库表是否已存在
            if let Some(pool) = db_manager.get_pool() {
                let client = pool.get().await
                    .map_err(|e| format!("Failed to get database client: {}", e))?;

                let tables_exist = database::check_tables_exist(&client).await
                    .map_err(|e| format!("Failed to check tables: {}", e))?;

                if !tables_exist {
                    // 表不存在，创建表
                    database::create_tables(&client).await
                        .map_err(|e| format!("Failed to create tables: {}", e))?;
                } else {
                    // 表已存在，执行一次同步
                    println!("Database tables already exist, will perform initial sync");
                }
            }

            // 更新应用状态
            *state.database_manager.lock().unwrap() = Some(Arc::new(db_manager));

            // 重新初始化存储管理器
            initialize_storage_manager(&app, &state).await
                .map_err(|e| format!("Failed to initialize storage: {}", e))?;

            // 如果表已存在，执行初始同步
            let pool_option = {
                let db_guard = state.database_manager.lock().unwrap();
                db_guard.as_ref().and_then(|db| db.get_pool())
            };

            if let Some(pool) = pool_option {
                let client = pool.get().await
                    .map_err(|e| format!("Failed to get database client for sync check: {}", e))?;

                let tables_exist = database::check_tables_exist(&client).await
                    .map_err(|e| format!("Failed to check tables for sync: {}", e))?;

                if tables_exist {
                    // 执行初始双向同步
                    let storage_manager = {
                        let storage_guard = state.storage_manager.lock().unwrap();
                        storage_guard.as_ref().cloned()
                    };

                    if let Some(storage_manager) = storage_manager {
                        match storage_manager.bidirectional_sync().await {
                            Ok(sync_result) => {
                                println!("Initial sync completed: {} tokens synced", sync_result.tokens_synced);
                            }
                            Err(e) => {
                                eprintln!("Initial sync failed: {}", e);
                                // 同步失败不影响配置保存
                            }
                        }
                    }
                }
            }

            Ok(())
        }
        Err(e) => Err(format!("Failed to connect to database: {}", e))
    }
}

#[tauri::command]
async fn load_database_config(
    app: tauri::AppHandle,
) -> Result<DatabaseConfig, String> {
    let config_manager = DatabaseConfigManager::new(&app)
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    config_manager.load_config()
        .map_err(|e| format!("Failed to load config: {}", e))
}

#[tauri::command]
async fn test_database_connection(
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
) -> Result<(), String> {
    let config = DatabaseConfig::new(host, port, database, username, password);

    database::test_database_connection(&config).await
        .map_err(|e| format!("Connection test failed: {}", e))
}

#[tauri::command]
async fn delete_database_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config_manager = DatabaseConfigManager::new(&app)
        .map_err(|e| format!("Failed to create config manager: {}", e))?;

    config_manager.delete_config()
        .map_err(|e| format!("Failed to delete config: {}", e))?;

    // 清除应用状态中的数据库管理器
    *state.database_manager.lock().unwrap() = None;

    // 重新初始化存储管理器（仅本地存储）
    initialize_storage_manager(&app, &state).await
        .map_err(|e| format!("Failed to reinitialize storage: {}", e))?;

    Ok(())
}

// 同步相关命令
#[tauri::command]
async fn sync_tokens_to_database(
    state: State<'_, AppState>,
) -> Result<storage::SyncStatus, String> {
    let storage_manager = {
        let guard = state.storage_manager.lock().unwrap();
        guard.clone().ok_or("Storage manager not initialized")?
    };

    storage_manager.sync_local_to_remote().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
async fn sync_tokens_from_database(
    state: State<'_, AppState>,
) -> Result<storage::SyncStatus, String> {
    let storage_manager = {
        let guard = state.storage_manager.lock().unwrap();
        guard.clone().ok_or("Storage manager not initialized")?
    };

    storage_manager.sync_remote_to_local().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
async fn delete_token(
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
async fn bidirectional_sync_tokens(
    state: State<'_, AppState>,
) -> Result<storage::SyncStatus, String> {
    let storage_manager = {
        let guard = state.storage_manager.lock().unwrap();
        guard.clone().ok_or("Storage manager not initialized")?
    };

    storage_manager.bidirectional_sync().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
async fn get_storage_status(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    // 检查存储管理器是否已初始化，如果没有则尝试初始化
    let storage_manager = {
        let manager_option = {
            let guard = state.storage_manager.lock().unwrap();
            guard.clone()
        };

        if let Some(manager) = manager_option {
            manager
        } else {
            // 尝试初始化存储管理器
            if let Err(e) = initialize_storage_manager(&app, &state).await {
                return Err(format!("Failed to initialize storage manager: {}", e));
            }
            // 重新获取存储管理器
            let guard = state.storage_manager.lock().unwrap();
            guard.clone().ok_or("Storage manager still not initialized after initialization attempt")?
        }
    };

    let is_available = storage_manager.is_available().await;
    let storage_type = storage_manager.storage_type();
    let is_database_available = storage_manager.is_database_available();

    Ok(serde_json::json!({
        "is_available": is_available,
        "storage_type": storage_type,
        "is_database_available": is_database_available
    }))
}

#[tauri::command]
async fn get_sync_status(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<storage::SyncStatus>, String> {
    // 检查存储管理器是否已初始化，如果没有则尝试初始化
    let storage_manager = {
        let manager_option = {
            let guard = state.storage_manager.lock().unwrap();
            guard.clone()
        };

        if let Some(manager) = manager_option {
            manager
        } else {
            // 尝试初始化存储管理器
            if let Err(e) = initialize_storage_manager(&app, &state).await {
                return Err(format!("Failed to initialize storage manager: {}", e));
            }
            // 重新获取存储管理器
            let guard = state.storage_manager.lock().unwrap();
            guard.clone().ok_or("Storage manager still not initialized after initialization attempt")?
        }
    };

    storage_manager.get_sync_status().await
        .map_err(|e| format!("Failed to get sync status: {}", e))
}

// 辅助函数：初始化存储管理器
async fn initialize_storage_manager(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 创建本地存储
    let local_storage = Arc::new(LocalFileStorage::new(app)?);

    // 尝试加载数据库配置并创建数据库存储
    let postgres_storage = {
        let db_manager_guard = state.database_manager.lock().unwrap();
        if let Some(db_manager) = db_manager_guard.as_ref() {
            Some(Arc::new(PostgreSQLStorage::new(db_manager.clone())))
        } else {
            None
        }
    };

    // 创建双重存储管理器
    let dual_storage = Arc::new(DualStorage::new(local_storage, postgres_storage));

    // 更新应用状态
    *state.storage_manager.lock().unwrap() = Some(dual_storage);

    Ok(())
}



fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_state = AppState {
                augment_oauth_state: Mutex::new(None),
                http_server: Mutex::new(None),
                outlook_manager: Mutex::new(OutlookManager::new()),
                storage_manager: Arc::new(Mutex::new(None)),
                database_manager: Arc::new(Mutex::new(None)),
            };

            app.manage(app_state);

            // 异步初始化存储管理器
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<AppState>();

                // 尝试加载数据库配置
                let mut should_sync = false;
                match DatabaseConfigManager::new(&app_handle) {
                    Ok(config_manager) => {
                        match config_manager.load_config() {
                            Ok(config) => {
                                if config.enabled {
                                    let mut db_manager = DatabaseManager::new(config);
                                    if db_manager.initialize().await.is_ok() {
                                        // 检查表是否存在
                                        should_sync = if let Some(pool) = db_manager.get_pool() {
                                            match pool.get().await {
                                                Ok(client) => {
                                                    match database::check_tables_exist(&client).await {
                                                        Ok(exists) => {
                                                            if !exists {
                                                                // 创建表
                                                                if let Err(e) = database::create_tables(&client).await {
                                                                    eprintln!("Failed to create tables on startup: {}", e);
                                                                }
                                                                false // 新创建的表不需要同步
                                                            } else {
                                                                true // 表已存在，需要同步
                                                            }
                                                        }
                                                        Err(e) => {
                                                            eprintln!("Failed to check tables on startup: {}", e);
                                                            false
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to get database client on startup: {}", e);
                                                    false
                                                }
                                            }
                                        } else {
                                            false
                                        };

                                        *state.database_manager.lock().unwrap() = Some(Arc::new(db_manager));

                                        // 如果需要同步，在存储管理器初始化后执行
                                        if should_sync {
                                            // 初始化存储管理器
                                            if let Err(e) = initialize_storage_manager(&app_handle, &state).await {
                                                eprintln!("Failed to initialize storage manager on startup: {}", e);
                                            } else {
                                                // 执行初始同步
                                                let storage_manager = {
                                                    let storage_guard = state.storage_manager.lock().unwrap();
                                                    storage_guard.as_ref().cloned()
                                                };

                                                if let Some(storage_manager) = storage_manager {
                                                    match storage_manager.bidirectional_sync().await {
                                                        Ok(sync_result) => {
                                                            println!("Startup sync completed: {} tokens synced", sync_result.tokens_synced);
                                                        }
                                                        Err(e) => {
                                                            eprintln!("Startup sync failed: {}", e);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => eprintln!("Failed to load database config: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Failed to create config manager: {}", e),
                }

                // 如果没有执行同步（表不存在或数据库不可用），则初始化存储管理器
                if !should_sync {
                    if let Err(e) = initialize_storage_manager(&app_handle, &state).await {
                        eprintln!("Failed to initialize storage manager: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_auth_url,
            generate_augment_auth_url,
            get_token,
            get_augment_token,
            check_account_status,
            open_url,
            // 新的简化命令
            save_tokens_json,
            load_tokens_json,
            // 书签管理命令
            add_bookmark,
            update_bookmark,
            delete_bookmark,
            get_bookmarks,
            get_all_bookmarks,
            // API 调用命令
            get_customer_info,
            get_ledger_summary,
            test_api_call,
            open_data_folder,
            open_editor_with_protocol,
            create_jetbrains_token_file,
            // Outlook 邮箱管理命令
            outlook_save_credentials,
            outlook_get_all_accounts,
            outlook_delete_account,
            outlook_check_account_status,
            outlook_get_emails,
            outlook_get_email_details,
            // 数据库配置命令
            save_database_config,
            load_database_config,
            test_database_connection,
            delete_database_config,
            // 同步命令
            sync_tokens_to_database,
            sync_tokens_from_database,
            // 删除命令
            delete_token,
            bidirectional_sync_tokens,
            get_storage_status,
            get_sync_status,

            open_internal_browser,
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
