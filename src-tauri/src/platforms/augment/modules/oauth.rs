use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use regex::Regex;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

use crate::http_client::create_proxy_client;
use crate::AppState;
use crate::platforms::augment::models::{
    AugmentOAuthState,
    AugmentTokenResponse,
    ParsedCode,
    TokenFromSessionResponse,
};
use crate::platforms::augment::account::get_models;

const CLIENT_ID: &str = "v";
const AUTH_BASE_URL: &str = "https://auth.augmentcode.com";

#[derive(Debug, Deserialize)]
struct TokenApiResponse {
    access_token: String,
}

/// Base64 URL encode without padding
fn base64_url_encode(data: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

/// Create SHA256 hash
fn sha256_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Generate random bytes
fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.r#gen()).collect()
}

/// Create OAuth state with code verifier, challenge and state
pub fn create_augment_oauth_state() -> AugmentOAuthState {
    let code_verifier_bytes = generate_random_bytes(32);
    let code_verifier = base64_url_encode(&code_verifier_bytes);

    let code_challenge_bytes = sha256_hash(code_verifier.as_bytes());
    let code_challenge = base64_url_encode(&code_challenge_bytes);

    let state_bytes = generate_random_bytes(8);
    let state = base64_url_encode(&state_bytes);

    let creation_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    AugmentOAuthState {
        code_verifier,
        code_challenge,
        state,
        creation_time,
    }
}

/// Generate OAuth authorization URL
pub fn generate_augment_authorize_url(oauth_state: &AugmentOAuthState) -> Result<String, Box<dyn std::error::Error>> {
    let mut url = Url::parse(&format!("{}/authorize", AUTH_BASE_URL))?;

    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("code_challenge", &oauth_state.code_challenge)
        .append_pair("client_id", CLIENT_ID)
        .append_pair("state", &oauth_state.state)
        .append_pair("prompt", "login");

    Ok(url.to_string())
}

/// Parse the authorization code response
pub fn parse_code(code: &str) -> Result<ParsedCode, Box<dyn std::error::Error>> {
    let parsed: ParsedCode = serde_json::from_str(code)?;
    Ok(parsed)
}

/// Get access token using authorization code
pub async fn get_augment_access_token(
    tenant_url: &str,
    code_verifier: &str,
    code: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client().map_err(|e| Box::<dyn std::error::Error>::from(e))?;

    let mut data = HashMap::new();
    data.insert("grant_type", "authorization_code");
    data.insert("client_id", CLIENT_ID);
    data.insert("code_verifier", code_verifier);
    data.insert("redirect_uri", "");
    data.insert("code", code);

    let token_url = format!("{}token", tenant_url);
    let response = client
        .post(&token_url)
        .json(&data)
        .send()
        .await?;

    let token_response: TokenApiResponse = response.json().await?;
    Ok(token_response.access_token)
}

/// Complete OAuth flow and return token with tenant URL
pub async fn complete_augment_oauth_flow(
    oauth_state: &AugmentOAuthState,
    code_input: &str,
) -> Result<AugmentTokenResponse, Box<dyn std::error::Error>> {
    let parsed_code = parse_code(code_input)?;

    let token = get_augment_access_token(
        &parsed_code.tenant_url,
        &oauth_state.code_verifier,
        &parsed_code.code,
    ).await?;

    // 获取用户邮箱
    let email = match get_models(&token, &parsed_code.tenant_url).await {
        Ok(models_response) => Some(models_response.user.email),
        Err(err) => {
            println!("Failed to get user email: {}", err);
            None
        }
    };

    Ok(AugmentTokenResponse {
        access_token: token,
        tenant_url: parsed_code.tenant_url,
        email,
    })
}

/// 从 auth session 中提取 access token
pub async fn extract_token_from_session(session: &str) -> Result<AugmentTokenResponse, String> {
    // 生成 PKCE 参数
    let code_verifier = generate_random_string(32);
    let code_challenge = base64_url_encode(&sha256_hash(code_verifier.as_bytes()));
    let state = generate_random_string(42);
    let client_id = CLIENT_ID;

    // 使用 session 访问 terms-accept 获取 HTML
    let terms_url = format!(
        "{}/terms-accept?response_type=code&code_challenge={}&client_id={}&state={}&prompt=login",
        AUTH_BASE_URL, code_challenge, client_id, state
    );

    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;
    let html_response = client
        .get(&terms_url)
        .header("Cookie", format!("session={}", session))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch terms page: {}", e))?;

    let html = html_response
        .text()
        .await
        .map_err(|e| format!("Failed to read HTML response: {}", e))?;

    // 使用正则表达式提取 code, state, tenant_url
    let code_regex = Regex::new(r#"code:\s*"([^"]+)""#).unwrap();
    let state_regex = Regex::new(r#"state:\s*"([^"]+)""#).unwrap();
    let tenant_url_regex = Regex::new(r#"tenant_url:\s*"([^"]+)""#).unwrap();

    let code = code_regex
        .captures(&html)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .ok_or("SESSION_ERROR_OR_ACCOUNT_BANNED")?;

    let parsed_state = state_regex
        .captures(&html)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .ok_or("SESSION_ERROR_OR_ACCOUNT_BANNED")?;

    let tenant_url = tenant_url_regex
        .captures(&html)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .ok_or("SESSION_ERROR_OR_ACCOUNT_BANNED")?;

    println!("Extracted - code: {}, state: {}, tenant_url: {}", code, parsed_state, tenant_url);

    // 用授权码换 Token
    let token_url = format!("{}token", tenant_url);
    let token_payload = serde_json::json!({
        "grant_type": "authorization_code",
        "client_id": client_id,
        "code_verifier": code_verifier,
        "redirect_uri": "",
        "code": code
    });

    let token_response = client
        .post(&token_url)
        .header("Content-Type", "application/json")
        .json(&token_payload)
        .send()
        .await
        .map_err(|e| format!("Failed to exchange token: {}", e))?;

    let token_data: TokenApiResponse = token_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    // 获取用户邮箱
    let token = token_data.access_token.clone();
    let tenant_url_clone = tenant_url.to_string();

    let email = match get_models(&token, &tenant_url_clone).await {
        Ok(models_response) => Some(models_response.user.email),
        Err(err) => {
            println!("Failed to get user email from session: {}", err);
            None
        }
    };

    Ok(AugmentTokenResponse {
        access_token: token_data.access_token,
        tenant_url: tenant_url.to_string(),
        email,
    })
}

/// 生成随机字符串
fn generate_random_string(length: usize) -> String {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut random_bytes = vec![0u8; length];
    rng.fill_bytes(&mut random_bytes);
    base64_url_encode(&random_bytes)
}

// 内部函数,不发送进度事件,使用缓存的 app_session
pub async fn add_token_from_session_internal_with_cache(
    session: &str,
    _state: &AppState,
) -> Result<TokenFromSessionResponse, String> {
    let token_response = extract_token_from_session(session).await?;

    Ok(TokenFromSessionResponse {
        access_token: token_response.access_token,
        tenant_url: token_response.tenant_url,
        email: token_response.email,
    })
}

// 内部函数,不发送进度事件（保留用于向后兼容）
#[allow(dead_code)]
pub async fn add_token_from_session_internal(session: &str) -> Result<TokenFromSessionResponse, String> {
    let token_response = extract_token_from_session(session).await?;

    Ok(TokenFromSessionResponse {
        access_token: token_response.access_token,
        tenant_url: token_response.tenant_url,
        email: token_response.email,
    })
}


/// 刷新 auth_session（返回新的 session）
pub async fn refresh_auth_session(
    existing_session: &str
) -> Result<String, String> {
    // 1. 生成 OAuth 状态
    let oauth_state = create_augment_oauth_state();

    // 2. 构建 URL
    let url = format!(
        "{}/terms-accept?response_type=code&code_challenge={}&client_id={}&state={}&prompt=login",
        AUTH_BASE_URL,
        oauth_state.code_challenge,
        CLIENT_ID,
        oauth_state.state
    );

    // 3. 发送请求
    let client = create_proxy_client()?;
    let response = client
        .get(&url)
        .header("Cookie", format!("session={}", existing_session))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Failed to refresh session: {}", e))?;

    // 4. 从响应头中提取新 session
    extract_session_from_response(&response)
}

/// 从响应头中提取 session cookie
fn extract_session_from_response(
    response: &reqwest::Response
) -> Result<String, String> {
    let cookies = response.headers().get_all("set-cookie");

    for cookie in cookies {
        if let Ok(cookie_str) = cookie.to_str() {
            // 匹配 session=xxx
            if let Some(start) = cookie_str.find("session=") {
                let session_start = start + 8; // "session=".len()
                let session_end = cookie_str[session_start..]
                    .find(';')
                    .map(|i| session_start + i)
                    .unwrap_or(cookie_str.len());

                let session = &cookie_str[session_start..session_end];
                println!("Extracted session: {}", session);
                return Ok(session.to_string());
            }
        }
    }

    Err("No session cookie found in response".to_string())
}
