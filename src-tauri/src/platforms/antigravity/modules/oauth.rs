use crate::antigravity::models::token::{TokenResponse, UserInfo};
use reqwest;

// 使用 Antigravity Manager 的 OAuth 配置
const CLIENT_ID: &str = "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
const CLIENT_SECRET: &str = "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";
const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USER_INFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

/// 生成 OAuth 授权 URL
pub fn get_auth_url(redirect_uri: &str) -> String {
    // 使用 Antigravity 需要的完整 scopes
    let scopes = vec![
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/userinfo.email",
        "https://www.googleapis.com/auth/userinfo.profile",
        "https://www.googleapis.com/auth/cclog",
        "https://www.googleapis.com/auth/experimentsandconfigs"
    ].join(" ");

    let params = vec![
        ("client_id", CLIENT_ID),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("scope", &scopes),
        ("access_type", "offline"),
        ("prompt", "consent"),
        ("include_granted_scopes", "true"),
    ];

    let url = url::Url::parse_with_params(AUTH_URL, &params).expect("Invalid Auth URL");
    url.to_string()
}

/// 使用 Authorization Code 交换 Token
pub async fn exchange_code(code: &str, redirect_uri: &str) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    let response = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    if response.status().is_success() {
        response.json::<TokenResponse>().await
            .map_err(|e| format!("Failed to parse token response: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Token exchange failed: {}", error_text))
    }
}

/// 使用 refresh_token 刷新 access_token
pub async fn refresh_access_token(refresh_token: &str) -> Result<TokenResponse, String> {
    let client = reqwest::Client::new();
    
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let response = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Refresh request failed: {}", e))?;

    if response.status().is_success() {
        let mut token_data = response
            .json::<TokenResponse>()
            .await
            .map_err(|e| format!("Failed to parse refresh response: {}", e))?;
        
        // 刷新时可能不返回新的 refresh_token，保留原有的
        if token_data.refresh_token.is_none() {
            token_data.refresh_token = Some(refresh_token.to_string());
        }
        
        Ok(token_data)
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Refresh failed: {}", error_text))
    }
}

/// 获取用户信息
pub async fn get_user_info(access_token: &str) -> Result<UserInfo, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(USER_INFO_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("User info request failed: {}", e))?;

    if response.status().is_success() {
        response.json::<UserInfo>().await
            .map_err(|e| format!("Failed to parse user info: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Get user info failed: {}", error_text))
    }
}

/// 检查并在需要时刷新 Token
/// 返回最新的 TokenData
pub async fn ensure_fresh_token(
    current_token: &crate::antigravity::models::TokenData,
) -> Result<crate::antigravity::models::TokenData, String> {
    let now = chrono::Utc::now().timestamp();
    
    // 如果还有超过 5 分钟有效期，直接返回
    if current_token.expiry_timestamp > now + 300 {
        return Ok(current_token.clone());
    }
    
    // 需要刷新
    let response = refresh_access_token(&current_token.refresh_token).await?;
    
    // 构造新 TokenData
    Ok(crate::antigravity::models::TokenData::new(
        response.access_token,
        response.refresh_token.unwrap_or(current_token.refresh_token.clone()),
        response.expires_in,
        current_token.email.clone(),
        current_token.project_id.clone(),
        None,
    ))
}

