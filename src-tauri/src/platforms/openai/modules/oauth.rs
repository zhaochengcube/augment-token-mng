use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use rand::Rng;
use serde_json::{Map, Value};
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
const CHATGPT_ACCOUNTS_CHECK_URL: &str =
    "https://chatgpt.com/backend-api/accounts/check/v4-2023-04-27";

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

pub fn build_authorization_url(
    state: &str,
    code_challenge: &str,
    redirect_uri: &str,
) -> Result<String, String> {
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
    session
        .created_at
        .elapsed()
        .unwrap_or(Duration::from_secs(0))
        > Duration::from_secs(SESSION_TTL_SECS)
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
    let decoded = general_purpose::URL_SAFE_NO_PAD
        .decode(payload.as_bytes())
        .ok()?;
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

/// 提取 id_token 中的 OpenAI auth 对象，返回 JSON 字符串
pub fn extract_openai_auth_json(id_token: &str) -> Option<String> {
    let payload = id_token.split('.').nth(1)?;
    let decoded = general_purpose::URL_SAFE_NO_PAD
        .decode(payload.as_bytes())
        .ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    let auth_obj = claims.get("https://api.openai.com/auth")?.as_object()?;
    serde_json::to_string(auth_obj).ok()
}

fn decode_jwt_payload(jwt: &str) -> Option<Value> {
    let payload = jwt.split('.').nth(1)?;
    let decoded = general_purpose::URL_SAFE_NO_PAD
        .decode(payload.as_bytes())
        .or_else(|_| {
            let mut padded = payload.to_string();
            let remainder = padded.len() % 4;
            if remainder != 0 {
                padded.push_str(&"=".repeat(4 - remainder));
            }
            general_purpose::URL_SAFE.decode(padded.as_bytes())
        })
        .ok()?;
    serde_json::from_slice(&decoded).ok()
}

fn organization_id_from_auth(auth: &Value) -> Option<String> {
    if let Some(poid) = auth
        .get("poid")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        return Some(poid.to_string());
    }

    let orgs = auth.get("organizations")?.as_array()?;
    let selected = orgs
        .iter()
        .find(|org| {
            org.get("is_default")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .or_else(|| orgs.first())?;
    selected.get("id")?.as_str().map(ToOwned::to_owned)
}

#[derive(Debug, Clone)]
pub struct OpenAIAccessTokenImport {
    pub access_token: String,
    pub email: Option<String>,
    pub chatgpt_account_id: Option<String>,
    pub chatgpt_user_id: Option<String>,
    pub organization_id: Option<String>,
    pub openai_auth_json: Option<String>,
    pub expires_at: Option<i64>,
}

fn trimmed_value_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(ToOwned::to_owned)
}

fn set_auth_json_string(auth: &mut Map<String, Value>, key: &str, value: &Option<String>) {
    if let Some(value) = value.as_ref().map(|v| v.trim()).filter(|v| !v.is_empty()) {
        auth.insert(key.to_string(), Value::String(value.to_string()));
    }
}

fn extract_auth_map_from_claims(claims: &Value) -> Map<String, Value> {
    claims
        .get("https://api.openai.com/auth")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

#[derive(Debug, Clone, Default)]
struct AccountCheckEnrichment {
    plan_type: Option<String>,
    subscription_expires_at: Option<String>,
}

fn extract_account_plan_type(account: &Map<String, Value>) -> Option<String> {
    account
        .get("account")
        .and_then(Value::as_object)
        .and_then(|v| trimmed_value_string(v.get("plan_type")))
        .or_else(|| {
            account
                .get("entitlement")
                .and_then(Value::as_object)
                .and_then(|v| trimmed_value_string(v.get("subscription_plan")))
        })
}

fn extract_account_subscription_expires_at(account: &Map<String, Value>) -> Option<String> {
    account
        .get("entitlement")
        .and_then(Value::as_object)
        .and_then(|v| v.get("expires_at"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn build_account_check_enrichment(account: &Map<String, Value>) -> AccountCheckEnrichment {
    AccountCheckEnrichment {
        plan_type: extract_account_plan_type(account),
        subscription_expires_at: extract_account_subscription_expires_at(account),
    }
}

fn extract_account_check_enrichment(
    value: &Value,
    preferred_account_ids: &[Option<&str>],
) -> AccountCheckEnrichment {
    let Some(accounts) = value.get("accounts").and_then(Value::as_object) else {
        return AccountCheckEnrichment::default();
    };

    for account_id in preferred_account_ids
        .iter()
        .filter_map(|v| v.map(str::trim))
        .filter(|v| !v.is_empty())
    {
        if let Some(account) = accounts.get(account_id).and_then(Value::as_object) {
            let enrichment = build_account_check_enrichment(account);
            if enrichment.plan_type.is_some() {
                return enrichment;
            }
        }
    }

    let mut default_candidate = AccountCheckEnrichment::default();
    let mut paid_candidate = AccountCheckEnrichment::default();
    let mut any_candidate = AccountCheckEnrichment::default();

    for account in accounts.values().filter_map(Value::as_object) {
        let enrichment = build_account_check_enrichment(account);
        let Some(plan_type) = enrichment.plan_type.as_deref() else {
            continue;
        };

        if any_candidate.plan_type.is_none() {
            any_candidate = enrichment.clone();
        }

        let is_default = account
            .get("account")
            .and_then(Value::as_object)
            .and_then(|v| v.get("is_default"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if is_default {
            default_candidate = enrichment.clone();
        }

        if !plan_type.eq_ignore_ascii_case("free") && paid_candidate.plan_type.is_none() {
            paid_candidate = enrichment;
        }
    }

    if default_candidate.plan_type.is_some() {
        default_candidate
    } else if paid_candidate.plan_type.is_some() {
        paid_candidate
    } else {
        any_candidate
    }
}

fn merge_account_check_enrichment_into_auth_json(
    openai_auth_json: &mut Option<String>,
    enrichment: AccountCheckEnrichment,
) {
    if enrichment.plan_type.is_none() && enrichment.subscription_expires_at.is_none() {
        return;
    }

    let mut auth_json = openai_auth_json
        .as_deref()
        .and_then(|v| serde_json::from_str::<Value>(v).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();

    set_auth_json_string(&mut auth_json, "chatgpt_plan_type", &enrichment.plan_type);
    set_auth_json_string(
        &mut auth_json,
        "chatgpt_subscription_active_until",
        &enrichment.subscription_expires_at,
    );

    if !auth_json.is_empty() {
        *openai_auth_json = serde_json::to_string(&auth_json).ok();
    }
}

pub async fn enrich_openai_auth_json_with_account_check(
    access_token: &str,
    organization_id: Option<&str>,
    chatgpt_account_id: Option<&str>,
    openai_auth_json: &mut Option<String>,
) {
    let client = match create_proxy_client() {
        Ok(client) => client,
        Err(e) => {
            println!("OpenAI accounts/check client creation failed: {}", e);
            return;
        }
    };

    let response = match client
        .get(CHATGPT_ACCOUNTS_CHECK_URL)
        .header("authorization", format!("Bearer {}", access_token))
        .header("origin", "https://chatgpt.com")
        .header("referer", "https://chatgpt.com/")
        .header("accept", "application/json")
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            println!("OpenAI accounts/check request failed: {}", e);
            return;
        }
    };

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        println!(
            "OpenAI accounts/check failed: status={}, body={}",
            status,
            body.chars().take(200).collect::<String>()
        );
        return;
    }

    let value = match response.json::<Value>().await {
        Ok(value) => value,
        Err(e) => {
            println!("OpenAI accounts/check response parse failed: {}", e);
            return;
        }
    };

    let preferred_account_ids = [organization_id, chatgpt_account_id];
    let enrichment = extract_account_check_enrichment(&value, &preferred_account_ids);
    merge_account_check_enrichment_into_auth_json(openai_auth_json, enrichment);
}

pub async fn enrich_access_token_import_with_account_check(import: &mut OpenAIAccessTokenImport) {
    enrich_openai_auth_json_with_account_check(
        &import.access_token,
        import.organization_id.as_deref(),
        import.chatgpt_account_id.as_deref(),
        &mut import.openai_auth_json,
    )
    .await;
}

pub fn parse_access_token_import_input(input: &str) -> Result<OpenAIAccessTokenImport, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Access Token cannot be empty".to_string());
    }

    let parsed_json = serde_json::from_str::<Value>(trimmed).ok();
    let session_json = match parsed_json.as_ref() {
        Some(Value::Object(_)) => parsed_json.as_ref(),
        Some(_) => {
            return Err("Only raw Access Token or /api/auth/session JSON is supported".to_string());
        }
        None => None,
    };

    let access_token = match session_json {
        Some(value) => trimmed_value_string(value.get("accessToken"))
            .ok_or_else(|| "Missing accessToken in /api/auth/session JSON".to_string())?,
        None => trimmed.to_string(),
    };

    let claims = decode_jwt_payload(&access_token);
    let auth_claims = claims
        .as_ref()
        .and_then(|v| v.get("https://api.openai.com/auth"));
    let profile_claims = claims
        .as_ref()
        .and_then(|v| v.get("https://api.openai.com/profile"));

    let mut email = session_json
        .and_then(|value| value.get("user"))
        .and_then(|user| trimmed_value_string(user.get("email")))
        .or_else(|| {
            profile_claims
                .and_then(|v| trimmed_value_string(v.get("email")))
                .or_else(|| {
                    claims
                        .as_ref()
                        .and_then(|v| trimmed_value_string(v.get("email")))
                })
        });

    let chatgpt_account_id = session_json
        .and_then(|value| value.get("account"))
        .and_then(|account| trimmed_value_string(account.get("id")))
        .or_else(|| auth_claims.and_then(|v| trimmed_value_string(v.get("chatgpt_account_id"))));

    let chatgpt_user_id = session_json
        .and_then(|value| value.get("user"))
        .and_then(|user| trimmed_value_string(user.get("id")))
        .or_else(|| {
            auth_claims
                .and_then(|v| trimmed_value_string(v.get("chatgpt_user_id")))
                .or_else(|| auth_claims.and_then(|v| trimmed_value_string(v.get("user_id"))))
                .or_else(|| {
                    claims
                        .as_ref()
                        .and_then(|v| trimmed_value_string(v.get("sub")))
                })
        });

    let organization_id = auth_claims.and_then(organization_id_from_auth);

    let plan_type = session_json
        .and_then(|value| value.get("account"))
        .and_then(|account| trimmed_value_string(account.get("planType")))
        .or_else(|| auth_claims.and_then(|v| trimmed_value_string(v.get("chatgpt_plan_type"))));

    let compute_residency = session_json
        .and_then(|value| value.get("account"))
        .and_then(|account| trimmed_value_string(account.get("computeResidency")))
        .or_else(|| {
            auth_claims.and_then(|v| trimmed_value_string(v.get("chatgpt_compute_residency")))
        });

    let expires_at = claims
        .as_ref()
        .and_then(|v| v.get("exp"))
        .and_then(Value::as_i64);

    let mut auth_json = claims
        .as_ref()
        .map(extract_auth_map_from_claims)
        .unwrap_or_default();
    set_auth_json_string(&mut auth_json, "chatgpt_account_id", &chatgpt_account_id);
    set_auth_json_string(&mut auth_json, "chatgpt_user_id", &chatgpt_user_id);
    set_auth_json_string(&mut auth_json, "user_id", &chatgpt_user_id);
    set_auth_json_string(&mut auth_json, "chatgpt_plan_type", &plan_type);
    set_auth_json_string(
        &mut auth_json,
        "chatgpt_compute_residency",
        &compute_residency,
    );

    if email.is_none() {
        email = chatgpt_user_id.clone();
    }

    let openai_auth_json = if auth_json.is_empty() {
        None
    } else {
        serde_json::to_string(&auth_json).ok()
    };

    Ok(OpenAIAccessTokenImport {
        access_token,
        email,
        chatgpt_account_id,
        chatgpt_user_id,
        organization_id,
        openai_auth_json,
        expires_at,
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
        chatgpt_account_id: user_info
            .as_ref()
            .and_then(|info| info.chatgpt_account_id.clone()),
        chatgpt_user_id: user_info
            .as_ref()
            .and_then(|info| info.chatgpt_user_id.clone()),
        organization_id: user_info
            .as_ref()
            .and_then(|info| info.organization_id.clone()),
    })
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
