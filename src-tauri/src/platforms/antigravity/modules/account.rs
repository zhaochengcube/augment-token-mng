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
    storage::save_account(app_handle, &account)?;
    storage::upsert_account_summary(app_handle, &account)?;
    
    Ok(account)
}

/// 删除账号
pub async fn delete_account(
    app_handle: &tauri::AppHandle,
    account_id: String,
) -> Result<(), String> {
    let mut index = storage::load_account_index(app_handle)?;
    
    // 从索引中移除
    let original_len = index.accounts.len();
    index.accounts.retain(|s| s.id != account_id);
    
    if index.accounts.len() == original_len {
        return Err(format!("Account not found: {}", account_id));
    }
    
    // 如果是当前账号，清除当前账号
    if index.current_account_id.as_deref() == Some(&account_id) {
        index.current_account_id = index.accounts.first().map(|s| s.id.clone());
    }
    
    storage::save_account_index(app_handle, &index)?;
    
    // 删除账号文件
    storage::delete_account_file(app_handle, &account_id)?;
    
    Ok(())
}

/// 获取当前账号 ID
pub fn get_current_account_id(app_handle: &tauri::AppHandle) -> Result<Option<String>, String> {
    let index = storage::load_account_index(app_handle)?;
    Ok(index.current_account_id)
}

/// 更新账号配额
pub fn update_account_quota(
    app_handle: &tauri::AppHandle,
    account_id: &str,
    quota: QuotaData,
) -> Result<(), String> {
    let mut account = storage::load_account(app_handle, account_id)?;
    account.update_quota(quota);
    storage::save_account(app_handle, &account)
}

/// 带有重试机制的配额查询
pub async fn fetch_quota_with_retry(
    app_handle: &tauri::AppHandle,
    account: &mut Account,
) -> Result<QuotaData, String> {
    // 1. 基于时间的检查 - 先确保 Token 有效
    let token = oauth::ensure_fresh_token(&account.token).await?;
    
    if token.access_token != account.token.access_token {
        account.token = token.clone();
        
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
        storage::save_account(app_handle, account)?;
    }
    
    // 2. 尝试查询
    let result = quota::fetch_quota(&account.token.access_token).await;
    
    // 3. 处理 401 错误
    if let Err(ref e) = result {
        if e.contains("401") {
            // 强制刷新
            let token_res = oauth::refresh_access_token(&account.token.refresh_token).await?;
            
            let new_token = TokenData::new(
                token_res.access_token.clone(),
                account.token.refresh_token.clone(),
                token_res.expires_in,
                account.token.email.clone(),
                account.token.project_id.clone(),
                None,
            );
            
            account.token = new_token.clone();
            storage::save_account(app_handle, account)?;
            
            // 重试查询
            return quota::fetch_quota(&new_token.access_token).await;
        }
    }
    
    result
}

