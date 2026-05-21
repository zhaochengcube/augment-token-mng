//! Windsurf Tauri Commands

use crate::windsurf::models::{Account, QuotaData, TokenData};
use crate::windsurf::modules::{api, auth, db, devin, machine, patch, process, storage, switcher};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

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
}

/// 无感切号响应
#[derive(Serialize)]
pub struct SeamlessSwitchAccountResponse {
    pub success: bool,
    pub message: String,
    pub fallback_used: bool,
    pub machine_id_reset: bool,
    pub patch_applied: bool,
    pub patch_restarted: bool,
}

#[derive(Clone, Serialize)]
struct SwitchProgressPayload {
    step: &'static str,
    label: String,
    percent: u8,
    phase: &'static str,
}

fn emit_switch_progress(
    app: &AppHandle,
    step: &'static str,
    label: impl Into<String>,
    percent: u8,
    phase: &'static str,
) {
    let payload = SwitchProgressPayload {
        step,
        label: label.into(),
        percent,
        phase,
    };
    let _ = app.emit("windsurf-switch-progress", payload);
}

/// 使用邮箱密码登录
#[tauri::command]
pub async fn windsurf_login(
    app: AppHandle,
    email: String,
    password: String,
) -> Result<Account, String> {
    match windsurf_login_with_devin(&app, &email, &password).await {
        Ok(account) => return Ok(account),
        Err(devin_error) => {
            if should_return_devin_login_error(&devin_error) {
                return Err(devin_error);
            }
            eprintln!(
                "Devin login failed, falling back to Firebase: {}",
                devin_error
            );
        }
    }

    // 1. Firebase 登录
    let token_res = auth::login_with_email_password(&email, &password).await?;

    // 2. 检查邮箱是否已存在
    let email_to_check = email.trim().to_lowercase();
    let existing_accounts = storage::list_accounts(&app).await?;

    if existing_accounts
        .iter()
        .any(|account| account.email.trim().to_lowercase() == email_to_check)
    {
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

fn should_return_devin_login_error(error: &str) -> bool {
    let lower = error.to_lowercase();
    lower.contains("account with email")
        || lower.contains("invalid email or password")
        || lower.contains("too many attempts")
        || lower.contains("windsurfpostauth")
        || lower.contains("did not return a session token")
        || lower.contains("failed to parse devin login response")
}

async fn windsurf_login_with_devin(
    app: &AppHandle,
    email: &str,
    password: &str,
) -> Result<Account, String> {
    let login = devin::login_with_email_password(email, password).await?;

    let email_to_check = email.trim().to_lowercase();
    let existing_accounts = storage::list_accounts(app).await?;
    if existing_accounts
        .iter()
        .any(|account| account.email.trim().to_lowercase() == email_to_check)
    {
        return Err(format!("Account with email '{}' already exists", email));
    }

    let token = TokenData::new(
        login.session_token,
        String::new(),
        devin::devin_session_expires_in(),
        Some(email.to_string()),
        login.account_id.clone(),
    );

    let account_id = uuid::Uuid::new_v4().to_string();
    let mut account = Account::new(account_id, email.to_string(), token);
    account.auth_provider = Some("devin".to_string());
    account.devin_auth1_token = Some(login.auth1_token);
    account.devin_account_id = login.account_id;
    account.devin_primary_org_id = login.primary_org_id;
    account.api_server_url = Some("https://server.self-serve.windsurf.com".to_string());

    if let Err(e) = devin::enrich_account(&mut account).await {
        eprintln!("Failed to enrich Devin account {}: {}", account.email, e);
    }

    storage::save_account(app, &account).await?;

    Ok(account)
}

#[tauri::command]
pub async fn windsurf_add_account_by_devin_auth1(
    app: AppHandle,
    auth1_token: String,
    org_id: Option<String>,
) -> Result<Account, String> {
    let auth1_token = auth1_token.trim();
    if auth1_token.is_empty() {
        return Err("Auth1 token is required".to_string());
    }

    let login = devin::login_with_auth1_token(auth1_token, org_id.as_deref()).await?;
    let token = TokenData::new(
        login.session_token,
        String::new(),
        devin::devin_session_expires_in(),
        None,
        login.account_id.clone(),
    );

    let account_id = uuid::Uuid::new_v4().to_string();
    let fallback_email = login
        .account_id
        .clone()
        .unwrap_or_else(|| "devin-auth1-account".to_string());
    let mut account = Account::new(account_id, fallback_email, token);
    account.auth_provider = Some("devin".to_string());
    account.devin_auth1_token = Some(login.auth1_token);
    account.devin_account_id = login.account_id;
    account.devin_primary_org_id = login.primary_org_id;
    account.api_server_url = Some("https://server.self-serve.windsurf.com".to_string());

    devin::enrich_account(&mut account).await?;
    ensure_windsurf_email_not_exists(&app, &account.email).await?;
    storage::save_account(&app, &account).await?;

    Ok(account)
}

#[tauri::command]
pub async fn windsurf_add_account_by_devin_session(
    app: AppHandle,
    session_token: String,
) -> Result<Account, String> {
    let session_token = session_token.trim();
    if session_token.is_empty() {
        return Err("Session token is required".to_string());
    }

    let token = TokenData::new(
        session_token.to_string(),
        String::new(),
        devin::devin_session_expires_in(),
        None,
        None,
    );

    let account_id = uuid::Uuid::new_v4().to_string();
    let mut account = Account::new(account_id, "devin-session-account".to_string(), token);
    account.auth_provider = Some("devin".to_string());
    account.api_server_url = Some("https://server.self-serve.windsurf.com".to_string());

    devin::enrich_account(&mut account).await?;
    ensure_windsurf_email_not_exists(&app, &account.email).await?;
    storage::save_account(&app, &account).await?;

    Ok(account)
}

async fn ensure_windsurf_email_not_exists(app: &AppHandle, email: &str) -> Result<(), String> {
    let email_to_check = email.trim().to_lowercase();
    let existing_accounts = storage::list_accounts(app).await?;
    if existing_accounts
        .iter()
        .any(|account| account.email.trim().to_lowercase() == email_to_check)
    {
        return Err(format!("Account with email '{}' already exists", email));
    }
    Ok(())
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

    if existing_accounts
        .iter()
        .any(|account| account.email.trim().to_lowercase() == email_to_check)
    {
        return Err(format!(
            "Account with email '{}' already exists",
            user_info.email
        ));
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

/// 更新账号（标签等属性）
#[tauri::command]
pub async fn windsurf_update_account(app: AppHandle, account: Account) -> Result<(), String> {
    storage::save_account(&app, &account).await
}

/// 删除账号
#[tauri::command]
pub async fn windsurf_delete_account(app: AppHandle, account_id: String) -> Result<(), String> {
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
) -> Result<SwitchAccountResponse, String> {
    // 1. 加载账号
    let mut acc = storage::load_account(&app, &account_id).await?;

    // 2. 确保 Token 有效
    if acc.is_devin_account() {
        if acc.token.needs_refresh() {
            devin::refresh_session(&mut acc).await?;
            storage::save_account(&app, &acc).await?;
        }
    } else {
        let token = auth::ensure_fresh_token(&acc.token).await?;
        if token.access_token != acc.token.access_token {
            acc.token = token.clone();
            storage::save_account(&app, &acc).await?;
        }
    }

    // 2.1 切号前强制刷新 API Key / 用户名，避免把 state.vscdb 写成已失效的旧 api_key。
    if acc.is_devin_account() {
        if let Err(e) = devin::enrich_account(&mut acc).await {
            eprintln!("Failed to enrich Devin account {}: {}", acc.email, e);
        }
    } else {
        let api_res = api::get_api_key(&acc.token.access_token).await?;
        if api_res.api_key.is_some() {
            acc.api_key = api_res.api_key;
        }
        if api_res.api_server_url.is_some() {
            acc.api_server_url = api_res.api_server_url;
        }
        if api_res.name.is_some() {
            acc.name = api_res.name;
        }
    }
    if acc.api_key.is_none() {
        return Err("Failed to obtain Windsurf api_key".to_string());
    }
    storage::save_account(&app, &acc).await?;

    // 3. 检查 Windsurf 是否运行
    let was_running = process::is_windsurf_running();

    if was_running {
        // 4. 关闭进程
        process::close_windsurf(20)?;

        // 4.0.1 二次验证：等待后再次检查
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        if process::is_windsurf_running() {
            // 重试一次
            process::close_windsurf(10)?;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            if process::is_windsurf_running() {
                return Err(
                    "Windsurf process still running after multiple close attempts".to_string(),
                );
            }
        }
    }

    // 4.1 重置遥测数据（失败不阻断切号）
    match tokio::task::spawn_blocking(|| machine::reset_machine_id()).await {
        Ok(Ok(result)) => {
            println!("Machine ID reset: {}", result.machine_id);
        }
        Ok(Err(e)) => {
            eprintln!("Failed to reset machine/telemetry IDs: {}", e);
        }
        Err(e) => {
            eprintln!("Failed to spawn blocking task: {}", e);
        }
    }

    // 5. 注入 Token
    let db_path = db::get_db_path()?;
    if db_path.exists() {
        let backup_path = db_path.with_extension("vscdb.backup");
        if let Err(e) = tokio::fs::copy(&db_path, &backup_path).await {
            eprintln!("Failed to backup database: {}", e);
        }
    }

    // 注意：参考项目不写入 firebase:authUser:*，只写入 secret:// 和 windsurfAuthStatus
    // 因此这里跳过 inject_token 调用，直接写入 Windsurf 扩展登录状态

    // 5. 写入 Windsurf 扩展登录状态
    let api_key = acc.api_key.clone().ok_or("Missing Windsurf api_key")?;
    let api_server_url = acc
        .api_server_url
        .clone()
        .unwrap_or_else(|| "https://server.self-serve.windsurf.com".to_string());
    let name = acc.name.clone().unwrap_or_else(|| acc.email.clone());
    db::write_windsurf_auth_state(&db_path, &api_key, &api_server_url, &name, &acc.email)?;

    // 6. 更新当前账号
    storage::set_current_account_id(&app, Some(account_id.clone())).await?;

    // 7. 更新最后使用时间
    acc.update_last_used();
    storage::save_account(&app, &acc).await?;

    // 8. 获取自定义路径并启动 Windsurf
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    use crate::core::path_manager::{WINDSURF_CONFIG, read_custom_path_from_config};
    let custom_path = read_custom_path_from_config(&app, &WINDSURF_CONFIG);

    process::launch_windsurf_with_path(custom_path.as_deref())?;

    Ok(SwitchAccountResponse {
        message: format!("Account switched and Windsurf started: {}", acc.email),
    })
}

/// 无感切换账号（deep link + extension.js patch），失败时回退到旧的写库重启流程。
#[tauri::command]
pub async fn windsurf_switch_account_seamless(
    app: AppHandle,
    account_id: String,
) -> Result<SeamlessSwitchAccountResponse, String> {
    emit_switch_progress(&app, "preparing", "开始准备账号信息...", 5, "running");

    match windsurf_switch_account_seamless_inner(&app, &account_id).await {
        Ok(response) => {
            emit_switch_progress(&app, "done", "切号完成", 100, "success");
            Ok(response)
        }
        Err(seamless_error) => {
            eprintln!(
                "Seamless Windsurf switch failed, falling back to direct switch: {}",
                seamless_error
            );
            emit_switch_progress(
                &app,
                "fallback",
                format!("无感切号失败，回退到重启切号: {}", seamless_error),
                88,
                "running",
            );

            let fallback = windsurf_switch_account(app.clone(), account_id.clone()).await?;
            emit_switch_progress(&app, "done", "已通过回退流程完成切号", 100, "success");

            Ok(SeamlessSwitchAccountResponse {
                success: true,
                message: fallback.message,
                fallback_used: true,
                machine_id_reset: false,
                patch_applied: false,
                patch_restarted: false,
            })
        }
    }
}

async fn windsurf_switch_account_seamless_inner(
    app: &AppHandle,
    account_id: &str,
) -> Result<SeamlessSwitchAccountResponse, String> {
    let mut acc = storage::load_account(app, account_id).await?;

    emit_switch_progress(app, "fetch_access", "确保账号 token 可用...", 15, "running");
    if acc.is_devin_account() {
        if acc.token.needs_refresh() {
            devin::refresh_session(&mut acc).await?;
            storage::save_account(app, &acc).await?;
        }
    } else {
        let token = auth::ensure_fresh_token(&acc.token).await?;
        if token.access_token != acc.token.access_token {
            acc.token = token;
            storage::save_account(app, &acc).await?;
        }
    }

    emit_switch_progress(app, "auto_patch", "检查无感切号补丁...", 32, "running");
    use crate::core::path_manager::{WINDSURF_CONFIG, read_custom_path_from_config};
    let custom_path = read_custom_path_from_config(app, &WINDSURF_CONFIG);
    let patch_result = patch::ensure_seamless_patch(app, custom_path.as_deref())?;
    if patch_result.restarted {
        emit_switch_progress(app, "auto_patch", "等待 Windsurf 重启加载补丁...", 45, "running");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }

    emit_switch_progress(app, "reset_mid", "重置机器 ID...", 70, "running");
    let machine_id_reset = match tokio::task::spawn_blocking(machine::reset_machine_id).await {
        Ok(Ok(result)) => {
            println!("Machine ID reset: {}", result.machine_id);
            true
        }
        Ok(Err(e)) => {
            eprintln!("Failed to reset machine/telemetry IDs: {}", e);
            false
        }
        Err(e) => {
            eprintln!("Failed to spawn machine reset task: {}", e);
            false
        }
    };

    emit_switch_progress(app, "cleanup", "清理旧 Windsurf 登录状态...", 76, "running");
    if process::is_windsurf_running() {
        process::close_windsurf(20)?;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    let db_path = db::get_db_path()?;
    if db_path.exists() {
        let backup_path = db_path.with_extension("vscdb.seamless-cleanup.backup");
        if let Err(e) = tokio::fs::copy(&db_path, &backup_path).await {
            eprintln!("Failed to backup database before seamless cleanup: {}", e);
        }
        db::clear_auth_state(&db_path)?;
    }

    emit_switch_progress(app, "launch", "启动 Windsurf 并等待扩展加载...", 78, "running");
    process::launch_windsurf_with_path(custom_path.as_deref())?;
    tokio::time::sleep(std::time::Duration::from_secs(6)).await;

    emit_switch_progress(
        app,
        "fetch_auth",
        "正在获取 one-time auth_token...",
        80,
        "running",
    );
    let auth_token = switcher::get_one_time_auth_token(&acc).await?;

    emit_switch_progress(app, "callback", "触发 Windsurf 登录回调...", 85, "running");
    switcher::trigger_windsurf_callback(&auth_token, "windsurf")?;

    emit_switch_progress(app, "finalize", "保存账号状态...", 95, "running");
    storage::set_current_account_id(app, Some(account_id.to_string())).await?;
    acc.update_last_used();
    storage::save_account(app, &acc).await?;

    Ok(SeamlessSwitchAccountResponse {
        success: true,
        message: if patch_result.already_patched {
            format!("已无感切换到账号 {}", acc.email)
        } else {
            format!("已启用无感切号并切换到账号 {}", acc.email)
        },
        fallback_used: false,
        machine_id_reset,
        patch_applied: patch_result.applied,
        patch_restarted: patch_result.restarted,
    })
}

/// 查询账号配额信息
#[tauri::command]
pub async fn windsurf_fetch_quota(app: AppHandle, account_id: String) -> Result<Account, String> {
    let mut acc = storage::load_account(&app, &account_id).await?;

    // 确保 Token 有效
    if acc.is_devin_account() {
        if acc.token.needs_refresh() {
            devin::refresh_session(&mut acc).await?;
        }
        devin::enrich_account(&mut acc).await?;
        if let Ok(quota_result) = api::get_plan_status_for_account(&acc).await {
            acc.quota = Some(QuotaData {
                plan_name: quota_result.plan_name,
                used_credits: quota_result.used_credits,
                total_credits: quota_result.total_credits,
                usage_percentage: quota_result.usage_percentage,
                expires_at: quota_result.expires_at,
                plan_start: quota_result.plan_start,
                billing_strategy: quota_result.billing_strategy,
                daily_quota_remaining_percent: quota_result.daily_quota_remaining_percent,
                weekly_quota_remaining_percent: quota_result.weekly_quota_remaining_percent,
                daily_quota_reset_at_unix: quota_result.daily_quota_reset_at_unix,
                weekly_quota_reset_at_unix: quota_result.weekly_quota_reset_at_unix,
                overage_balance_micros: quota_result.overage_balance_micros,
                last_updated: chrono::Utc::now().timestamp(),
            });
            acc.updated_at = chrono::Utc::now().timestamp();
        }
        storage::save_account(&app, &acc).await?;
        return Ok(acc);
    } else {
        let token = auth::ensure_fresh_token(&acc.token).await?;
        if token.access_token != acc.token.access_token {
            acc.token = token.clone();
        }
    }

    // 查询配额
    let quota_result = api::get_plan_status_for_account(&acc).await?;

    // 更新账号配额信息
    acc.quota = Some(QuotaData {
        plan_name: quota_result.plan_name,
        used_credits: quota_result.used_credits,
        total_credits: quota_result.total_credits,
        usage_percentage: quota_result.usage_percentage,
        expires_at: quota_result.expires_at,
        plan_start: quota_result.plan_start,
        billing_strategy: quota_result.billing_strategy,
        daily_quota_remaining_percent: quota_result.daily_quota_remaining_percent,
        weekly_quota_remaining_percent: quota_result.weekly_quota_remaining_percent,
        daily_quota_reset_at_unix: quota_result.daily_quota_reset_at_unix,
        weekly_quota_reset_at_unix: quota_result.weekly_quota_reset_at_unix,
        overage_balance_micros: quota_result.overage_balance_micros,
        last_updated: chrono::Utc::now().timestamp(),
    });
    acc.updated_at = chrono::Utc::now().timestamp();

    // 保存更新
    storage::save_account(&app, &acc).await?;

    Ok(acc)
}

/// 批量查询所有账号配额
#[tauri::command]
pub async fn windsurf_fetch_all_quotas(app: AppHandle) -> Result<Vec<Account>, String> {
    let accounts = storage::list_accounts(&app).await?;
    let mut updated_accounts = Vec::new();

    for mut acc in accounts {
        // 确保 Token 有效
        if acc.is_devin_account() {
            match async {
                if acc.token.needs_refresh() {
                    devin::refresh_session(&mut acc).await?;
                }
                devin::enrich_account(&mut acc).await?;
                if let Ok(quota_result) = api::get_plan_status_for_account(&acc).await {
                    acc.quota = Some(QuotaData {
                        plan_name: quota_result.plan_name,
                        used_credits: quota_result.used_credits,
                        total_credits: quota_result.total_credits,
                        usage_percentage: quota_result.usage_percentage,
                        expires_at: quota_result.expires_at,
                        plan_start: quota_result.plan_start,
                        billing_strategy: quota_result.billing_strategy,
                        daily_quota_remaining_percent: quota_result.daily_quota_remaining_percent,
                        weekly_quota_remaining_percent: quota_result.weekly_quota_remaining_percent,
                        daily_quota_reset_at_unix: quota_result.daily_quota_reset_at_unix,
                        weekly_quota_reset_at_unix: quota_result.weekly_quota_reset_at_unix,
                        overage_balance_micros: quota_result.overage_balance_micros,
                        last_updated: chrono::Utc::now().timestamp(),
                    });
                    acc.updated_at = chrono::Utc::now().timestamp();
                }
                Ok::<(), String>(())
            }
            .await
            {
                Ok(()) => {
                    let _ = storage::save_account(&app, &acc).await;
                }
                Err(e) => {
                    eprintln!("Failed to fetch quota for {}: {}", acc.email, e);
                }
            }
        } else {
            match auth::ensure_fresh_token(&acc.token).await {
                Ok(token) => {
                    if token.access_token != acc.token.access_token {
                        acc.token = token.clone();
                    }

                    // 查询配额
                    match api::get_plan_status_for_account(&acc).await {
                        Ok(quota_result) => {
                            acc.quota = Some(QuotaData {
                                plan_name: quota_result.plan_name,
                                used_credits: quota_result.used_credits,
                                total_credits: quota_result.total_credits,
                                usage_percentage: quota_result.usage_percentage,
                                expires_at: quota_result.expires_at,
                                plan_start: quota_result.plan_start,
                                billing_strategy: quota_result.billing_strategy,
                                daily_quota_remaining_percent: quota_result
                                    .daily_quota_remaining_percent,
                                weekly_quota_remaining_percent: quota_result
                                    .weekly_quota_remaining_percent,
                                daily_quota_reset_at_unix: quota_result.daily_quota_reset_at_unix,
                                weekly_quota_reset_at_unix: quota_result.weekly_quota_reset_at_unix,
                                overage_balance_micros: quota_result.overage_balance_micros,
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
        }

        updated_accounts.push(acc);
    }

    Ok(updated_accounts)
}

/// 获取无感切号补丁状态
#[tauri::command]
pub async fn windsurf_get_seamless_patch_status(
    app: AppHandle,
) -> Result<patch::PatchStatus, String> {
    use crate::core::path_manager::{WINDSURF_CONFIG, read_custom_path_from_config};
    let custom_path = read_custom_path_from_config(&app, &WINDSURF_CONFIG);
    Ok(patch::get_patch_status(&app, custom_path.as_deref()))
}

/// 手动应用无感切号补丁
#[tauri::command]
pub async fn windsurf_apply_seamless_patch(
    app: AppHandle,
) -> Result<patch::PatchApplyResult, String> {
    use crate::core::path_manager::{WINDSURF_CONFIG, read_custom_path_from_config};
    let custom_path = read_custom_path_from_config(&app, &WINDSURF_CONFIG);
    patch::ensure_seamless_patch(&app, custom_path.as_deref())
}

/// 还原无感切号补丁
#[tauri::command]
pub async fn windsurf_restore_seamless_patch(
    app: AppHandle,
) -> Result<patch::PatchApplyResult, String> {
    use crate::core::path_manager::{WINDSURF_CONFIG, read_custom_path_from_config};
    let custom_path = read_custom_path_from_config(&app, &WINDSURF_CONFIG);
    patch::restore_seamless_patch(&app, custom_path.as_deref())
}

/// 获取自定义 Windsurf 路径
#[tauri::command]
pub async fn windsurf_get_custom_path(app: AppHandle) -> Result<Option<String>, String> {
    use crate::core::path_manager::{WINDSURF_CONFIG, get_custom_path};
    get_custom_path(&app, &WINDSURF_CONFIG)
}

/// 设置自定义 Windsurf 路径
#[tauri::command]
pub async fn windsurf_set_custom_path(app: AppHandle, path: Option<String>) -> Result<(), String> {
    use crate::core::path_manager::{WINDSURF_CONFIG, set_custom_path};
    set_custom_path(&app, &WINDSURF_CONFIG, path, |p| {
        process::validate_windsurf_path(p)
    })
}

/// 验证 Windsurf 路径
#[tauri::command]
pub async fn windsurf_validate_path(path: String) -> Result<bool, String> {
    process::validate_windsurf_path(&path)
}

/// 获取默认 Windsurf 路径
#[tauri::command]
pub async fn windsurf_get_default_path() -> Result<String, String> {
    process::get_windsurf_executable_path().map(|p| p.to_string_lossy().to_string())
}

/// 打开文件选择对话框选择 Windsurf 可执行文件
#[tauri::command]
pub async fn windsurf_select_executable_path() -> Result<Option<String>, String> {
    use crate::core::path_manager::{WINDSURF_CONFIG, select_executable_path};
    select_executable_path(&WINDSURF_CONFIG)
}
