use tauri::{AppHandle, Manager, State};
use std::fs;

use crate::AppState;
use crate::platforms::openai::models::{Account, OpenAIAuthUrlResult, OpenAITokenInfo, TokenData};
use crate::platforms::openai::modules::{oauth, storage, account as account_module, oauth_server};

#[tauri::command]
pub async fn openai_generate_auth_url(
    state: State<'_, AppState>,
    redirect_uri: Option<String>,
) -> Result<OpenAIAuthUrlResult, String> {
    let (session_id, session, auth_url) = oauth::create_session_and_auth_url(redirect_uri)?;

    let mut sessions = state.openai_oauth_sessions.lock().unwrap();
    sessions.insert(session_id.clone(), session);

    Ok(OpenAIAuthUrlResult { auth_url, session_id })
}

#[tauri::command]
pub async fn openai_exchange_code(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    code: String,
    redirect_uri: Option<String>,
) -> Result<Account, String> {
    let session = {
        let mut sessions = state.openai_oauth_sessions.lock().unwrap();
        match sessions.get(&session_id) {
            Some(session) if !oauth::is_session_expired(session) => session.clone(),
            _ => {
                sessions.remove(&session_id);
                return Err("session not found or expired".to_string());
            }
        }
    };

    let resolved_redirect_uri = redirect_uri.unwrap_or(session.redirect_uri.clone());
    let token = oauth::exchange_code(&code, &session.code_verifier, &resolved_redirect_uri).await?;
    let user_info = token
        .id_token
        .as_deref()
        .and_then(oauth::parse_id_token);

    {
        let mut sessions = state.openai_oauth_sessions.lock().unwrap();
        sessions.remove(&session_id);
    } // 锁在这里释放

    // 获取邮箱
    let email = user_info.as_ref()
        .and_then(|u| u.email.as_ref())
        .ok_or_else(|| "Failed to get email from token".to_string())?;

    // 获取 chatgpt_account_id
    let chatgpt_account_id = user_info.as_ref()
        .and_then(|u| u.chatgpt_account_id.clone());

    // 加载现有账号用于去重检查
    let existing_accounts = storage::list_accounts(&app).await.unwrap_or_default();

    println!("=== OpenAI Exchange Code ===");
    println!("Email: {}", email);
    println!("ChatGPT Account ID: {:?}", chatgpt_account_id);

    // 检查账号是否已存在（邮箱和 account_id 都相同）
    if Account::is_duplicate(&email, &chatgpt_account_id, &existing_accounts) {
        return Err("该账号已存在".to_string());
    }

    // 生成唯一的邮箱（相同邮箱不同 account_id 时添加序号）
    let unique_email = Account::generate_unique_email(&email, &chatgpt_account_id, &existing_accounts);
    println!("Unique Email: {}", unique_email);

    // 构造 TokenData
    let now = chrono::Utc::now().timestamp();
    let token_data = TokenData::new(
        token.access_token.clone(),
        token.refresh_token,
        token.id_token.clone(),
        token.expires_in,
        now + token.expires_in,
        token.token_type,
    );

    // 创建账号
    let account = Account::new_oauth(
        unique_email,
        token_data,
        chatgpt_account_id.clone(),
        user_info.as_ref().and_then(|u| u.chatgpt_user_id.clone()),
        user_info.as_ref().and_then(|u| u.organization_id.clone()),
    );

    // 保存账号到本地
    storage::save_account(&app, &account).await?;

    Ok(account)
}

#[tauri::command]
pub async fn openai_refresh_token(
    refresh_token: String,
) -> Result<OpenAITokenInfo, String> {
    let token = oauth::refresh_token(&refresh_token).await?;
    let user_info = token
        .id_token
        .as_deref()
        .and_then(oauth::parse_id_token);
    oauth::build_token_info(token, user_info)
}

/// 列出所有 OpenAI 账号
#[tauri::command]
pub async fn openai_list_accounts(app: AppHandle) -> Result<Vec<Account>, String> {
    storage::list_accounts(&app).await
}

/// 加载单个 OpenAI 账号
#[tauri::command]
pub async fn openai_load_account(
    app: AppHandle,
    account_id: String,
) -> Result<Account, String> {
    storage::load_account(&app, &account_id).await
}

/// 保存 OpenAI 账号
#[tauri::command]
pub async fn openai_save_account(
    app: AppHandle,
    account: Account,
) -> Result<(), String> {
    storage::save_account(&app, &account).await
}

/// 删除 OpenAI 账号
#[tauri::command]
pub async fn openai_delete_account(
    app: AppHandle,
    account_id: String,
) -> Result<bool, String> {
    storage::delete_account(&app, &account_id).await
}

/// 获取当前 OpenAI 账号 ID
#[tauri::command]
pub async fn openai_get_current_account_id(app: AppHandle) -> Result<Option<String>, String> {
    storage::get_current_account_id(&app).await
}

/// 设置当前 OpenAI 账号 ID
#[tauri::command]
pub async fn openai_set_current_account_id(
    app: AppHandle,
    account_id: Option<String>,
) -> Result<(), String> {
    storage::set_current_account_id(&app, account_id).await
}

/// 刷新账号配额，返回更新后的完整账户数据
#[tauri::command]
pub async fn openai_fetch_quota(
    app: AppHandle,
    account_id: String,
) -> Result<Account, String> {
    println!("=== openai_fetch_quota ===");
    println!("account_id: {}", account_id);

    let mut acc = storage::load_account(&app, &account_id).await?;
    println!("Loaded account: {}", acc.email);

    // API 账号不支持配额查询
    if acc.account_type == crate::platforms::openai::models::account::AccountType::API {
        return Err("API accounts do not support quota fetching".to_string());
    }

    let quota = account_module::fetch_quota_with_retry(&mut acc).await?;
    println!("Fetched quota: {:?}", quota);

    // 更新账户配额
    acc.update_quota(quota);

    // 保存账户
    storage::save_account(&app, &acc).await?;
    println!("Updated account quota");

    // 重新加载账户以获取更新后的完整数据
    let updated_acc = storage::load_account(&app, &account_id).await?;
    Ok(updated_acc)
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
pub async fn openai_refresh_all_quotas(app: AppHandle) -> Result<RefreshStats, String> {
    let accounts = storage::list_accounts(&app).await?;
    let mut success = 0;
    let mut failed = 0;
    let mut details = Vec::new();

    for mut account in accounts {
        match account_module::fetch_quota_with_retry(&mut account).await {
            Ok(quota) => {
                account.update_quota(quota);
                if let Err(e) = storage::save_account(&app, &account).await {
                    failed += 1;
                    details.push(format!("Account {}: Failed to save: {}", account.email, e));
                } else {
                    success += 1;
                }
            }
            Err(e) => {
                failed += 1;
                details.push(format!("Account {}: {}", account.email, e));
            }
        }
    }

    Ok(RefreshStats { total: success + failed, success, failed, details })
}

#[derive(serde::Serialize)]
pub struct TokenRefreshResult {
    pub account: Account,
    pub refreshed: bool,
}

#[derive(serde::Serialize)]
pub struct TokenRefreshStats {
    total: usize,
    refreshed: usize,
    skipped: usize,
    failed: usize,
    details: Vec<String>,
}

#[tauri::command]
pub async fn openai_refresh_account_token(
    app: AppHandle,
    account_id: String,
    refresh_window_secs: Option<i64>,
    force: Option<bool>,
) -> Result<TokenRefreshResult, String> {
    let mut account = storage::load_account(&app, &account_id).await?;

    // API 账号不支持 token 刷新
    if account.account_type == crate::platforms::openai::models::account::AccountType::API {
        return Ok(TokenRefreshResult {
            account,
            refreshed: false,
        });
    }

    let window_secs = refresh_window_secs.unwrap_or(300);
    let force = force.unwrap_or(false);

    let refreshed = account_module::refresh_token_if_needed(&mut account, window_secs, force).await?;
    if refreshed {
        storage::save_account(&app, &account).await?;
    }

    let updated = storage::load_account(&app, &account_id).await?;
    Ok(TokenRefreshResult { account: updated, refreshed })
}

#[tauri::command]
pub async fn openai_refresh_all_tokens(
    app: AppHandle,
    refresh_window_secs: Option<i64>,
    force: Option<bool>,
) -> Result<TokenRefreshStats, String> {
    use crate::platforms::openai::models::account::AccountType;

    let mut accounts = storage::list_accounts(&app).await?;
    let window_secs = refresh_window_secs.unwrap_or(300);
    let force = force.unwrap_or(false);

    // 过滤掉 API 账号
    accounts.retain(|a| a.account_type != AccountType::API);

    let mut refreshed = 0;
    let mut skipped = 0;
    let mut failed = 0;
    let mut details = Vec::new();

    for mut account in accounts {
        match account_module::refresh_token_if_needed(&mut account, window_secs, force).await {
            Ok(true) => {
                if let Err(e) = storage::save_account(&app, &account).await {
                    failed += 1;
                    details.push(format!("Account {}: Failed to save: {}", account.email, e));
                } else {
                    refreshed += 1;
                }
            }
            Ok(false) => {
                skipped += 1;
            }
            Err(e) => {
                failed += 1;
                details.push(format!("Account {}: {}", account.email, e));
            }
        }
    }

    Ok(TokenRefreshStats {
        total: refreshed + skipped + failed,
        refreshed,
        skipped,
        failed,
        details,
    })
}

/// 直接从本地文件加载账号（返回 JSON 字符串）
#[tauri::command]
pub async fn openai_load_accounts_json(app: AppHandle) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let storage_path = app_data_dir.join("openai_accounts.json");

    if storage_path.exists() {
        let content = fs::read_to_string(&storage_path)
            .map_err(|e| format!("Failed to read accounts file: {}", e))?;

        if content.trim().is_empty() {
            return Ok(r#"{"accounts":[],"current_account_id":null}"#.to_string());
        }

        Ok(content)
    } else {
        Ok(r#"{"accounts":[],"current_account_id":null}"#.to_string())
    }
}

/// 添加账号（使用 refresh_token）
#[tauri::command]
pub async fn openai_add_account(
    app: AppHandle,
    email: String,
    refresh_token: String,
) -> Result<Account, String> {
    // 1. 使用 refresh_token 获取 access_token
    let token_res = oauth::refresh_token(&refresh_token).await?;
    let user_info = token_res
        .id_token
        .as_deref()
        .and_then(oauth::parse_id_token);

    // 获取 chatgpt_account_id
    let chatgpt_account_id = user_info.as_ref()
        .and_then(|u| u.chatgpt_account_id.clone());

    // 加载现有账号用于去重检查
    let existing_accounts = storage::list_accounts(&app).await.unwrap_or_default();

    println!("=== OpenAI Add Account ===");
    println!("Email: {}", email);
    println!("ChatGPT Account ID: {:?}", chatgpt_account_id);

    // 检查账号是否已存在（邮箱和 account_id 都相同）
    if Account::is_duplicate(&email, &chatgpt_account_id, &existing_accounts) {
        return Err("该账号已存在".to_string());
    }

    // 生成唯一的邮箱（相同邮箱不同 account_id 时添加序号）
    let unique_email = Account::generate_unique_email(&email, &chatgpt_account_id, &existing_accounts);
    println!("Unique Email: {}", unique_email);

    // 2. 构造 TokenData
    let now = chrono::Utc::now().timestamp();
    let token = TokenData::new(
        token_res.access_token.clone(),
        Some(refresh_token),
        token_res.id_token.clone(),
        token_res.expires_in,
        now + token_res.expires_in,
        token_res.token_type,
    );

    // 3. 创建账号
    let account = Account::new_oauth(
        unique_email,
        token,
        chatgpt_account_id.clone(),
        user_info.as_ref().and_then(|u| u.chatgpt_user_id.clone()),
        user_info.as_ref().and_then(|u| u.organization_id.clone()),
    );

    // 4. 保存账号
    storage::save_account(&app, &account).await?;

    Ok(account)
}

/// 添加 API 类型账号
#[tauri::command]
pub async fn openai_add_api_account(
    app: AppHandle,
    model_provider: String,
    model: String,
    reasoning_effort: Option<String>,
    wire_api: Option<String>,
    base_url: String,
    key: String,
) -> Result<Account, String> {
    use crate::platforms::openai::models::account::{Account, ApiConfig};

    // wire_api 默认值统一在此处处理
    let wire_api_value = wire_api
        .filter(|w| !w.is_empty())
        .unwrap_or_else(|| "responses".to_string());

    let api_config = ApiConfig {
        model_provider: if model_provider.is_empty() { None } else { Some(model_provider) },
        model: if model.is_empty() { None } else { Some(model) },
        model_reasoning_effort: reasoning_effort.filter(|s| !s.is_empty()),
        wire_api: Some(wire_api_value),
        base_url: if base_url.is_empty() { None } else { Some(base_url) },
        key: if key.is_empty() { None } else { Some(key) },
    };

    // 生成唯一 ID
    let id = format!("api_{}", chrono::Utc::now().timestamp_millis());

    // 使用 model_provider 作为邮箱显示名称
    let email = api_config.model_provider.as_deref().unwrap_or("API Account").to_string();

    let account = Account::new_api(id, email, api_config);

    // 保存账号
    storage::save_account(&app, &account).await?;

    Ok(account)
}

/// 更新 API 类型账号
#[tauri::command]
pub async fn openai_update_api_account(
    app: AppHandle,
    account_id: String,
    model_provider: String,
    model: String,
    reasoning_effort: Option<String>,
    wire_api: Option<String>,
    base_url: String,
    key: String,
) -> Result<Account, String> {
    use crate::platforms::openai::models::account::{AccountType, ApiConfig};

    // 加载现有账号
    let mut account = storage::load_account(&app, &account_id).await?;

    // 确保是 API 类型账号
    if account.account_type != AccountType::API {
        return Err("Only API accounts can be updated with this command".to_string());
    }

    // wire_api 默认值
    let wire_api_value = wire_api
        .filter(|w| !w.is_empty())
        .unwrap_or_else(|| "responses".to_string());

    // 更新 API 配置
    let api_config = ApiConfig {
        model_provider: if model_provider.is_empty() { None } else { Some(model_provider.clone()) },
        model: if model.is_empty() { None } else { Some(model) },
        model_reasoning_effort: reasoning_effort.filter(|s| !s.is_empty()),
        wire_api: Some(wire_api_value),
        base_url: if base_url.is_empty() { None } else { Some(base_url) },
        key: if key.is_empty() { None } else { Some(key) },
    };

    // 更新账号的 api_config 和 email
    account.api_config = Some(api_config);
    account.email = if !model_provider.is_empty() { model_provider } else { "API Account".to_string() };
    account.updated_at = chrono::Utc::now().timestamp();

    // 保存账号
    storage::save_account(&app, &account).await?;

    Ok(account)
}

/// 保存多个账号
#[tauri::command]
pub async fn openai_save_accounts(
    app: AppHandle,
    accounts: Vec<Account>,
) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let storage_path = app_data_dir.join("openai_accounts.json");

    // 获取 current_account_id
    let current_id = storage::get_current_account_id(&app).await?;

    let data = serde_json::json!({
        "accounts": accounts,
        "current_account_id": current_id
    });

    let content = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("Failed to serialize accounts: {}", e))?;

    fs::write(&storage_path, content)
        .map_err(|e| format!("Failed to write accounts file: {}", e))?;

    Ok(())
}

/// 更新账号
#[tauri::command]
pub async fn openai_update_account(
    app: AppHandle,
    account: Account,
) -> Result<(), String> {
    storage::save_account(&app, &account).await
}

/// 刷新账号（刷新 token）
#[tauri::command]
pub async fn openai_refresh_account(
    app: AppHandle,
    account_id: String,
) -> Result<Account, String> {
    let mut acc = storage::load_account(&app, &account_id).await?;

    // API 账号不支持刷新 token
    if acc.account_type == crate::platforms::openai::models::account::AccountType::API {
        return Err("API accounts do not support token refresh".to_string());
    }

    println!("=== OpenAI Refresh Account ===");
    println!("Email: {}", acc.email);
    println!("ChatGPT Account ID: {:?}", acc.chatgpt_account_id);

    // 刷新 token
    if let Some(ref token) = acc.token {
        if let Some(refresh_token) = &token.refresh_token {
            let token_res = oauth::refresh_token(refresh_token).await?;

            let now = chrono::Utc::now().timestamp();
            acc.token = Some(TokenData::new(
                token_res.access_token,
                token_res.refresh_token,
                token_res.id_token,
                token_res.expires_in,
                now + token_res.expires_in,
                token_res.token_type,
            ));
            acc.updated_at = now;

            // 解析新的 id_token 获取 chatgpt_account_id
            if let Some(id_token) = &acc.token.as_ref().and_then(|t| t.id_token.as_ref()) {
                if let Some(user_info) = oauth::parse_id_token(id_token) {
                    println!("New ChatGPT Account ID from refresh: {:?}", user_info.chatgpt_account_id);
                    if let Some(new_chatgpt_id) = user_info.chatgpt_account_id {
                        acc.chatgpt_account_id = Some(new_chatgpt_id);
                    }
                }
            }

            storage::save_account(&app, &acc).await?;
        }
    }

    Ok(acc)
}

/// 启动 OAuth 授权流程（使用本地服务器自动处理回调）
#[tauri::command]
pub async fn openai_start_oauth_login(app: AppHandle) -> Result<Account, String> {
    println!("开始 OpenAI OAuth 授权流程...");

    // 启动 OAuth 流程获取账号
    let mut account = oauth_server::start_oauth_flow(app.clone()).await?;

    println!("=== OpenAI OAuth Login ===");
    println!("Email: {}", account.email);
    println!("ChatGPT Account ID: {:?}", account.chatgpt_account_id);

    // 加载现有账号用于去重检查
    let existing_accounts = storage::list_accounts(&app).await.unwrap_or_default();

    // 检查账号是否已存在（邮箱和 account_id 都相同）
    if Account::is_duplicate(&account.email, &account.chatgpt_account_id, &existing_accounts) {
        return Err("该账号已存在".to_string());
    }

    // 生成唯一的邮箱（相同邮箱不同 account_id 时添加序号）
    let unique_email = Account::generate_unique_email(&account.email, &account.chatgpt_account_id, &existing_accounts);

    // 如果邮箱需要修改（添加了序号），则更新
    if unique_email != account.email {
        println!("邮箱去重: {} -> {}", account.email, unique_email);
        account.email = unique_email;
    }

    // 保存账号到本地
    storage::save_account(&app, &account).await?;

    println!("OpenAI OAuth 授权完成，账号已保存: {}", account.email);

    Ok(account)
}

/// 取消 OAuth 授权流程
#[tauri::command]
pub async fn openai_cancel_oauth_login() -> Result<(), String> {
    oauth_server::cancel_oauth_flow();
    Ok(())
}

/// 切换 OpenAI 账号（写入 .codex/auth.json 并更新 current_account_id）
#[tauri::command]
pub async fn openai_switch_account(app: AppHandle, account_id: String) -> Result<(), String> {
    use crate::platforms::openai::models::account::AccountType;

    // 加载账号信息
    let account = storage::load_account(&app, &account_id).await?;

    // 获取用户主目录并构造 .codex 路径
    let home_dir = app.path().home_dir()
        .map_err(|e| format!("Failed to get home directory: {}", e))?;

    let codex_dir = home_dir.join(".codex");
    let auth_file = codex_dir.join("auth.json");
    let config_file = codex_dir.join("config.toml");

    // 确保目录存在
    std::fs::create_dir_all(&codex_dir)
        .map_err(|e| format!("Failed to create .codex directory: {}", e))?;

    if account.account_type == AccountType::API {
        // ========== API 账号切换 ==========
        let api_config = account.api_config.as_ref()
            .ok_or("API account missing configuration".to_string())?;

        // 1. 处理 config.toml
        let mut config: toml::Table = if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)
                .map_err(|e| format!("Failed to read config.toml: {}", e))?;
            toml::from_str(&content)
                .map_err(|e| format!("Failed to parse config.toml: {}", e))?
        } else {
            toml::Table::new()
        };

        // 设置顶层字段
        let provider_name = api_config.model_provider.clone()
            .ok_or("API account missing model_provider".to_string())?;
        let model_name = api_config.model.clone()
            .ok_or("API account missing model".to_string())?;
        let base_url = api_config.base_url.clone()
            .ok_or("API account missing base_url".to_string())?;
        let wire_api = api_config.wire_api.clone()
            .unwrap_or_else(|| "responses".to_string());

        config.insert("model_provider".to_string(), toml::Value::String(provider_name.clone()));
        config.insert("model".to_string(), toml::Value::String(model_name));
        
        // model_reasoning_effort 可选
        if let Some(ref reasoning_effort) = api_config.model_reasoning_effort {
            config.insert("model_reasoning_effort".to_string(), toml::Value::String(reasoning_effort.clone()));
        }

        // 设置 [model_providers.xxx] section
        let model_providers = config.entry("model_providers".to_string())
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        
        if let toml::Value::Table(providers) = model_providers {
            let mut provider_config = toml::Table::new();
            provider_config.insert("name".to_string(), toml::Value::String(provider_name.clone()));
            provider_config.insert("base_url".to_string(), toml::Value::String(base_url));
            provider_config.insert("wire_api".to_string(), toml::Value::String(wire_api));
            providers.insert(provider_name, toml::Value::Table(provider_config));
        }

        let config_content = toml::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config.toml: {}", e))?;
        std::fs::write(&config_file, config_content)
            .map_err(|e| format!("Failed to write config.toml: {}", e))?;

        // 2. 处理 auth.json
        let mut auth: serde_json::Map<String, serde_json::Value> = if auth_file.exists() {
            let content = std::fs::read_to_string(&auth_file)
                .map_err(|e| format!("Failed to read auth.json: {}", e))?;
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse auth.json: {}", e))?
        } else {
            serde_json::Map::new()
        };

        // 设置 API Key，删除 tokens
        let api_key = api_config.key.clone()
            .ok_or("API account missing key".to_string())?;
        auth.insert("OPENAI_API_KEY".to_string(), serde_json::Value::String(api_key));
        auth.remove("tokens");

        let auth_content = serde_json::to_string_pretty(&auth)
            .map_err(|e| format!("Failed to serialize auth.json: {}", e))?;
        std::fs::write(&auth_file, auth_content)
            .map_err(|e| format!("Failed to write auth.json: {}", e))?;

    } else {
        // ========== OAuth 账号切换 ==========
        let token = account.token.as_ref()
            .ok_or("OAuth account missing token".to_string())?;

        // 1. 处理 auth.json
        let auth_json = serde_json::json!({
            "OPENAI_API_KEY": null,
            "tokens": {
                "id_token": token.id_token.clone(),
                "access_token": token.access_token.clone(),
                "refresh_token": token.refresh_token.clone(),
                "account_id": account.chatgpt_account_id
            },
            "last_refresh": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        });

        let auth_content = serde_json::to_string_pretty(&auth_json)
            .map_err(|e| format!("Failed to serialize auth.json: {}", e))?;
        std::fs::write(&auth_file, auth_content)
            .map_err(|e| format!("Failed to write auth.json: {}", e))?;

        // 2. 处理 config.toml - 删除 API 相关字段
        if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)
                .map_err(|e| format!("Failed to read config.toml: {}", e))?;
            let mut config: toml::Table = toml::from_str(&content)
                .map_err(|e| format!("Failed to parse config.toml: {}", e))?;

            // 删除 API 相关字段
            config.remove("model_provider");
            config.remove("model");
            config.remove("model_reasoning_effort");
            config.remove("model_providers");

            let config_content = toml::to_string_pretty(&config)
                .map_err(|e| format!("Failed to serialize config.toml: {}", e))?;
            std::fs::write(&config_file, config_content)
                .map_err(|e| format!("Failed to write config.toml: {}", e))?;
        }
    }

    // 更新 current_account_id
    storage::set_current_account_id(&app, Some(account_id.clone())).await?;

    println!("OpenAI account switched: {} (type: {:?})", account.email, account.account_type);

    Ok(())
}

