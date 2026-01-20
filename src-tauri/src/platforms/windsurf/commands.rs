//! Windsurf Tauri Commands

use tauri::AppHandle;
use serde::Serialize;
use crate::windsurf::models::{Account, TokenData, QuotaData};
use crate::windsurf::modules::{storage, auth, process, db, machine, api};

/// 账号列表响应
#[derive(Serialize)]
pub struct AccountListResponse {
    pub accounts: Vec<Account>,
    pub current_account_id: Option<String>,
}

/// 使用邮箱密码登录
#[tauri::command]
pub async fn windsurf_login(
    app: AppHandle,
    email: String,
    password: String,
) -> Result<Account, String> {
    // 1. Firebase 登录
    let token_res = auth::login_with_email_password(&email, &password).await?;
    
    // 2. 检查邮箱是否已存在
    let email_to_check = email.trim().to_lowercase();
    let existing_accounts = storage::list_accounts(&app).await?;
    
    if existing_accounts.iter().any(|account| {
        account.email.trim().to_lowercase() == email_to_check
    }) {
        return Err(format!("Account with email '{}' already exists", email));
    }
    
    // 3. 创建 Token 数据
    let expires_in = token_res.expires_in_seconds();
    let token = TokenData::new(
        token_res.id_token,
        token_res.refresh_token,
        expires_in,
        Some(email.clone()),
        token_res.local_id.clone(),
    );
    
    // 4. 创建账号
    let account_id = uuid::Uuid::new_v4().to_string();
    let mut account = Account::new(account_id, email, token);
    
    // 5. 获取用户显示名称
    if let Ok(user_info) = auth::get_user_info(&account.token.access_token).await {
        account.name = user_info.display_name;
    }
    
    // 6. 保存账号
    storage::save_account(&app, &account).await?;
    
    Ok(account)
}

/// 使用 refresh_token 添加账号
#[tauri::command]
pub async fn windsurf_add_account(
    app: AppHandle,
    refresh_token: String,
) -> Result<Account, String> {
    // 1. 使用 refresh_token 获取新的 token
    let token_res = auth::refresh_access_token(&refresh_token).await?;
    
    // 2. 获取用户信息
    let user_info = auth::get_user_info(&token_res.id_token).await?;
    
    // 3. 检查邮箱是否已存在
    let email_to_check = user_info.email.trim().to_lowercase();
    let existing_accounts = storage::list_accounts(&app).await?;
    
    if existing_accounts.iter().any(|account| {
        account.email.trim().to_lowercase() == email_to_check
    }) {
        return Err(format!("Account with email '{}' already exists", user_info.email));
    }
    
    // 4. 创建 Token 数据
    let expires_in = token_res.expires_in_seconds();
    let token = TokenData::new(
        token_res.id_token,
        token_res.refresh_token,
        expires_in,
        Some(user_info.email.clone()),
        Some(user_info.local_id.clone()),
    );
    
    // 5. 创建账号
    let account_id = uuid::Uuid::new_v4().to_string();
    let mut account = Account::new(account_id, user_info.email, token);
    account.name = user_info.display_name;
    
    // 6. 保存账号
    storage::save_account(&app, &account).await?;
    
    Ok(account)
}

/// 列出所有账号
#[tauri::command]
pub async fn windsurf_list_accounts(app: AppHandle) -> Result<AccountListResponse, String> {
    let accounts = storage::list_accounts(&app).await?;
    let current_account_id = storage::get_current_account_id(&app).await?;

    Ok(AccountListResponse {
        accounts,
        current_account_id,
    })
}

/// 删除账号
#[tauri::command]
pub async fn windsurf_delete_account(
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
pub async fn windsurf_switch_account(
    app: AppHandle,
    account_id: String,
) -> Result<String, String> {
    // 1. 加载账号
    let mut acc = storage::load_account(&app, &account_id).await?;
    
    // 2. 确保 Token 有效
    let token = auth::ensure_fresh_token(&acc.token).await?;
    if token.access_token != acc.token.access_token {
        acc.token = token.clone();
        storage::save_account(&app, &acc).await?;
    }

    // 2.1 确保 API Key / 用户名可用
    if acc.api_key.is_none() || acc.api_server_url.is_none() || acc.name.is_none() {
        let api_res = api::get_api_key(&acc.token.access_token).await?;
        if acc.api_key.is_none() {
            acc.api_key = api_res.api_key;
        }
        if acc.api_server_url.is_none() {
            acc.api_server_url = api_res.api_server_url;
        }
        if acc.name.is_none() {
            acc.name = api_res.name;
        }
        if acc.api_key.is_none() {
            return Err("Failed to obtain Windsurf api_key".to_string());
        }
        storage::save_account(&app, &acc).await?;
    }

    // 3. 检查 Windsurf 是否运行
    let was_running = process::is_windsurf_running();

    if was_running {
        // 4. 关闭进程
        process::close_windsurf(20)?;

        // 4.0.1 二次验证：等待后再次检查
        std::thread::sleep(std::time::Duration::from_secs(2));
        if process::is_windsurf_running() {
            // 重试一次
            process::close_windsurf(10)?;
            std::thread::sleep(std::time::Duration::from_secs(1));
            if process::is_windsurf_running() {
                return Err("Windsurf process still running after multiple close attempts".to_string());
            }
        }
    }

    // 4.1 重置遥测数据（失败不阻断切号）
    if let Err(e) = machine::reset_machine_id() {
        eprintln!("Failed to reset machine/telemetry IDs: {}", e);
    }
    
    // 5. 注入 Token
    let db_path = db::get_db_path()?;
    if db_path.exists() {
        // 备份数据库
        if let Some(backup_path) = db_path.with_extension("vscdb.backup").to_str() {
            let _ = std::fs::copy(&db_path, backup_path);
        }
    }
    
    // 注意：参考项目不写入 firebase:authUser:*，只写入 secret:// 和 windsurfAuthStatus
    // 因此这里跳过 inject_token 调用，直接写入 Windsurf 扩展登录状态

    // 5. 写入 Windsurf 扩展登录状态
    let api_key = acc.api_key.clone().ok_or("Missing Windsurf api_key")?;
    let api_server_url = acc.api_server_url
        .clone()
        .unwrap_or_else(|| "https://server.self-serve.windsurf.com".to_string());
    let name = acc.name.clone().unwrap_or_else(|| acc.email.clone());
    db::write_windsurf_auth_state(
        &db_path,
        &api_key,
        &api_server_url,
        &name,
        &acc.email,
    )?;

    // 6. 更新当前账号
    storage::set_current_account_id(&app, Some(account_id.clone())).await?;

    // 7. 更新最后使用时间
    acc.update_last_used();
    storage::save_account(&app, &acc).await?;

    // 8. 获取自定义路径并启动 Windsurf
    std::thread::sleep(std::time::Duration::from_secs(1));
    use crate::core::path_manager::{read_custom_path_from_config, WINDSURF_CONFIG};
    let custom_path = read_custom_path_from_config(&app, &WINDSURF_CONFIG);

    process::launch_windsurf_with_path(custom_path.as_deref())?;

    Ok(format!("Account switched and Windsurf started: {}", acc.email))
}


/// 查询账号配额信息
#[tauri::command]
pub async fn windsurf_fetch_quota(
    app: AppHandle,
    account_id: String,
) -> Result<Account, String> {
    let mut acc = storage::load_account(&app, &account_id).await?;

    // 确保 Token 有效
    let token = auth::ensure_fresh_token(&acc.token).await?;
    if token.access_token != acc.token.access_token {
        acc.token = token.clone();
    }

    // 查询配额
    let quota_result = api::get_plan_status(&acc.token.access_token).await?;

    // 更新账号配额信息
    acc.quota = Some(QuotaData {
        plan_name: quota_result.plan_name,
        used_credits: quota_result.used_credits,
        total_credits: quota_result.total_credits,
        usage_percentage: quota_result.usage_percentage,
        expires_at: quota_result.expires_at,
        plan_start: quota_result.plan_start,
        last_updated: chrono::Utc::now().timestamp(),
    });
    acc.updated_at = chrono::Utc::now().timestamp();

    // 保存更新
    storage::save_account(&app, &acc).await?;

    Ok(acc)
}

/// 批量查询所有账号配额
#[tauri::command]
pub async fn windsurf_fetch_all_quotas(
    app: AppHandle,
) -> Result<Vec<Account>, String> {
    let accounts = storage::list_accounts(&app).await?;
    let mut updated_accounts = Vec::new();

    for mut acc in accounts {
        // 确保 Token 有效
        match auth::ensure_fresh_token(&acc.token).await {
            Ok(token) => {
                if token.access_token != acc.token.access_token {
                    acc.token = token.clone();
                }

                // 查询配额
                match api::get_plan_status(&acc.token.access_token).await {
                    Ok(quota_result) => {
                        acc.quota = Some(QuotaData {
                            plan_name: quota_result.plan_name,
                            used_credits: quota_result.used_credits,
                            total_credits: quota_result.total_credits,
                            usage_percentage: quota_result.usage_percentage,
                            expires_at: quota_result.expires_at,
                            plan_start: quota_result.plan_start,
                            last_updated: chrono::Utc::now().timestamp(),
                        });
                        acc.updated_at = chrono::Utc::now().timestamp();
                        let _ = storage::save_account(&app, &acc).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch quota for {}: {}", acc.email, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to refresh token for {}: {}", acc.email, e);
            }
        }

        updated_accounts.push(acc);
    }

    Ok(updated_accounts)
}


/// 获取自定义 Windsurf 路径
#[tauri::command]
pub async fn windsurf_get_custom_path(app: AppHandle) -> Result<Option<String>, String> {
    use crate::core::path_manager::{get_custom_path, WINDSURF_CONFIG};
    get_custom_path(&app, &WINDSURF_CONFIG)
}

/// 设置自定义 Windsurf 路径
#[tauri::command]
pub async fn windsurf_set_custom_path(
    app: AppHandle,
    path: Option<String>,
) -> Result<(), String> {
    use crate::core::path_manager::{set_custom_path, WINDSURF_CONFIG};
    set_custom_path(&app, &WINDSURF_CONFIG, path, |p| process::validate_windsurf_path(p))
}

/// 验证 Windsurf 路径
#[tauri::command]
pub async fn windsurf_validate_path(path: String) -> Result<bool, String> {
    process::validate_windsurf_path(&path)
}

/// 获取默认 Windsurf 路径
#[tauri::command]
pub async fn windsurf_get_default_path() -> Result<String, String> {
    process::get_windsurf_executable_path()
        .map(|p| p.to_string_lossy().to_string())
}

/// 打开文件选择对话框选择 Windsurf 可执行文件
#[tauri::command]
pub async fn windsurf_select_executable_path() -> Result<Option<String>, String> {
    use crate::core::path_manager::{select_executable_path, WINDSURF_CONFIG};
    select_executable_path(&WINDSURF_CONFIG)
}
