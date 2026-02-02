//! Cursor Tauri Commands

use tauri::AppHandle;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use crate::cursor::models::{Account, MachineInfo, TokenData};
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

/// 切换账号响应
#[derive(Serialize)]
pub struct SwitchAccountResponse {
    pub message: String,
    pub needs_admin: bool,
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
/// use_bound_machine_id: 是否使用账号绑定的机器码（如果有的话）
#[tauri::command]
pub async fn cursor_switch_account(
    app: AppHandle,
    account_id: String,
    use_bound_machine_id: Option<bool>,
) -> Result<SwitchAccountResponse, String> {
    // 1. 加载账号
    let mut acc = storage::load_account(&app, &account_id).await?;

    // 2. 检查 Cursor 是否运行并关闭
    // close_cursor 内部已经有完整的等待和验证逻辑，无需额外轮询
    if process::is_cursor_running() {
        process::close_cursor(5)?; // 减少超时时间，5秒足够
    }

    // 3. 获取自定义路径用于 JS 修改
    use crate::core::path_manager::{read_custom_path_from_config, CURSOR_CONFIG};
    let custom_path = read_custom_path_from_config(&app, &CURSOR_CONFIG);

    // 4. 机器码处理：根据选项决定使用绑定机器码还是随机生成
    let needs_admin = if use_bound_machine_id.unwrap_or(false) && acc.has_machine_info() {
        // 使用账号绑定的机器码
        let machine_info = acc.machine_info.as_ref().unwrap();
        match machine::complete_reset_with_machine_info(custom_path.as_deref(), machine_info) {
            Ok(result) => result.needs_admin,
            Err(_) => false,
        }
    } else {
        // 随机生成新的机器码
        match machine::complete_reset(custom_path.as_deref()) {
            Ok(result) => result.needs_admin,
            Err(_) => false,
        }
    };

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

    // 8. 启动 Cursor（短暂等待确保文件写入完成）
    std::thread::sleep(std::time::Duration::from_millis(100));
    process::launch_cursor_with_path(custom_path.as_deref())?;

    // 9. 后台刷新 access token（不阻塞主流程）
    // 注意：使用绑定机器码切换时不刷新 token，避免 token 与机器码不匹配
    let should_refresh_token = !use_bound_machine_id.unwrap_or(false);
    if should_refresh_token {
        if let Some(session_token) = acc.token.workos_cursor_session_token.clone() {
            let app_clone = app.clone();
            let mut acc_clone = acc.clone();
            tokio::spawn(async move {
                if let Ok(token_response) = auth::get_access_token_from_session(&session_token).await {
                    acc_clone.token.access_token = token_response.access_token;
                    if let Some(refresh_token) = token_response.refresh_token {
                        acc_clone.token.refresh_token = refresh_token;
                    }
                    let _ = storage::save_account(&app_clone, &acc_clone).await;
                }
            });
        }
    }

    Ok(SwitchAccountResponse {
        message: format!("Account switched and Cursor started: {}", acc.email),
        needs_admin,
    })
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

/// 更新账号信息
#[tauri::command]
pub async fn cursor_update_account(
    app: AppHandle,
    account: Account,
) -> Result<(), String> {
    storage::save_account(&app, &account).await
}

// ==================== 导入功能相关 ====================

/// 导入账号请求中的 auth_info 结构
#[derive(Debug, Clone, Deserialize)]
pub struct ImportAuthInfo {
    #[serde(rename = "WorkosCursorSessionToken")]
    pub workos_cursor_session_token: Option<String>,
    #[serde(rename = "cursorAuth/accessToken")]
    pub access_token: Option<String>,
    #[serde(rename = "cursorAuth/refreshToken")]
    pub refresh_token: Option<String>,
}

/// 导入账号请求中的单个账号数据
#[derive(Debug, Clone, Deserialize)]
pub struct ImportAccountData {
    pub email: String,
    pub auth_info: Option<ImportAuthInfo>,
    pub machine_info: Option<MachineInfo>,
}

/// 导入结果
#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub success: bool,
    pub email: String,
    pub error: Option<String>,
    pub account: Option<Account>,
}

/// 批量导入结果
#[derive(Debug, Clone, Serialize)]
pub struct BatchImportResult {
    pub total: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub results: Vec<ImportResult>,
}

/// 批量导入账号
/// 从 JSON 数据中导入账号，优先使用直接提供的 accessToken/refreshToken
/// 如果没有则尝试使用 WorkosCursorSessionToken 获取
#[tauri::command]
pub async fn cursor_import_accounts(
    app: AppHandle,
    accounts_data: Vec<ImportAccountData>,
) -> Result<BatchImportResult, String> {
    let total = accounts_data.len();
    let mut results = Vec::new();
    let mut success_count = 0;
    let mut failed_count = 0;

    // 获取现有账号列表用于检查重复
    let existing_accounts = storage::list_accounts(&app).await?;
    let existing_emails: std::collections::HashSet<String> = existing_accounts
        .iter()
        .map(|a| a.email.trim().to_lowercase())
        .collect();

    for data in accounts_data {
        let email = data.email.clone();
        let email_lower = email.trim().to_lowercase();

        // 检查邮箱是否已存在
        if existing_emails.contains(&email_lower) {
            results.push(ImportResult {
                success: false,
                email: email.clone(),
                error: Some(format!("Account with email '{}' already exists", email)),
                account: None,
            });
            failed_count += 1;
            continue;
        }

        // 获取 auth_info
        let auth_info = match &data.auth_info {
            Some(info) => info,
            None => {
                results.push(ImportResult {
                    success: false,
                    email: email.clone(),
                    error: Some("Missing auth_info".to_string()),
                    account: None,
                });
                failed_count += 1;
                continue;
            }
        };

        // 检查是否有直接的 accessToken 和 refreshToken
        let has_direct_tokens = auth_info.access_token.as_ref().map_or(false, |t| !t.is_empty())
            && auth_info.refresh_token.as_ref().map_or(false, |t| !t.is_empty());

        let import_result = if has_direct_tokens {
            // 直接使用提供的 accessToken 和 refreshToken
            import_account_with_tokens(
                &app,
                &email,
                auth_info.access_token.as_ref().unwrap(),
                auth_info.refresh_token.as_ref().unwrap(),
                auth_info.workos_cursor_session_token.clone(),
                data.machine_info.clone(),
            ).await
        } else if let Some(ref session_token) = auth_info.workos_cursor_session_token {
            if !session_token.is_empty() {
                // 使用 session token 获取 access token
                import_account_with_session_token(&app, &email, session_token, data.machine_info.clone()).await
            } else {
                Err("Missing valid auth credentials (accessToken or WorkosCursorSessionToken)".to_string())
            }
        } else {
            Err("Missing valid auth credentials (accessToken or WorkosCursorSessionToken)".to_string())
        };

        match import_result {
            Ok(account) => {
                results.push(ImportResult {
                    success: true,
                    email: email.clone(),
                    error: None,
                    account: Some(account),
                });
                success_count += 1;
            }
            Err(e) => {
                results.push(ImportResult {
                    success: false,
                    email: email.clone(),
                    error: Some(e),
                    account: None,
                });
                failed_count += 1;
            }
        }
    }

    Ok(BatchImportResult {
        total,
        success_count,
        failed_count,
        results,
    })
}

/// 使用直接提供的 accessToken 和 refreshToken 导入账号
async fn import_account_with_tokens(
    app: &AppHandle,
    email: &str,
    access_token: &str,
    refresh_token: &str,
    session_token: Option<String>,
    machine_info: Option<MachineInfo>,
) -> Result<Account, String> {
    // 1. 解析 JWT 中的过期时间
    let expiry_timestamp = parse_jwt_exp(access_token)
        .unwrap_or_else(|| chrono::Utc::now().timestamp() + 86400 * 60); // 默认 60 天

    // 2. 解析 session token 的过期时间（如果有的话）
    let session_expiry_timestamp = session_token.as_ref()
        .and_then(|t| parse_session_token_exp(t));

    // 3. 尝试从 session token 获取用户信息（用于获取 user_id）
    let user_id = if let Some(ref token) = session_token {
        match auth::get_user_info(token).await {
            Ok(user_info) => Some(user_info.id),
            Err(_) => None,
        }
    } else {
        None
    };

    // 4. 创建 Token 数据
    let token = TokenData::new(
        access_token.to_string(),
        refresh_token.to_string(),
        expiry_timestamp,
        user_id,
        session_token,
        session_expiry_timestamp,
    );

    // 5. 创建账号（带机器码信息）
    let account_id = uuid::Uuid::new_v4().to_string();
    let account = Account::new_with_machine_info(
        account_id,
        email.to_string(),
        token,
        machine_info,
    );

    // 6. 保存账号
    storage::save_account(app, &account).await?;

    Ok(account)
}

/// 使用 session token 获取 access token 并导入账号
async fn import_account_with_session_token(
    app: &AppHandle,
    email: &str,
    session_token: &str,
    machine_info: Option<MachineInfo>,
) -> Result<Account, String> {
    // 1. 获取 accessToken
    let token_response = auth::get_access_token_from_session(session_token).await?;

    // 2. 解析 JWT 中的过期时间
    let expiry_timestamp = parse_jwt_exp(&token_response.access_token)
        .unwrap_or_else(|| chrono::Utc::now().timestamp() + 86400 * 60); // 默认 60 天

    // 3. 解析 session token 的过期时间
    let session_expiry_timestamp = parse_session_token_exp(session_token);

    // 4. 尝试获取用户信息（用于获取 user_id）
    let user_id = match auth::get_user_info(session_token).await {
        Ok(user_info) => Some(user_info.id),
        Err(_) => None,
    };

    // 5. 创建 Token 数据
    let token = TokenData::new(
        token_response.access_token,
        token_response.refresh_token.unwrap_or_default(),
        expiry_timestamp,
        user_id,
        Some(session_token.to_string()),
        session_expiry_timestamp,
    );

    // 6. 创建账号（带机器码信息）
    let account_id = uuid::Uuid::new_v4().to_string();
    let account = Account::new_with_machine_info(
        account_id,
        email.to_string(),
        token,
        machine_info,
    );

    // 7. 保存账号
    storage::save_account(app, &account).await?;

    Ok(account)
}