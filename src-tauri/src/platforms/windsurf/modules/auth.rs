use crate::http_client::create_proxy_client;
use crate::windsurf::models::token::{FirebaseTokenResponse, FirebaseUserInfo};

// Windsurf 使用的 Firebase API Key
const FIREBASE_API_KEY: &str = "AIzaSyDsOl-1XpT5err0Tcnx8FFod1H8gVGIycY";
const FIREBASE_AUTH_URL: &str = "https://identitytoolkit.googleapis.com/v1/accounts";
const FIREBASE_TOKEN_URL: &str = "https://securetoken.googleapis.com/v1/token";

/// 使用邮箱密码登录 Firebase
pub async fn login_with_email_password(
    email: &str,
    password: &str,
) -> Result<FirebaseTokenResponse, String> {
    let client = create_proxy_client()?;
    
    let url = format!("{}:signInWithPassword?key={}", FIREBASE_AUTH_URL, FIREBASE_API_KEY);
    
    let body = serde_json::json!({
        "email": email,
        "password": password,
        "returnSecureToken": true
    });
    
    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Login request failed: {}", e))?;
    
    if response.status().is_success() {
        response.json::<FirebaseTokenResponse>().await
            .map_err(|e| format!("Failed to parse login response: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        // 解析 Firebase 错误
        if error_text.contains("INVALID_LOGIN_CREDENTIALS") || error_text.contains("INVALID_PASSWORD") {
            Err("Invalid email or password".to_string())
        } else if error_text.contains("EMAIL_NOT_FOUND") {
            Err("Email not found".to_string())
        } else if error_text.contains("USER_DISABLED") {
            Err("Account has been disabled".to_string())
        } else if error_text.contains("TOO_MANY_ATTEMPTS") {
            Err("Too many failed attempts. Please try again later".to_string())
        } else {
            Err(format!("Login failed: {}", error_text))
        }
    }
}

/// 使用 refresh_token 刷新 access_token
pub async fn refresh_access_token(refresh_token: &str) -> Result<FirebaseTokenResponse, String> {
    let client = create_proxy_client()?;
    
    let url = format!("{}?key={}", FIREBASE_TOKEN_URL, FIREBASE_API_KEY);
    
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];
    
    let response = client
        .post(&url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Refresh request failed: {}", e))?;
    
    if response.status().is_success() {
        // Firebase refresh endpoint 返回不同的字段名
        #[derive(serde::Deserialize)]
        struct RefreshResponse {
            id_token: String,
            refresh_token: String,
            expires_in: String,
            user_id: Option<String>,
        }
        
        let refresh_resp: RefreshResponse = response.json().await
            .map_err(|e| format!("Failed to parse refresh response: {}", e))?;
        
        Ok(FirebaseTokenResponse {
            id_token: refresh_resp.id_token,
            refresh_token: refresh_resp.refresh_token,
            expires_in: refresh_resp.expires_in,
            local_id: refresh_resp.user_id,
            email: None,
        })
    } else {
        let error_text = response.text().await.unwrap_or_default();
        if error_text.contains("TOKEN_EXPIRED") || error_text.contains("INVALID_REFRESH_TOKEN") {
            Err("Refresh token expired or invalid".to_string())
        } else {
            Err(format!("Refresh failed: {}", error_text))
        }
    }
}

/// 获取用户信息
pub async fn get_user_info(id_token: &str) -> Result<FirebaseUserInfo, String> {
    let client = create_proxy_client()?;
    
    let url = format!("{}:lookup?key={}", FIREBASE_AUTH_URL, FIREBASE_API_KEY);
    
    let body = serde_json::json!({
        "idToken": id_token
    });
    
    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("User info request failed: {}", e))?;
    
    if response.status().is_success() {
        #[derive(serde::Deserialize)]
        struct UserInfoResponse {
            users: Vec<FirebaseUserInfo>,
        }
        
        let resp: UserInfoResponse = response.json().await
            .map_err(|e| format!("Failed to parse user info response: {}", e))?;
        
        resp.users.into_iter().next()
            .ok_or_else(|| "No user info returned".to_string())
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Get user info failed: {}", error_text))
    }
}

/// 确保 Token 有效（如需要则刷新）
pub async fn ensure_fresh_token(
    current_token: &crate::windsurf::models::TokenData,
) -> Result<crate::windsurf::models::TokenData, String> {
    if !current_token.needs_refresh() {
        return Ok(current_token.clone());
    }
    
    let response = refresh_access_token(&current_token.refresh_token).await?;
    let expires_in = response.expires_in_seconds();

    Ok(crate::windsurf::models::TokenData::new(
        response.id_token,
        response.refresh_token,
        expires_in,
        current_token.email.clone(),
        current_token.user_id.clone(),
    ))
}

