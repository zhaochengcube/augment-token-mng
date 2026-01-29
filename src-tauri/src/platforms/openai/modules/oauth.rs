use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use crate::http_client::create_proxy_client;
use crate::platforms::openai::models::{
    IdTokenClaims, OpenAIOAuthSession, OpenAITokenInfo, OpenAIUserInfo, TokenResponse,
};

const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const DEFAULT_REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const DEFAULT_SCOPES: &str = "openid profile email offline_access";
const REFRESH_SCOPES: &str = "openid profile email";
const SESSION_TTL_SECS: u64 = 30 * 60;

fn generate_random_bytes(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.r#gen()).collect()
}

pub fn generate_state() -> String {
    hex::encode(generate_random_bytes(32))
}

fn generate_session_id() -> String {
    hex::encode(generate_random_bytes(16))
}

pub fn generate_code_verifier() -> String {
    hex::encode(generate_random_bytes(64))
}

pub fn generate_code_challenge(verifier: &str) -> Result<String, String> {
    let hash = Sha256::digest(verifier.as_bytes());
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(hash))
}

pub fn build_authorization_url(state: &str, code_challenge: &str, redirect_uri: &str) -> Result<String, String> {
    // 按照正确的顺序排列参数，与 Go 实现完全一致
    let params = vec![
        ("client_id", CLIENT_ID),
        ("code_challenge", code_challenge),
        ("code_challenge_method", "S256"),
        ("codex_cli_simplified_flow", "true"),
        ("id_token_add_organizations", "true"),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("scope", DEFAULT_SCOPES),
        ("state", state),
    ];

    // 手动编码查询参数，空格使用 + (与 Go 的 url.Values.Encode() 一致)
    let query_string = params
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                urlencoding::encode(k),
                urlencoding::encode(v).replace("%20", "+")
            )
        })
        .collect::<Vec<_>>()
        .join("&");

    let url = format!("{}?{}", AUTHORIZE_URL, query_string);
    Ok(url)
}

pub fn create_session_and_auth_url(
    redirect_uri: Option<String>,
) -> Result<(String, OpenAIOAuthSession, String), String> {
    let redirect_uri = redirect_uri.unwrap_or_else(|| DEFAULT_REDIRECT_URI.to_string());
    let state = generate_state();
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier)?;
    let session_id = generate_session_id();
    let auth_url = build_authorization_url(&state, &code_challenge, &redirect_uri)?;
    let session = OpenAIOAuthSession {
        state,
        code_verifier,
        redirect_uri,
        created_at: SystemTime::now(),
    };
    Ok((session_id, session, auth_url))
}

pub fn is_session_expired(session: &OpenAIOAuthSession) -> bool {
    session.created_at.elapsed().unwrap_or(Duration::from_secs(0)) > Duration::from_secs(SESSION_TTL_SECS)
}

pub async fn exchange_code(
    code: &str,
    code_verifier: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, String> {
    let client = create_proxy_client()?;
    let mut params = HashMap::new();
    params.insert("grant_type", "authorization_code");
    params.insert("client_id", CLIENT_ID);
    params.insert("code", code);
    params.insert("redirect_uri", redirect_uri);
    params.insert("code_verifier", code_verifier);

    let response = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    if response.status().is_success() {
        response
            .json::<TokenResponse>()
            .await
            .map_err(|e| format!("Failed to parse token response: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Token exchange failed: {}", error_text))
    }
}

pub async fn refresh_token(refresh_token: &str) -> Result<TokenResponse, String> {
    let client = create_proxy_client()?;
    let mut params = HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("refresh_token", refresh_token);
    params.insert("client_id", CLIENT_ID);
    params.insert("scope", REFRESH_SCOPES);

    let response = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Refresh request failed: {}", e))?;

    if response.status().is_success() {
        response
            .json::<TokenResponse>()
            .await
            .map_err(|e| format!("Failed to parse refresh response: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Token refresh failed: {}", error_text))
    }
}

pub fn parse_id_token(id_token: &str) -> Option<OpenAIUserInfo> {
    let payload = id_token.split('.').nth(1)?;
    let decoded = general_purpose::URL_SAFE_NO_PAD.decode(payload.as_bytes()).ok()?;
    let claims: IdTokenClaims = serde_json::from_slice(&decoded).ok()?;
    let mut org_id = None;
    if let Some(auth) = &claims.openai_auth {
        if let Some(orgs) = &auth.organizations {
            if let Some(default_org) = orgs.iter().find(|org| org.is_default.unwrap_or(false)) {
                org_id = default_org.id.clone();
            } else if let Some(first_org) = orgs.first() {
                org_id = first_org.id.clone();
            }
        }
        return Some(OpenAIUserInfo {
            email: claims.email,
            chatgpt_account_id: auth.chatgpt_account_id.clone(),
            chatgpt_user_id: auth.chatgpt_user_id.clone(),
            organization_id: org_id,
        });
    }

    Some(OpenAIUserInfo {
        email: claims.email,
        chatgpt_account_id: None,
        chatgpt_user_id: None,
        organization_id: org_id,
    })
}

pub fn build_token_info(
    token: TokenResponse,
    user_info: Option<OpenAIUserInfo>,
) -> Result<OpenAITokenInfo, String> {
    let now = Utc::now().timestamp();
    let expires_at = now + token.expires_in;
    Ok(OpenAITokenInfo {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        id_token: token.id_token,
        expires_in: token.expires_in,
        expires_at,
        email: user_info.as_ref().and_then(|info| info.email.clone()),
        chatgpt_account_id: user_info.as_ref().and_then(|info| info.chatgpt_account_id.clone()),
        chatgpt_user_id: user_info.as_ref().and_then(|info| info.chatgpt_user_id.clone()),
        organization_id: user_info.as_ref().and_then(|info| info.organization_id.clone()),
    })
}

/// 检查并在需要时刷新 Token
/// 返回最新的 TokenData
pub async fn ensure_fresh_token(
    current_token: &crate::platforms::openai::models::TokenData,
) -> Result<crate::platforms::openai::models::TokenData, String> {
    let now = chrono::Utc::now().timestamp();

    // 如果还有超过 5 分钟有效期，直接返回
    if current_token.expires_at > now + 300 {
        return Ok(current_token.clone());
    }

    // 需要刷新
    let refresh_token_value = current_token.refresh_token.as_ref()
        .ok_or_else(|| "No refresh token available".to_string())?;

    let response = refresh_token(refresh_token_value).await?;

    // 构造新 TokenData
    Ok(crate::platforms::openai::models::TokenData::new(
        response.access_token,
        response.refresh_token.or_else(|| current_token.refresh_token.clone()),
        response.id_token.or_else(|| current_token.id_token.clone()),
        response.expires_in,
        now + response.expires_in,
        response.token_type,
    ))
}

pub fn token_needs_refresh(
    current_token: &crate::platforms::openai::models::TokenData,
    refresh_window_secs: i64,
) -> bool {
    let now = chrono::Utc::now().timestamp();
    current_token.expires_at <= now + refresh_window_secs
}

pub async fn ensure_fresh_token_with_window(
    current_token: &crate::platforms::openai::models::TokenData,
    refresh_window_secs: i64,
) -> Result<crate::platforms::openai::models::TokenData, String> {
    let now = chrono::Utc::now().timestamp();

    if current_token.expires_at > now + refresh_window_secs {
        return Ok(current_token.clone());
    }

    let refresh_token_value = current_token
        .refresh_token
        .as_ref()
        .ok_or_else(|| "No refresh token available".to_string())?;

    let response = refresh_token(refresh_token_value).await?;

    Ok(crate::platforms::openai::models::TokenData::new(
        response.access_token,
        response
            .refresh_token
            .or_else(|| current_token.refresh_token.clone()),
        response.id_token.or_else(|| current_token.id_token.clone()),
        response.expires_in,
        now + response.expires_in,
        response.token_type,
    ))
}
