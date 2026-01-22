//! Cursor Tauri Commands

use tauri::AppHandle;
use serde::Serialize;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use crate::cursor::models::{Account, TokenData};
use crate::cursor::modules::{storage, auth, process, db, machine};

/// 从 JWT token 中解析 exp 字段（过期时间戳）
fn parse_jwt_exp(token: &str) -> Option<i64> {
    // JWT 格式: header.payload.signature
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // 解码 payload (第二部分)
    let payload = URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
    let payload_str = String::from_utf8(payload).ok()?;

    // 解析 JSON 并提取 exp 字段
    let json: serde_json::Value = serde_json::from_str(&payload_str).ok()?;
    json.get("exp")?.as_i64()
}

/// 从 session token 中解析 exp 字段
/// session token 格式: user_id%3A%3A<JWT> 或 user_id::<JWT>
fn parse_session_token_exp(session_token: &str) -> Option<i64> {
    // 提取 JWT 部分
    let jwt = if session_token.contains("%3A%3A") {
        session_token.split("%3A%3A").nth(1)?
    } else if session_token.contains("::") {
        session_token.split("::").nth(1)?
    } else {
        return None;
    };

    parse_jwt_exp(jwt)
}

/// 账号列表响应
#[derive(Serialize)]
pub struct AccountListResponse {
    pub accounts: Vec<Account>,
    pub current_account_id: Option<String>,
}

/// 列出所有账号
#[tauri::command]
pub async fn cursor_list_accounts(app: AppHandle) -> Result<AccountListResponse, String> {
    let accounts = storage::list_accounts(&app).await?;
    let current_account_id = storage::get_current_account_id(&app).await?;

    Ok(AccountListResponse {
        accounts,
        current_account_id,
    })
}

/// 删除账号
#[tauri::command]
pub async fn cursor_delete_account(
    app: AppHandle,
    account_id: String,
) -> Result<(), String> {
    let deleted = storage::delete_account(&app, &account_id).await?;
    if !deleted {
        return Err(format!("Account not found: {}", account_id));
    }
    Ok(())
}

/// 切换账号（完整流程）
#[tauri::command]
pub async fn cursor_switch_account(
    app: AppHandle,
    account_id: String,
) -> Result<String, String> {
    // 1. 加载账号
    let mut acc = storage::load_account(&app, &account_id).await?;

    // 3. 检查 Cursor 是否运行
    let was_running = process::is_cursor_running();

    if was_running {
        // 4. 关闭进程
        process::close_cursor(20)?;

        // 二次验证：等待后再次检查
        std::thread::sleep(std::time::Duration::from_secs(2));
        if process::is_cursor_running() {
            // 重试一次
            process::close_cursor(10)?;
            std::thread::sleep(std::time::Duration::from_secs(1));
            if process::is_cursor_running() {
                return Err("Cursor process still running after multiple close attempts".to_string());
            }
        }
    }

    // 4.1 获取自定义路径用于 JS 修改
    use crate::core::path_manager::{read_custom_path_from_config, CURSOR_CONFIG};
    let custom_path = read_custom_path_from_config(&app, &CURSOR_CONFIG);

    // 4.2 完整重置：机器ID + main.js 修改（失败不阻断切号）
    if let Err(e) = machine::complete_reset(custom_path.as_deref()) {
        eprintln!("Warning during reset: {:?}", e);
    }

    // 5. 注入 Token
    let db_path = db::get_db_path()?;
    if db_path.exists() {
        // 备份数据库
        if let Some(backup_path) = db_path.with_extension("vscdb.backup").to_str() {
            let _ = std::fs::copy(&db_path, backup_path);
        }
    }
    
    // 写入 Cursor 认证状态
    let user_id = acc.token.user_id.clone().unwrap_or_else(|| acc.id.clone());
    db::write_cursor_auth_state(
        &db_path,
        &acc.token.access_token,
        &acc.token.refresh_token,
        &acc.email,
        &user_id,
    )?;

    // 6. 更新当前账号
    storage::set_current_account_id(&app, Some(account_id.clone())).await?;

    // 7. 更新最后使用时间
    acc.update_last_used();
    storage::save_account(&app, &acc).await?;

    // 8. 启动 Cursor（复用之前获取的自定义路径）
    std::thread::sleep(std::time::Duration::from_secs(1));
    process::launch_cursor_with_path(custom_path.as_deref())?;

    // 9. 刷新 access token（Cursor 启动后执行，失败不影响主流程）
    if let Some(session_token) = acc.token.workos_cursor_session_token.as_deref() {
        match auth::get_access_token_from_session(session_token).await {
            Ok(token_response) => {
                // 更新 access_token
                acc.token.access_token = token_response.access_token;
                // 如果返回了 refresh_token 也更新
                if let Some(refresh_token) = token_response.refresh_token {
                    acc.token.refresh_token = refresh_token;
                }
                // 保存更新后的账号信息
                if let Err(e) = storage::save_account(&app, &acc).await {
                    eprintln!("Warning: Failed to save refreshed token: {:?}", e);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to refresh access token: {:?}", e);
            }
        }
    }

    Ok(format!("Account switched and Cursor started: {}", acc.email))
}

/// 获取自定义 Cursor 路径
#[tauri::command]
pub async fn cursor_get_custom_path(app: AppHandle) -> Result<Option<String>, String> {
    use crate::core::path_manager::{get_custom_path, CURSOR_CONFIG};
    get_custom_path(&app, &CURSOR_CONFIG)
}

/// 设置自定义 Cursor 路径
#[tauri::command]
pub async fn cursor_set_custom_path(
    app: AppHandle,
    path: Option<String>,
) -> Result<(), String> {
    use crate::core::path_manager::{set_custom_path, CURSOR_CONFIG};
    set_custom_path(&app, &CURSOR_CONFIG, path, |p| process::validate_cursor_path(p))
}

/// 验证 Cursor 路径
#[tauri::command]
pub async fn cursor_validate_path(path: String) -> Result<bool, String> {
    process::validate_cursor_path(&path)
}

/// 获取默认 Cursor 路径
#[tauri::command]
pub async fn cursor_get_default_path() -> Result<String, String> {
    process::get_cursor_executable_path()
        .map(|p| p.to_string_lossy().to_string())
}

/// 打开文件选择对话框选择 Cursor 可执行文件
#[tauri::command]
pub async fn cursor_select_executable_path() -> Result<Option<String>, String> {
    use crate::core::path_manager::{select_executable_path, CURSOR_CONFIG};
    select_executable_path(&CURSOR_CONFIG)
}

/// 验证账号 Token 是否有效
#[tauri::command]
pub async fn cursor_validate_account(
    app: AppHandle,
    account_id: String,
) -> Result<bool, String> {
    let acc = storage::load_account(&app, &account_id).await?;
    let session_token = acc.token.workos_cursor_session_token.as_deref()
        .ok_or_else(|| "No session token available".to_string())?;

    // 尝试获取用户信息来验证 token 是否有效
    match auth::get_user_info(session_token).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// 批量验证所有账号
// #[tauri::command]
// pub async fn cursor_validate_all_accounts(
//     app: AppHandle,
// ) -> Result<Vec<(String, bool)>, String> {
//     let accounts = storage::list_accounts(&app).await?;
//     let mut results = Vec::new();

//     for acc in accounts {
//         let is_valid = if let Some(session_token) = acc.token.workos_cursor_session_token.as_deref() {
            
//         } else {
            
//         };
//         results.push((acc.id, is_valid));
//     }

//     // Ok(results)
// }

/// 获取订阅信息
#[tauri::command]
pub async fn cursor_get_subscription_info(
    session_token: String,
) -> Result<auth::SubscriptionInfo, String> {
    auth::get_subscription_info(&session_token).await
}

/// 获取账号聚合用量数据
#[tauri::command]
pub async fn cursor_get_aggregated_usage(
    session_token: String,
    start_date: u64,
    end_date: u64,
    team_id: i32,
) -> Result<Option<auth::AggregatedUsageData>, String> {
    auth::get_aggregated_usage_data(&session_token, start_date, end_date, team_id).await
}

/// 获取账号过滤的使用事件
#[tauri::command]
pub async fn cursor_get_filtered_usage_events(
    session_token: String,
    start_date: String,
    end_date: String,
    page: i32,
    page_size: i32,
    team_id: i32,
) -> Result<Option<auth::FilteredUsageEventsData>, String> {
    auth::get_filtered_usage_events(&session_token, team_id, &start_date, &end_date, page, page_size).await
}

/// 使用 session token 添加账号（自动获取 accessToken）
///
/// 一站式添加账号：传入 session cookie，自动完成：
/// 1. 获取用户信息
/// 2. 获取订阅信息
/// 3. PKCE 流程获取 accessToken
/// 4. 保存账号
#[tauri::command]
pub async fn cursor_add_account_with_session(
    app: AppHandle,
    session_token: String,
) -> Result<Account, String> {
    // 1. 获取用户信息（使用 session_token + Cookie 认证）
    let user_info = auth::get_user_info(&session_token).await?;

    // 2. 检查邮箱是否已存在
    let email_to_check = user_info.email.trim().to_lowercase();
    let existing_accounts = storage::list_accounts(&app).await?;

    if existing_accounts.iter().any(|account| {
        account.email.trim().to_lowercase() == email_to_check
    }) {
        return Err(format!("Account with email '{}' already exists", user_info.email));
    }

    // 4. 获取 accessToken
    let token_response = auth::get_access_token_from_session(&session_token).await?;

    // 5. 解析 JWT 中的过期时间
    let expiry_timestamp = parse_jwt_exp(&token_response.access_token)
        .unwrap_or_else(|| chrono::Utc::now().timestamp() + 86400 * 60); // 默认 60 天

    // 6. 解析 session token 的过期时间
    let session_expiry_timestamp = parse_session_token_exp(&session_token);

    // 7. 创建 Token 数据
    let token = TokenData::new(
        token_response.access_token,
        token_response.refresh_token.unwrap_or_default(),
        expiry_timestamp,
        Some(user_info.id.clone()),
        Some(session_token),
        session_expiry_timestamp,
    );

    // 8. 创建账号
    let account_id = uuid::Uuid::new_v4().to_string();
    let mut account = Account::new(account_id, user_info.email, token);
    account.name = user_info.name;

    // 9. 保存账号
    storage::save_account(&app, &account).await?;

    Ok(account)
}