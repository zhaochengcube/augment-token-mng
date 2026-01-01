use tauri::AppHandle;
use crate::antigravity::models::{Account, QuotaData, TokenData};
use crate::antigravity::modules::{storage, account, oauth, oauth_server, process, db};

async fn internal_refresh_account_quota(app: &AppHandle, account: &mut Account) -> Result<QuotaData, String> {
    let quota = account::fetch_quota_with_retry(app, account).await?;
    account::update_account_quota(app, &account.id, quota.clone()).await?;
    Ok(quota)
}

/// 列出所有账号
#[tauri::command]
pub async fn antigravity_list_accounts(app: AppHandle) -> Result<Vec<Account>, String> {
    storage::list_accounts(&app).await
}

/// 添加账号（使用 refresh_token）
#[tauri::command]
pub async fn antigravity_add_account(
    app: AppHandle,
    email: String,
    refresh_token: String,
) -> Result<Account, String> {
    let mut account = account::add_account(&app, email, refresh_token).await?;
    let _ = internal_refresh_account_quota(&app, &mut account).await;
    Ok(account)
}

/// 删除账号
#[tauri::command]
pub async fn antigravity_delete_account(
    app: AppHandle,
    account_id: String,
) -> Result<(), String> {
    account::delete_account(&app, account_id).await
}

/// 获取当前账号
#[tauri::command]
pub async fn antigravity_get_current_account(app: AppHandle) -> Result<Option<Account>, String> {
    let account_id = account::get_current_account_id(&app).await?;
    match account_id {
        Some(id) => Ok(Some(storage::load_account(&app, &id).await?)),
        None => Ok(None),
    }
}

/// 刷新账号配额
#[tauri::command]
pub async fn antigravity_fetch_quota(
    app: AppHandle,
    account_id: String,
) -> Result<QuotaData, String> {
    println!("=== antigravity_fetch_quota ===");
    println!("account_id: {}", account_id);

    let mut acc = storage::load_account(&app, &account_id).await?;
    println!("Loaded account: {}", acc.email);

    let quota = account::fetch_quota_with_retry(&app, &mut acc).await?;
    println!("Fetched quota: {:?}", quota);

    account::update_account_quota(&app, &account_id, quota.clone()).await?;
    println!("Updated account quota");

    Ok(quota)
}

#[derive(serde::Serialize)]
pub struct RefreshStats {
    total: usize,
    success: usize,
    failed: usize,
    details: Vec<String>,
}

/// 刷新所有账号配额
#[tauri::command]
pub async fn antigravity_refresh_all_quotas(app: AppHandle) -> Result<RefreshStats, String> {
    let accounts = storage::list_accounts(&app).await?;
    let mut success = 0;
    let mut failed = 0;
    let mut details = Vec::new();

    for mut account in accounts {
        if account.disabled {
            continue;
        }
        if let Some(ref q) = account.quota {
            if q.is_forbidden {
                continue;
            }
        }

        match internal_refresh_account_quota(&app, &mut account).await {
            Ok(_) => {
                success += 1;
            }
            Err(e) => {
                failed += 1;
                details.push(format!("Account {}: {}", account.email, e));
            }
        }
    }

    Ok(RefreshStats { total: success + failed, success, failed, details })
}

/// 切换账号（完整流程）
#[tauri::command]
pub async fn antigravity_switch_account(
    app: AppHandle,
    account_id: String,
) -> Result<String, String> {
    // 1. 加载账号
    let mut acc = storage::load_account(&app, &account_id).await?;
    
    // 2. 确保 Token 有效
    let token = oauth::ensure_fresh_token(&acc.token).await?;
    if token.access_token != acc.token.access_token {
        acc.token = token.clone();
        storage::save_account(&app, &acc).await?;
    }
    
    // 3. 检查 Antigravity 是否运行
    let was_running = process::is_antigravity_running();
    
    if was_running {
        // 4. 关闭进程（优先主进程）
        process::close_antigravity(20)?;
    }
    
    // 5. 注入 Token（先备份数据库）
    let db_path = db::get_db_path()?;
    if db_path.exists() {
        if let Some(backup_path) = db_path.with_extension("vscdb.backup").to_str() {
            let _ = std::fs::copy(&db_path, backup_path);
        }
    }
    db::inject_token(
        &db_path,
        &acc.token.access_token,
        &acc.token.refresh_token,
        acc.token.expiry_timestamp,
    )?;
    
    // 6. 更新当前账号
    storage::set_current_account_id(&app, Some(account_id.clone())).await?;
    
    // 7. 更新最后使用时间
    acc.update_last_used();
    storage::save_account(&app, &acc).await?;
    
    // 8. 启动 Antigravity
    std::thread::sleep(std::time::Duration::from_secs(1));
    process::launch_antigravity()?;
    Ok(format!(
        "Account switched and Antigravity started: {}",
        acc.email
    ))
}

/// 检查 Antigravity 是否安装
#[tauri::command]
pub async fn antigravity_check_installation() -> Result<bool, String> {
    Ok(db::check_db_exists())
}

/// 检查 Antigravity 是否正在运行
#[tauri::command]
pub async fn antigravity_is_running() -> Result<bool, String> {
    Ok(process::is_antigravity_running())
}

/// 启动 Antigravity
#[tauri::command]
pub async fn antigravity_launch() -> Result<(), String> {
    process::launch_antigravity()
}

/// 获取 OAuth 授权 URL
#[tauri::command]
pub async fn antigravity_get_auth_url(redirect_uri: String) -> Result<String, String> {
    Ok(oauth::get_auth_url(&redirect_uri))
}

/// 使用授权码交换 Token
#[tauri::command]
pub async fn antigravity_exchange_code(
    app: AppHandle,
    code: String,
    redirect_uri: String,
) -> Result<Account, String> {
    // 1. 交换 Token
    let token_res = oauth::exchange_code(&code, &redirect_uri).await?;

    // 2. 获取用户信息
    let user_info = oauth::get_user_info(&token_res.access_token).await?;

    // 3. 创建账号
    let account_id = uuid::Uuid::new_v4().to_string();
    let token = crate::antigravity::models::TokenData::new(
        token_res.access_token,
        token_res.refresh_token.unwrap_or_default(),
        token_res.expires_in,
        Some(user_info.email.clone()),
        None,
        None,
    );

    let mut account = Account::new(account_id, user_info.email.clone(), token);
    account.name = user_info.get_display_name();

    // 4. 保存账号
    storage::save_account(&app, &account).await?;
    let _ = internal_refresh_account_quota(&app, &mut account).await;

    Ok(account)
}

/// 启动 OAuth 授权流程（使用本地服务器）
#[tauri::command]
pub async fn antigravity_start_oauth_login(app: AppHandle) -> Result<Account, String> {
    eprintln!("开始 OAuth 授权流程...");

    // 1. 启动 OAuth 流程获取 Token
    let token_res = oauth_server::start_oauth_flow(app.clone()).await?;

    // 2. 检查 refresh_token
    let refresh_token = token_res.refresh_token.ok_or_else(|| {
        "未获取到 Refresh Token。\n\n\
         可能原因:\n\
         1. 您之前已授权过此应用,Google 不会再次返回 refresh_token\n\n\
         解决方案:\n\
         1. 访问 https://myaccount.google.com/permissions\n\
         2. 撤销 'Antigravity Tools' 的访问权限\n\
         3. 重新进行 OAuth 授权\n\n\
         或者使用 '手动添加' 方式添加账号".to_string()
    })?;

    // 3. 获取用户信息
    let user_info = oauth::get_user_info(&token_res.access_token).await?;
    eprintln!("获取用户信息成功: {}", user_info.email);

    // 4. 创建账号
    let account_id = uuid::Uuid::new_v4().to_string();
    let token = TokenData::new(
        token_res.access_token.clone(),
        refresh_token.clone(),
        token_res.expires_in,
        Some(user_info.email.clone()),
        None,
        None,
    );

    eprintln!("创建 TokenData:");
    eprintln!("  access_token: {}...", &token_res.access_token.chars().take(20).collect::<String>());
    eprintln!("  refresh_token: {}...", &refresh_token.chars().take(20).collect::<String>());
    eprintln!("  expires_in: {}", token_res.expires_in);
    eprintln!("  email: {:?}", user_info.email);

    let mut account = Account::new(account_id, user_info.email.clone(), token);
    account.name = user_info.get_display_name();

    eprintln!("创建 Account:");
    eprintln!("  id: {}", account.id);
    eprintln!("  email: {}", account.email);
    eprintln!("  name: {:?}", account.name);
    eprintln!("  token.refresh_token: {}...", &account.token.refresh_token.chars().take(20).collect::<String>());

    // 5. 保存账号
    eprintln!("正在保存账号信息...");
    storage::save_account(&app, &account).await?;

    eprintln!("账号保存成功！");
    let _ = internal_refresh_account_quota(&app, &mut account).await;

    Ok(account)
}

/// 取消 OAuth 授权流程
#[tauri::command]
pub async fn antigravity_cancel_oauth_login() -> Result<(), String> {
    oauth_server::cancel_oauth_flow();
    Ok(())
}
