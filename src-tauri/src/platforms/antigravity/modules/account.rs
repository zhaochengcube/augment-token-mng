use crate::antigravity::models::{Account, TokenData, QuotaData};
use crate::antigravity::modules::{storage, oauth, quota};
use uuid::Uuid;

/// 添加账号（使用 refresh_token）
pub async fn add_account(
    app_handle: &tauri::AppHandle,
    _email: String,
    refresh_token: String,
) -> Result<Account, String> {
    // 1. 使用 refresh_token 获取 access_token
    let token_res = oauth::refresh_access_token(&refresh_token).await?;
    
    // 2. 获取用户信息
    let user_info = oauth::get_user_info(&token_res.access_token).await?;
    
    // 3. 构造 TokenData
    let token = TokenData::new(
        token_res.access_token,
        refresh_token,
        token_res.expires_in,
        Some(user_info.email.clone()),
        None,
        None,
    );
    
    // 4. 创建账号
    let account_id = Uuid::new_v4().to_string();
    let mut account = Account::new(account_id, user_info.email.clone(), token);
    account.name = user_info.get_display_name();
    
    // 5. 保存账号
    storage::save_account(app_handle, &account).await?;
    
    Ok(account)
}

/// 删除账号
pub async fn delete_account(
    app_handle: &tauri::AppHandle,
    account_id: String,
) -> Result<(), String> {
    let deleted = storage::delete_account(app_handle, &account_id).await?;
    if !deleted {
        return Err(format!("Account not found: {}", account_id));
    }
    Ok(())
}

/// 获取当前账号 ID
pub async fn get_current_account_id(app_handle: &tauri::AppHandle) -> Result<Option<String>, String> {
    storage::get_current_account_id(app_handle).await
}

/// 更新账号配额
pub async fn update_account_quota(
    app_handle: &tauri::AppHandle,
    account_id: &str,
    quota: QuotaData,
) -> Result<(), String> {
    let mut account = storage::load_account(app_handle, account_id).await?;
    account.update_quota(quota);
    storage::save_account(app_handle, &account).await
}

/// 带有重试机制的配额查询
pub async fn fetch_quota_with_retry(
    app_handle: &tauri::AppHandle,
    account: &mut Account,
) -> Result<QuotaData, String> {
    println!("=== fetch_quota_with_retry ===");
    println!("account.email: {}", account.email);

    // 1. 基于时间的检查 - 先确保 Token 有效
    let token = match oauth::ensure_fresh_token(&account.token).await {
        Ok(token) => token,
        Err(e) => {
            if e.contains("invalid_grant") {
                account.disabled = true;
                account.disabled_at = Some(chrono::Utc::now().timestamp());
                account.disabled_reason = Some(format!("invalid_grant: {}", e));
                let _ = storage::save_account(app_handle, account).await;
            }
            return Err(e);
        }
    };

    if token.access_token != account.token.access_token {
        println!("Token refreshed, updating account");
        account.token = token.clone();
        account.updated_at = chrono::Utc::now().timestamp();

        // 重新获取用户名
        let name = if account.name.is_none() || account.name.as_ref().map_or(false, |n| n.trim().is_empty()) {
            match oauth::get_user_info(&token.access_token).await {
                Ok(user_info) => user_info.get_display_name(),
                Err(_) => None
            }
        } else {
            account.name.clone()
        };

        account.name = name;
        storage::save_account(app_handle, account).await?;
    }

    if account.name.is_none() || account.name.as_ref().map_or(false, |n| n.trim().is_empty()) {
        if let Ok(user_info) = oauth::get_user_info(&account.token.access_token).await {
            account.name = user_info.get_display_name();
            storage::save_account(app_handle, account).await?;
        }
    }

    // 2. 尝试查询
    println!("Attempting to fetch quota...");
    let result = quota::fetch_quota(&account.token.access_token).await;

    if let Ok((ref _quota, ref project_id)) = result {
        if project_id.is_some() && *project_id != account.token.project_id {
            account.token.project_id = project_id.clone();
            storage::save_account(app_handle, account).await?;
        }
    }

    // 3. 处理 401 错误
    if let Err(ref e) = result {
        if e.contains("401") {
            println!("Got 401 error, forcing token refresh...");
            // 强制刷新
            let token_res = match oauth::refresh_access_token(&account.token.refresh_token).await {
                Ok(token_res) => token_res,
                Err(e) => {
                    if e.contains("invalid_grant") {
                        account.disabled = true;
                        account.disabled_at = Some(chrono::Utc::now().timestamp());
                        account.disabled_reason = Some(format!("invalid_grant: {}", e));
                        let _ = storage::save_account(app_handle, account).await;
                    }
                    return Err(e);
                }
            };

            let new_token = TokenData::new(
                token_res.access_token.clone(),
                account.token.refresh_token.clone(),
                token_res.expires_in,
                account.token.email.clone(),
                account.token.project_id.clone(),
                None,
            );

            account.token = new_token.clone();
            account.updated_at = chrono::Utc::now().timestamp();
            storage::save_account(app_handle, account).await?;

            // 重试查询
            println!("Retrying quota fetch with new token...");
            let retry_result = quota::fetch_quota(&new_token.access_token).await;
            if let Ok((ref _quota, ref project_id)) = retry_result {
                if project_id.is_some() && *project_id != account.token.project_id {
                    account.token.project_id = project_id.clone();
                    storage::save_account(app_handle, account).await?;
                }
            }
            return retry_result.map(|(q, _)| q);
        }
    }

    result.map(|(q, _)| q)
}
