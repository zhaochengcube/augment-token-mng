use base64::{engine::general_purpose, Engine as _};
use rand::Rng;
use reqwest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;
use regex::Regex;
use crate::http_client::create_proxy_client;
use crate::augment_user_info::exchange_auth_session_for_app_session;

const CLIENT_ID: &str = "v";
const AUTH_BASE_URL: &str = "https://auth.augmentcode.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AugmentOAuthState {
    pub code_verifier: String,
    pub code_challenge: String,
    pub state: String,
    pub creation_time: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedCode {
    pub code: String,
    pub state: String,
    pub tenant_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AugmentTokenResponse {
    pub access_token: String,
    pub tenant_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountStatus {
    pub is_banned: bool,
    pub status: String,
    pub error_message: Option<String>,
    pub response_code: Option<u16>,
    pub response_body: Option<String>,
    // 调试信息
    pub debug_info: DebugInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugInfo {
    pub request_url: String,
    pub request_headers: std::collections::HashMap<String, String>,
    pub request_body: String,
    pub response_headers: std::collections::HashMap<String, String>,
    pub response_body: String,
    pub response_status_text: String,
}

// 批量检测相关结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub access_token: String,
    pub tenant_url: String,
    pub id: Option<String>, // 用于前端识别是哪个token
    pub portal_url: Option<String>, // Portal URL用于获取使用次数信息
    pub auth_session: Option<String>, // Auth session用于自动刷新token
}

// Portal信息结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct PortalInfo {
    pub credits_balance: i32,
    pub expiry_date: Option<String>,
    pub can_still_use: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenStatusResult {
    pub token_id: Option<String>, // 对应输入的id
    pub access_token: String, // 保留token用于前端更新 (如果被刷新,这里是新token)
    pub tenant_url: String, // 保留tenant_url用于前端更新 (如果被刷新,这里是新url)
    pub status_result: AccountStatus,
    pub portal_info: Option<PortalInfo>, // Portal信息（如果有）
    pub portal_error: Option<String>, // Portal获取错误（如果有）
    pub suspensions: Option<serde_json::Value>, // 封禁详情（如果有）
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub response_text: String,
    pub request_message: String,
}


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

    Ok(AugmentTokenResponse {
        access_token: token,
        tenant_url: parsed_code.tenant_url,
    })
}

pub async fn check_account_ban_status(
    token: &str,
    tenant_url: &str,
) -> Result<AccountStatus, String> {
    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;



    // Ensure tenant_url ends with a slash
    let base_url = if tenant_url.ends_with('/') {
        tenant_url.to_string()
    } else {
        format!("{}/", tenant_url)
    };

    let api_url = format!("{}find-missing", base_url);

    // Empty request body for find-missing endpoint
    let request_body = serde_json::json!({});

    // Prepare request headers for debugging
    let mut request_headers = HashMap::new();
    request_headers.insert("Content-Type".to_string(), "application/json".to_string());
    request_headers.insert("Authorization".to_string(), format!("Bearer {}", token));

    // Print debug info to console
    println!("=== API Request Debug Info ===");
    println!("URL: {}", api_url);
    println!("Method: POST");
    println!("Headers: {:#?}", request_headers);
    println!("Request Body: {}", request_body.to_string());
    println!("==============================");

    // Send request to find-missing API
    let response = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status_code = response.status().as_u16();
    let status_text = response.status().to_string();

    // Collect response headers
    let mut response_headers = HashMap::new();
    for (name, value) in response.headers() {
        response_headers.insert(
            name.to_string(),
            value.to_str().unwrap_or("<invalid utf8>").to_string(),
        );
    }

    // Print response debug info
    println!("=== API Response Debug Info ===");
    println!("Status Code: {} ({})", status_code, status_text);
    println!("Response Headers: {:#?}", response_headers);

    // Read response body
    let response_body = response.text().await
        .map_err(|e| format!("Failed to read response body: {}", e))?;
    println!("Response Body: {}", response_body);
    println!("===============================");

    // Create debug info
    let debug_info = DebugInfo {
        request_url: api_url,
        request_headers,
        request_body: request_body.to_string(),
        response_headers,
        response_body: response_body.clone(),
        response_status_text: status_text,
    };

    // Analyze response to determine ban status
    // First check for "suspended" keyword in response body regardless of status code
    if response_body.to_lowercase().contains("suspended") {
        Ok(AccountStatus {
            is_banned: true,
            status: "SUSPENDED".to_string(),
            error_message: Some("Account is suspended based on response content".to_string()),
            response_code: Some(status_code),
            response_body: Some(response_body),
            debug_info,
        })
    } else if response_body.to_lowercase().contains("invalid token") {
        // Special case for invalid token - not banned, just invalid
        Ok(AccountStatus {
            is_banned: false,
            status: "INVALID_TOKEN".to_string(),
            error_message: Some("Token is invalid".to_string()),
            response_code: Some(status_code),
            response_body: Some(response_body),
            debug_info,
        })
    } else if (200..300).contains(&status_code) {
        // Success status and no "suspended" keyword - account is active
        Ok(AccountStatus {
            is_banned: false,
            status: "ACTIVE".to_string(),
            error_message: None,
            response_code: Some(status_code),
            response_body: Some(response_body),
            debug_info,
        })
    } else {
        // Handle different error status codes (without "suspended" keyword)
        let (is_banned, status, error_message) = match status_code {
            402 => (false, "EXPIRED", "Subscription inactive or expired"),
            401 => (true, "UNAUTHORIZED", "Token is invalid or account is banned"),
            403 => (true, "FORBIDDEN", "Access forbidden - account may be banned"),
            429 => (false, "RATE_LIMITED", "Rate limited - account is active but throttled"),
            500..=599 => (false, "SERVER_ERROR", "Server error - cannot determine ban status"),
            _ => (true, "UNKNOWN_ERROR", "Unknown error - possible ban"),
        };

        Ok(AccountStatus {
            is_banned,
            status: status.to_string(),
            error_message: Some(format!("{}: {}", error_message, response_body)),
            response_code: Some(status_code),
            response_body: Some(response_body),
            debug_info,
        })
    }
}

// 批量检测账号状态
pub async fn batch_check_account_status(
    tokens: Vec<TokenInfo>,
    app_session_cache: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, crate::AppSessionCache>>>,
) -> Result<Vec<TokenStatusResult>, String> {


    // 创建并发任务并立即spawn
    let mut handles = Vec::new();

    for token_info in tokens {
        let mut token = token_info.access_token.clone();
        let mut tenant_url = token_info.tenant_url.clone();
        let token_id = token_info.id.clone();
        let portal_url = token_info.portal_url.clone();
        let auth_session = token_info.auth_session.clone();
        let cache = app_session_cache.clone();

        let handle = tokio::spawn(async move {
            println!("Checking status for token: {:?}", token_id);

            // 1. 先检测账号封禁状态
            let status_result = check_account_ban_status(&token, &tenant_url).await;

            // 处理账号状态检测结果
            let mut status_result = match status_result {
                Ok(status) => status,
                Err(err) => {
                    // 如果出错，创建一个错误状态并直接返回
                    let error_status = AccountStatus {
                        is_banned: false,
                        status: "ERROR".to_string(),
                        error_message: Some(format!("Failed to check status: {}", err)),
                        response_code: None,
                        response_body: None,
                        debug_info: DebugInfo {
                            request_url: format!("{}find-missing", tenant_url),
                            request_headers: HashMap::new(),
                            request_body: "{}".to_string(),
                            response_headers: HashMap::new(),
                            response_body: format!("Error: {}", err),
                            response_status_text: "Error".to_string(),
                        },
                    };

                    return TokenStatusResult {
                        token_id,
                        access_token: token,
                        tenant_url,
                        status_result: error_status,
                        portal_info: None,
                        portal_error: Some(format!("Status check failed: {}", err)),
                        suspensions: None,
                    };
                }
            };

            // 2. 如果检测到 INVALID_TOKEN 且有 auth_session，尝试自动刷新
            if status_result.status == "INVALID_TOKEN" {
                if let Some(ref session) = auth_session {
                    println!("Detected INVALID_TOKEN for {:?}, attempting auto-refresh with auth_session", token_id);

                    match extract_token_from_session(session).await {
                        Ok(new_token_response) => {
                            println!("Successfully refreshed token for {:?}", token_id);
                            // 更新 token 和 tenant_url
                            token = new_token_response.access_token;
                            tenant_url = new_token_response.tenant_url;

                            // 重新检测状态
                            match check_account_ban_status(&token, &tenant_url).await {
                                Ok(new_status) => {
                                    status_result = new_status;
                                    status_result.error_message = Some(format!(
                                        "Token was invalid but successfully auto-refreshed. New status: {}",
                                        status_result.status
                                    ));
                                }
                                Err(err) => {
                                    println!("Failed to check status after refresh: {}", err);
                                    status_result.error_message = Some(format!(
                                        "Token refreshed but status check failed: {}",
                                        err
                                    ));
                                }
                            }
                        }
                        Err(err) => {
                            println!("Failed to refresh token for {:?}: {}", token_id, err);

                            // 如果刷新失败原因是 SESSION_ERROR_OR_ACCOUNT_BANNED，视为账号封禁
                            if err.contains("SESSION_ERROR_OR_ACCOUNT_BANNED") {
                                status_result.status = "SUSPENDED".to_string();
                                status_result.is_banned = true;
                                status_result.error_message = Some(
                                    "Account is suspended (detected during token refresh)".to_string()
                                );
                            } else {
                                status_result.error_message = Some(format!(
                                    "Token is invalid. Auto-refresh failed: {}",
                                    err
                                ));
                            }
                        }
                    }
                } else {
                    println!("Token {:?} is invalid but no auth_session available for refresh", token_id);
                    status_result.error_message = Some(
                        "Token is invalid. No auth_session available for auto-refresh".to_string()
                    );
                }
            }

            // 3. 如果账号被封禁，尝试获取详细的用户信息
            let mut suspensions_info = None;
            if status_result.is_banned {
                // 如果有 auth_session,获取详细的封禁信息
                if let Some(ref session) = auth_session {
                    println!("Account banned for {:?}, fetching detailed user info", token_id);
                    match crate::augment_user_info::get_user_info(session).await {
                        Ok(user_info) => {
                            println!("Successfully fetched user info for banned account {:?}", token_id);
                            // 保存 suspensions 信息
                            if let Some(suspensions) = user_info.suspensions {
                                suspensions_info = Some(suspensions.clone());
                                status_result.error_message = Some(format!(
                                    "Account banned. Suspensions: {}",
                                    serde_json::to_string(&suspensions).unwrap_or_else(|_| "N/A".to_string())
                                ));
                            }
                        }
                        Err(err) => {
                            println!("Failed to fetch user info for banned account {:?}: {}", token_id, err);
                            // 不影响主流程,只记录错误
                        }
                    }
                }

                return TokenStatusResult {
                    token_id,
                    access_token: token,
                    tenant_url,
                    status_result,
                    portal_info: None,
                    portal_error: None,
                    suspensions: suspensions_info,
                };
            }

            // 4. 获取余额和过期时间信息
            // 优先使用 auth_session,其次使用 portal_url
            let (portal_info, portal_error) = if let Some(ref session) = auth_session {
                // 优先使用 auth_session 获取信息(使用缓存)
                println!("Using auth_session to fetch credits and expiry for {:?}", token_id);

                // 1. 检查缓存
                let cached_app_session = {
                    let cache_lock = cache.lock().unwrap();
                    cache_lock.get(session).map(|c| c.app_session.clone())
                };

                // 2. 尝试使用缓存的 app_session
                let user_info_result = if let Some(app_session) = cached_app_session {
                    println!("Using cached app_session for user info");
                    match crate::augment_user_info::get_user_info_with_app_session(&app_session).await {
                        Ok(info) => Ok(info),
                        Err(e) => {
                            println!("Cached app_session failed: {}, will refresh", e);
                            // 缓存失效,获取新的
                            match crate::augment_user_info::exchange_auth_session_for_app_session(session).await {
                                Ok(new_app_session) => {
                                    // 更新缓存
                                    {
                                        let mut cache_lock = cache.lock().unwrap();
                                        cache_lock.insert(
                                            session.clone(),
                                            crate::AppSessionCache {
                                                app_session: new_app_session.clone(),
                                                created_at: std::time::SystemTime::now(),
                                            },
                                        );
                                    }
                                    crate::augment_user_info::get_user_info_with_app_session(&new_app_session).await
                                }
                                Err(e) => Err(e)
                            }
                        }
                    }
                } else {
                    // 没有缓存,获取新的
                    println!("No cached app_session, exchanging new one");
                    match crate::augment_user_info::exchange_auth_session_for_app_session(session).await {
                        Ok(new_app_session) => {
                            // 更新缓存
                            {
                                let mut cache_lock = cache.lock().unwrap();
                                cache_lock.insert(
                                    session.clone(),
                                    crate::AppSessionCache {
                                        app_session: new_app_session.clone(),
                                        created_at: std::time::SystemTime::now(),
                                    },
                                );
                            }
                            crate::augment_user_info::get_user_info_with_app_session(&new_app_session).await
                        }
                        Err(e) => Err(e)
                    }
                };

                // 3. 处理结果
                match user_info_result {
                    Ok(user_info) => {
                        let credits_balance = user_info.credits_balance.unwrap_or(0);
                        let expiry_date = user_info.expiry_date;

                        // 判断是否可以继续使用
                        let can_still_use = if credits_balance == 0 {
                            // 如果余额为0，检查订阅状态
                            match check_subscription_info(token.clone(), tenant_url.clone()).await {
                                Ok(can_use) => can_use,
                                Err(err) => {
                                    println!("Failed to check subscription info: {}", err);
                                    false
                                }
                            }
                        } else {
                            true
                        };

                        // 如果账号状态为 EXPIRED，则无论上面检测结果如何，都视为不可使用
                        let can_still_use = if status_result.status == "EXPIRED" {
                            false
                        } else {
                            can_still_use
                        };

                        (Some(PortalInfo {
                            credits_balance,
                            expiry_date,
                            can_still_use,
                        }), None)
                    }
                    Err(err) => {
                        println!("Failed to fetch user info with auth_session: {}", err);
                        (None, Some(format!("Failed to fetch user info: {}", err)))
                    }
                }
            } else if let Some(ref portal_url_ref) = portal_url {
                // 没有 auth_session,使用 portal_url
                println!("Using portal_url to fetch credits and expiry for {:?}", token_id);
                match get_portal_info(portal_url_ref).await {
                    Ok(mut portal_info) => {
                        // 如果credits_balance为0，检查订阅状态
                        if portal_info.credits_balance == 0 {
                            match check_subscription_info(token.clone(), tenant_url.clone()).await {
                                Ok(can_use) => {
                                    portal_info.can_still_use = can_use;
                                }
                                Err(err) => {
                                    println!("Failed to check subscription info: {}", err);
                                    portal_info.can_still_use = false;
                                }
                            }
                        } else {
                            // 如果有余额，设置为可以使用
                            portal_info.can_still_use = true;
                        }

                        // 如果账号状态为 EXPIRED，则无论上面检测结果如何，都视为不可使用
                        if status_result.status == "EXPIRED" {
                            portal_info.can_still_use = false;
                        }
                        (Some(portal_info), None)
                    }
                    Err(err) => (None, Some(err))
                }
            } else {
                // 既没有 auth_session 也没有 portal_url
                println!("No auth_session or portal_url available for {:?}", token_id);
                (None, None)
            };


            TokenStatusResult {
                token_id,
                access_token: token,
                tenant_url,
                status_result,
                portal_info,
                portal_error,
                suspensions: None,  // 正常情况下不需要 suspensions
            }
        });
        
        handles.push(handle);
    }

    let mut results = Vec::new();
    for (index, handle) in handles.into_iter().enumerate() {
        match handle.await {
            Ok(result) => results.push(result),
            Err(err) => {
                eprintln!("Task {} failed: {}", index, err);
                // 创建一个错误状态的结果
                results.push(TokenStatusResult {
                    token_id: Some(format!("task_{}", index)),
                    access_token: "".to_string(),
                    tenant_url: "".to_string(),
                    status_result: AccountStatus {
                        is_banned: false,
                        status: "ERROR".to_string(),
                        error_message: Some(format!("Task execution failed: {}", err)),
                        response_code: None,
                        response_body: None,
                        debug_info: DebugInfo {
                            request_url: "".to_string(),
                            request_headers: HashMap::new(),
                            request_body: "{}".to_string(),
                            response_headers: HashMap::new(),
                            response_body: format!("Task Error: {}", err),
                            response_status_text: "Error".to_string(),
                        },
                    },
                    portal_info: None,
                    portal_error: Some(format!("Task failed: {}", err)),
                    suspensions: None,
                });
            }
        }
    }
    
    
    Ok(results)
}

// 从Portal URL提取token
fn extract_token_from_portal_url(portal_url: &str) -> Option<String> {
    if let Ok(url) = url::Url::parse(portal_url) {
        url.query_pairs()
            .find(|(key, _)| key == "token")
            .map(|(_, value)| value.into_owned())
    } else {
        None
    }
}

// 获取Portal信息
async fn get_portal_info(portal_url: &str) -> Result<PortalInfo, String> {
    let token = extract_token_from_portal_url(portal_url)
        .ok_or("Failed to extract token from portal URL")?;

    // 获取customer信息
    let customer_url = format!("https://portal.withorb.com/api/v1/customer_from_link?token={}", token);

    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;
    let customer_response = client
        .get(&customer_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await
        .map_err(|e| format!("Failed to get customer info: {}", e))?;

    if !customer_response.status().is_success() {
        return Err(format!("Customer API request failed: {}", customer_response.status()));
    }

    let customer_text = customer_response.text().await
        .map_err(|e| format!("Failed to read customer response: {}", e))?;
    
    let customer_data: serde_json::Value = serde_json::from_str(&customer_text)
        .map_err(|e| format!("Failed to parse customer response: {}", e))?;

    // 提取customer_id和pricing_unit_id
    let customer_id = customer_data["customer"]["id"]
        .as_str()
        .ok_or("Customer ID not found")?;
    
    let pricing_unit_id = customer_data["customer"]["ledger_pricing_units"][0]["id"]
        .as_str()
        .ok_or("Pricing unit ID not found")?;
    

    // 获取ledger summary
    let ledger_url = format!(
        "https://portal.withorb.com/api/v1/customers/{}/ledger_summary?pricing_unit_id={}&token={}",
        customer_id, pricing_unit_id, token
    );

    let ledger_response = client
        .get(&ledger_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Accept", "application/json, text/plain, */*")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .await
        .map_err(|e| format!("Failed to get ledger info: {}", e))?;

    if !ledger_response.status().is_success() {
        return Err(format!("Ledger API request failed: {}", ledger_response.status()));
    }

    let ledger_text = ledger_response.text().await
        .map_err(|e| format!("Failed to read ledger response: {}", e))?;
    
    let ledger_data: serde_json::Value = serde_json::from_str(&ledger_text)
        .map_err(|e| format!("Failed to parse ledger response: {}", e))?;


    // 解析Portal信息（根据当前返回，credits_balance 为字符串，如 "9.00"）
    let credits_balance: i32 = ledger_data["credits_balance"].as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|v| v.floor() as i32)
        .unwrap_or(0);
    
    // println removed by request: parsed credits balance

    let mut expiry_date = None;

    if let Some(credit_blocks) = ledger_data["credit_blocks"].as_array() {
        if let Some(first_block) = credit_blocks.first() {
            expiry_date = first_block["expiry_date"].as_str().map(|s| s.to_string());
        }
    }

    Ok(PortalInfo {
        credits_balance,
        expiry_date,
        can_still_use: false, // 默认值，将在batch_check_account_status中更新
    })
}

pub async fn check_subscription_info(token: String, tenant_url: String) -> Result<bool, String> {
    let url = format!("{}/subscription-info", tenant_url.trim_end_matches('/'));

    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("Failed to make API request: {}", e))?;

    let status = response.status();

    if status.is_success() {
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        // 检查响应中是否包含 "out of user messages"
        let has_usage_limit = response_text.contains("out of user messages");
        Ok(!has_usage_limit) // 如果包含限制信息则返回false，否则返回true
    } else {
        Err(format!("API request failed with status {}: {}", status, response.status()))
    }
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

    Ok(AugmentTokenResponse {
        access_token: token_data.access_token,
        tenant_url: tenant_url.to_string(),
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

// ============ Credit Consumption API ============

/// Credit 消费数据点
#[derive(Debug, Serialize, Deserialize)]
pub struct CreditDataPoint {
    #[serde(rename(serialize = "group_key", deserialize = "groupKey"))]
    pub group_key: Option<String>, // 模型名称
    #[serde(rename(serialize = "date_range", deserialize = "dateRange"))]
    pub date_range: Option<DateRange>,
    #[serde(rename(serialize = "credits_consumed", deserialize = "creditsConsumed"), default = "default_credits_consumed")]
    pub credits_consumed: String,
}

/// 默认值函数：当 creditsConsumed 字段缺失时返回 "0"
fn default_credits_consumed() -> String {
    "0".to_string()
}

/// 日期范围
#[derive(Debug, Serialize, Deserialize)]
pub struct DateRange {
    #[serde(rename(serialize = "start_date_iso", deserialize = "startDateIso"))]
    pub start_date_iso: String,
    #[serde(rename(serialize = "end_date_iso", deserialize = "endDateIso"))]
    pub end_date_iso: String,
}

/// Credit 消费响应
#[derive(Debug, Serialize, Deserialize)]
pub struct CreditConsumptionResponse {
    #[serde(rename(serialize = "data_points", deserialize = "dataPoints"), default)]
    pub data_points: Vec<CreditDataPoint>,
}

/// 批量获取 Credit 消费数据的响应
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCreditConsumptionResponse {
    pub stats_data: CreditConsumptionResponse,
    pub chart_data: CreditConsumptionResponse,
}

/// 使用已有的 app_session 获取 Credit 消费数据
pub async fn get_batch_credit_consumption_with_app_session(
    app_session: &str,
) -> Result<BatchCreditConsumptionResponse, String> {
    // 使用 ProxyClient
    let client = create_proxy_client()?;

    // 并行获取两个数据
    let stats_url = "https://app.augmentcode.com/api/credit-consumption?groupBy=NONE&granularity=DAY&billingCycle=CURRENT_BILLING_CYCLE";
    let chart_url = "https://app.augmentcode.com/api/credit-consumption?groupBy=MODEL_NAME&granularity=TOTAL&billingCycle=CURRENT_BILLING_CYCLE";

    println!("Fetching stats from: {}", stats_url);
    println!("Fetching chart from: {}", chart_url);

    let (stats_result, chart_result) = tokio::join!(
        async {
            let response = client
                .get(stats_url)
                .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| format!("Failed to fetch stats data: {}", e))?;

            let status = response.status();
            if !status.is_success() {
                let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(format!("Stats API returned status {}: {}", status, error_body));
            }

            let response_text = response.text().await
                .map_err(|e| format!("Failed to read stats response body: {}", e))?;

            println!("Stats response: {}", response_text);

            serde_json::from_str::<CreditConsumptionResponse>(&response_text)
                .map_err(|e| format!("Failed to parse stats response: {}. Response body: {}", e, response_text))
        },
        async {
            let response = client
                .get(chart_url)
                .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .header("Accept", "application/json")
                .send()
                .await
                .map_err(|e| format!("Failed to fetch chart data: {}", e))?;

            let status = response.status();
            if !status.is_success() {
                let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(format!("Chart API returned status {}: {}", status, error_body));
            }

            let response_text = response.text().await
                .map_err(|e| format!("Failed to read chart response body: {}", e))?;

            println!("Chart response: {}", response_text);

            serde_json::from_str::<CreditConsumptionResponse>(&response_text)
                .map_err(|e| format!("Failed to parse chart response: {}. Response body: {}", e, response_text))
        }
    );

    let stats_data = stats_result?;
    let chart_data = chart_result?;

    Ok(BatchCreditConsumptionResponse {
        stats_data,
        chart_data,
    })
}