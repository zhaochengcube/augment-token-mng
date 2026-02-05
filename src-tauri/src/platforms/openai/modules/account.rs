use crate::platforms::openai::models::{Account, AccountType, QuotaData, TokenData};
use crate::platforms::openai::modules::{oauth, quota};

/// 更新账号配额
pub async fn update_account_quota(
    account: &mut Account,
) -> Result<QuotaData, String> {
    // API 账号不支持配额查询
    if account.account_type == AccountType::API {
        return Err("API accounts do not support quota fetching".to_string());
    }

    // 1. 确保 Token 有效
    let current_token = account.token.as_ref()
        .ok_or_else(|| "OAuth account missing token".to_string())?;
    let token = oauth::ensure_fresh_token(current_token).await?;

    if let Some(ref account_token) = account.token {
        if token.access_token != account_token.access_token {
            account.token = Some(token.clone());
            account.updated_at = chrono::Utc::now().timestamp();
        }
    }

    // 2. 查询配额
    let quota_data = quota::fetch_quota(
        &token.access_token,
        account.chatgpt_account_id.as_deref(),
    ).await?;

    // 3. 更新账户配额
    account.update_quota(quota_data.clone());

    Ok(quota_data)
}

/// 带有重试机制的配额查询
pub async fn fetch_quota_with_retry(
    account: &mut Account,
) -> Result<QuotaData, String> {
    println!("=== OpenAI fetch_quota_with_retry ===");
    println!("account.email: {}", account.email);

    // API 账号不支持配额查询
    if account.account_type == AccountType::API {
        return Err("API accounts do not support quota fetching".to_string());
    }

    // 1. 确保Token有效
    let current_token = account.token.as_ref()
        .ok_or_else(|| "OAuth account missing token".to_string())?;
    let token = match oauth::ensure_fresh_token(current_token).await {
        Ok(token) => token,
        Err(e) => {
            if e.contains("invalid_grant") || e.contains("401") {
                return Err(format!("Token refresh failed: {}", e));
            }
            return Err(e);
        }
    };

    if let Some(ref account_token) = account.token {
        if token.access_token != account_token.access_token {
            println!("Token refreshed, updating account");
            account.token = Some(token.clone());
            account.updated_at = chrono::Utc::now().timestamp();
        }
    }

    // 2. 查询配额
    let result = quota::fetch_quota(
        &token.access_token,
        account.chatgpt_account_id.as_deref(),
    ).await;

    // 3. 处理 401 错误 - 尝试强制刷新
    if let Err(ref e) = result {
        if e.contains("401") || e.contains("unauthorized") {
            println!("Got 401 error, trying token refresh...");

            if let Some(ref account_token) = account.token {
                if let Some(refresh_token) = &account_token.refresh_token {
                    let token_res = oauth::refresh_token(refresh_token).await;

                    match token_res {
                        Ok(new_token_res) => {
                            let new_token = crate::platforms::openai::models::TokenData::new(
                                new_token_res.access_token.clone(),
                                new_token_res.refresh_token.clone(),
                                new_token_res.id_token.clone(),
                                new_token_res.expires_in,
                                chrono::Utc::now().timestamp() + new_token_res.expires_in,
                                new_token_res.token_type,
                            );

                            account.token = Some(new_token.clone());
                            account.updated_at = chrono::Utc::now().timestamp();

                            // 重试查询
                            println!("Retrying quota fetch with new token...");
                            return quota::fetch_quota(
                                &new_token.access_token,
                                account.chatgpt_account_id.as_deref(),
                            ).await;
                        }
                        Err(e) => {
                            return Err(format!("Token refresh failed: {}", e));
                        }
                    }
                } else {
                    return Err("No refresh token available".to_string());
                }
            }
        }
    }

    result
}

pub async fn refresh_token_if_needed(
    account: &mut Account,
    refresh_window_secs: i64,
    force: bool,
) -> Result<bool, String> {
    // API 账号不支持 token 刷新
    if account.account_type == AccountType::API {
        return Ok(false);
    }

    let refresh_window_secs = refresh_window_secs.max(0);

    let current_token = account.token.as_ref()
        .ok_or("OAuth account missing token".to_string())?;

    if !force && !oauth::token_needs_refresh(current_token, refresh_window_secs) {
        return Ok(false);
    }

    let account_token = current_token;

    let new_token = if force {
        let refresh_token_value = account_token
            .refresh_token
            .as_ref()
            .ok_or_else(|| "No refresh token available".to_string())?;
        let response = oauth::refresh_token(refresh_token_value).await?;
        let now = chrono::Utc::now().timestamp();
        TokenData::new(
            response.access_token,
            response
                .refresh_token
                .or_else(|| account_token.refresh_token.clone()),
            response.id_token.or_else(|| account_token.id_token.clone()),
            response.expires_in,
            now + response.expires_in,
            response.token_type,
        )
    } else {
        oauth::ensure_fresh_token_with_window(account_token, refresh_window_secs).await?
    };

    if let Some(ref account_token) = account.token {
        if new_token.access_token == account_token.access_token
            && new_token.expires_at == account_token.expires_at
        {
            return Ok(false);
        }
    }

    account.token = Some(new_token);
    account.updated_at = chrono::Utc::now().timestamp();

    Ok(true)
}
