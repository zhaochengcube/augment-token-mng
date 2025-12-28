use std::time::SystemTime;

use crate::http_client::{create_http_client_with_cookies, create_proxy_client};
use crate::platforms::augment::models::{
    AccountStatus,
    BatchCreditConsumptionResponse,
    CompleteUserInfo,
    CreditConsumptionResponse,
    CreditInfoResponse,
    ModelsResponse,
    PaymentMethodLinkResponse,
    PortalInfo,
    SubscriptionInfo,
    TokenInfo,
    TokenStatusResult,
    UserInfo,
};
use super::oauth::{extract_token_from_session};

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

    println!("=== check_account_ban_status ===");
    println!("URL: {}", api_url);

    // Send request to find-missing API
    let response = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status_code = response.status().as_u16();
    println!("Response Status: {}", status_code);

    // Read response body
    let response_body = response.text().await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    // Analyze response based on status code
    match status_code {
        200 => {
            Ok(AccountStatus {
                status: "ACTIVE".to_string(),
                error_message: None,
            })
        }
        401 => {
            Ok(AccountStatus {
                status: "INVALID_TOKEN".to_string(),
                error_message: None,
            })
        }
        403 => {
            Ok(AccountStatus {
                status: "SUSPENDED".to_string(),
                error_message: None,
            })
        }
        _ => {
            Ok(AccountStatus {
                status: "ERROR".to_string(),
                error_message: Some(format!("HTTP {}: {}", status_code, response_body)),
            })
        }
    }
}

/// 获取用户模型信息 (get-models API)
pub async fn get_models(
    token: &str,
    tenant_url: &str,
) -> Result<ModelsResponse, String> {
    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;

    // Ensure tenant_url ends with a slash
    let base_url = if tenant_url.ends_with('/') {
        tenant_url.to_string()
    } else {
        format!("{}/", tenant_url)
    };

    let api_url = format!("{}get-models", base_url);

    println!("=== get-models API Request ===");
    println!("URL: {}", api_url);

    // Send request to get-models API
    let response = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status_code = response.status().as_u16();
    println!("Response Status: {}", status_code);

    if !response.status().is_success() {
        let error_body = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API request failed with status {}: {}", status_code, error_body));
    }

    // Read response body
    let response_body = response.text().await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    // Parse response
    let models_info: ModelsResponse = serde_json::from_str(&response_body)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(models_info)
}

/// 获取用户额度信息 (get-credit-info API)
pub async fn get_credit_info(
    token: &str,
    tenant_url: &str,
) -> Result<CreditInfoResponse, String> {
    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;

    // Ensure tenant_url ends with a slash
    let base_url = if tenant_url.ends_with('/') {
        tenant_url.to_string()
    } else {
        format!("{}/", tenant_url)
    };

    let api_url = format!("{}get-credit-info", base_url);

    println!("=== get-credit-info API Request ===");
    println!("URL: {}", api_url);

    // Send request to get-credit-info API
    let response = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status_code = response.status().as_u16();
    println!("Response Status: {}", status_code);

    if !response.status().is_success() {
        let error_body = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API request failed with status {}: {}", status_code, error_body));
    }

    // Read response body
    let response_body = response.text().await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    // Parse response
    let credit_info: CreditInfoResponse = serde_json::from_str(&response_body)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(credit_info)
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
        let mut auth_session = token_info.auth_session.clone();
        let should_refresh_session = token_info.should_refresh_session.unwrap_or(false);
        let cache = app_session_cache.clone();

        let handle = tokio::spawn(async move {
            println!("Checking status for token: {:?}", token_id);

            // 1. 根据前端标记决定是否刷新 session
            if should_refresh_session {
                if let Some(session) = &auth_session {
                    println!("⚠️ Frontend requested session refresh for token {:?}, attempting to refresh", token_id);
                    match crate::platforms::augment::oauth::refresh_auth_session(session).await {
                        Ok(new_session) => {
                            println!("✅ Session refreshed successfully for token {:?}", token_id);
                            auth_session = Some(new_session);
                            // 注意：session_updated_at 将在返回结果时由前端设置
                        }
                        Err(err) => {
                            println!("❌ Failed to refresh session for token {:?}: {}", token_id, err);
                            // 刷新失败不影响后续检测
                        }
                    }
                } else {
                    println!("  - Frontend requested session refresh but no auth_session available for token {:?}", token_id);
                }
            } else {
                println!("  - No session refresh requested for token {:?}", token_id);
            }

            // 2. 检测账号封禁状态
            let status_result = check_account_ban_status(&token, &tenant_url).await;

            // 处理账号状态检测结果
            let mut status_result = match status_result {
                Ok(status) => status,
                Err(err) => {
                    // 如果出错，创建一个错误状态并直接返回
                    let error_status = AccountStatus {
                        status: "ERROR".to_string(),
                        error_message: Some(format!("Failed to check status: {}", err)),
                    };

                    return TokenStatusResult {
                        token_id,
                        access_token: token,
                        tenant_url,
                        status_result: error_status,
                        portal_info: None,
                        portal_error: Some(format!("Status check failed: {}", err)),
                        suspensions: None,
                        email_note: None,
                        portal_url: None,
                        auth_session,
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
            if status_result.status == "SUSPENDED" {
                // 如果有 auth_session,获取详细的封禁信息
                if let Some(ref session) = auth_session {
                    println!("Account banned for {:?}, fetching detailed user info", token_id);
                    match get_user_info(session, &cache).await {
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
                    email_note: None,  // 封禁账号不获取邮箱
                    portal_url: None,
                    auth_session,
                };
            }

            // 4. 获取余额和过期时间信息
            // 优先使用 get_credit_info，失败时降级到 get_portal_info
            let mut fetched_portal_url = portal_url.clone();

            // 首次尝试获取余额
            let mut balance_result = get_balance_info(&token, &tenant_url, fetched_portal_url.as_deref()).await;

            // 5. 如果获取余额失败且没有 portal_url 但有 auth_session，尝试获取 portal_url 后重试
            if balance_result.is_err() && fetched_portal_url.is_none() && auth_session.is_some() {
                println!("Failed to fetch balance info for token {:?}, attempting to fetch portal_url from auth_session", token_id);

                if let Some(ref session) = auth_session {
                    // 检查缓存
                    let cached_app_session = {
                        let cache_guard = cache.lock().unwrap();
                        cache_guard.get(session).map(|c| c.app_session.clone())
                    };

                    // 尝试使用缓存的 app_session
                    let app_session = if let Some(app_session) = cached_app_session {
                        println!("Using cached app_session for portal_url fetch");
                        match fetch_app_subscription(&app_session).await {
                            Ok(subscription) => {
                                fetched_portal_url = subscription.portal_url.clone();
                                if let Some(ref url) = fetched_portal_url {
                                    println!("Successfully fetched portal_url from cached app_session: {}", url);
                                } else {
                                    println!("Subscription response has no portal_url");
                                }
                                Some(app_session)
                            }
                            Err(e) => {
                                println!("Cached app_session failed: {}, will refresh", e);
                                None
                            }
                        }
                    } else {
                        println!("No cached app_session found, will exchange auth_session");
                        None
                    };

                    // 如果缓存失败或不存在，交换新的 app_session
                    if app_session.is_none() && fetched_portal_url.is_none() {
                        match exchange_auth_session_for_app_session(session).await {
                            Ok(new_app_session) => {
                                // 更新缓存
                                {
                                    let mut cache_guard = cache.lock().unwrap();
                                    cache_guard.insert(session.clone(), crate::AppSessionCache {
                                        app_session: new_app_session.clone(),
                                        created_at: std::time::SystemTime::now(),
                                    });
                                }

                                // 获取订阅信息
                                match fetch_app_subscription(&new_app_session).await {
                                    Ok(subscription) => {
                                        fetched_portal_url = subscription.portal_url;
                                    }
                                    Err(e) => {
                                        println!("Failed to fetch subscription with new app_session: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Failed to exchange auth_session for app_session: {}", e);
                            }
                        }
                    }

                    // 如果成功获取到 portal_url，重试获取余额信息
                    if let Some(ref url) = fetched_portal_url {
                        println!("Successfully fetched portal_url for token {:?}: {}, retrying balance fetch", token_id, url);
                        balance_result = get_balance_info(&token, &tenant_url, Some(url.as_str())).await;
                    }
                }
            }

            // 处理最终的余额结果
            let (portal_info, portal_error) = match balance_result {
                Ok(info) => {
                    println!("Successfully fetched balance info for token {:?}: balance={}, total={:?}, expiry={:?}",
                             token_id, info.credits_balance, info.credit_total, info.expiry_date);
                    (Some(info), None)
                }
                Err(err) => {
                    println!("Failed to fetch balance info for token {:?}: {}", token_id, err);
                    (None, Some(err))
                }
            };

            // 6. 如果没有邮箱备注,尝试获取邮箱
            let email_note = if token_info.email_note.is_none() {
                match get_models(&token, &tenant_url).await {
                    Ok(models_response) => {
                        Some(models_response.user.email)
                    }
                    Err(_err) => {
                        None
                    }
                }
            } else {
                token_info.email_note.clone()
            };

            TokenStatusResult {
                token_id,
                access_token: token,
                tenant_url,
                status_result,
                portal_info,
                portal_error,
                suspensions: None,  // 正常情况下不需要 suspensions
                email_note,
                portal_url: fetched_portal_url,
                auth_session,
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
                        status: "ERROR".to_string(),
                        error_message: Some(format!("Task execution failed: {}", err)),
                    },
                    portal_info: None,
                    portal_error: Some(format!("Task failed: {}", err)),
                    suspensions: None,
                    email_note: None,
                    portal_url: None,
                    auth_session: None,
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
    let mut credit_total = None;

    if let Some(credit_blocks) = ledger_data["credit_blocks"].as_array() {
        if let Some(first_block) = credit_blocks.first() {
            // 获取 maximum_initial_balance 作为 credit_total
            if let Some(max_balance) = first_block["maximum_initial_balance"].as_str() {
                credit_total = max_balance.parse::<f64>().ok().map(|v| v.floor() as i32);
            }

            // 获取 effective_date 并加 1 个月
            if let Some(effective_date_str) = first_block["effective_date"].as_str() {
                // 尝试解析日期
                if let Ok(effective_date) = chrono::DateTime::parse_from_rfc3339(effective_date_str) {
                    // 加 1 个月（精确月份）
                    let expiry = effective_date.with_timezone(&chrono::Utc) + chrono::Months::new(1);
                    expiry_date = Some(expiry.to_rfc3339());
                }
            }
        }
    }

    Ok(PortalInfo {
        credits_balance,
        credit_total,
        expiry_date,
    })
}

/// 获取余额信息 - 优先使用 get_credit_info，失败时降级到 get_portal_info
async fn get_balance_info(
    token: &str,
    tenant_url: &str,
    portal_url: Option<&str>,
) -> Result<PortalInfo, String> {
    // 1. 优先尝试 get_credit_info
    match get_credit_info(token, tenant_url).await {
        Ok(credit_info) => {
            // 转换 CreditInfoResponse 到 PortalInfo
            let credits_balance = credit_info.usage_units_remaining.floor() as i32;
            let credit_total = Some(credit_info.usage_units_total.floor() as i32);
            let expiry_date = Some(credit_info.current_billing_cycle_end_date_iso);

            Ok(PortalInfo {
                credits_balance,
                credit_total,
                expiry_date,
            })
        }
        Err(err) => {
            println!("get_credit_info failed: {}, trying get_portal_info", err);

            // 2. 降级到 get_portal_info (仅当 portal_url 存在时)
            if let Some(url) = portal_url {
                get_portal_info(url).await
            } else {
                Err(format!("get_credit_info failed and no portal_url available: {}", err))
            }
        }
    }
}

// ============ Credit Consumption API ============

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
        portal_url: None,
    })
}

/// 通过 auth session 交换 app session
pub async fn exchange_auth_session_for_app_session(auth_session: &str) -> Result<String, String> {
    use reqwest::cookie::{Jar, CookieStore};
    use std::sync::Arc;

    // 创建 cookie jar
    let jar = Arc::new(Jar::default());

    // 设置 auth session cookie 到 auth.augmentcode.com 域
    let auth_url = "https://auth.augmentcode.com/".parse::<reqwest::Url>()
        .map_err(|e| format!("Failed to parse auth URL: {}", e))?;
    jar.add_cookie_str(
        &format!("session={}", auth_session),
        &auth_url
    );

    // 创建带 cookie store 的客户端
    let client = create_http_client_with_cookies(jar.clone())?;

    // 直接 GET /login 触发授权流
    let _login_response = client
        .get("https://app.augmentcode.com/login")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Failed to exchange session: {}", e))?;

    // 从 cookie jar 中获取 app.augmentcode.com 的 cookies
    let app_url = "https://app.augmentcode.com/".parse::<reqwest::Url>()
        .map_err(|e| format!("Failed to parse app URL: {}", e))?;

    if let Some(header_value) = jar.cookies(&app_url) {
        if let Ok(cookie_str) = header_value.to_str() {
            let cookie_string = cookie_str.to_string();
            // 提取 _session 的值
            if let Some(start) = cookie_string.find("_session=") {
                let session_part = &cookie_string[start + 9..];
                // 可能有多个 cookie，用分号分隔，只取第一个
                let session_value = session_part.split(';').next().unwrap_or(session_part);
                return Ok(urlencoding::decode(session_value)
                    .unwrap_or_else(|_| session_value.into())
                    .to_string());
            }
        }
    }

    Err("Failed to extract app session cookie".to_string())
}

/// 通过 auth session 交换完整的 Cookie 字符串(专门用于绑卡链接)
pub async fn exchange_auth_session_for_full_cookies(auth_session: &str) -> Result<String, String> {
    // 复用第一个方法获取 _session 值
    let session_value = exchange_auth_session_for_app_session(auth_session).await?;

    // 使用固定的tracking cookies模板,只替换 _session 值
    let full_cookie = format!(
        "_ga=GA1.1.242842833.1762031996; _fbp=fb.1.1762262295646.930716998848089033; _rdt_uuid=1762262295652.34187976-0dc0-4e33-8a70-645aa454b403; _gcl_au=1.1.1646447425.1762031996.1424920390.1763188023.1763188269; _ga_F6GPDJDCJY=GS2.1.s1763188002$o15$g1$t1763188269$j31$l0$h0; _session={}",
        urlencoding::encode(&session_value)
    );

    Ok(full_cookie)
}

/// 获取用户信息
pub async fn fetch_app_user(app_session: &str) -> Result<UserInfo, String> {
    // 使用新的 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;
    let response = client
        .get("https://app.augmentcode.com/api/user")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user info: {}", e))?;

    response
        .json::<UserInfo>()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))
}

/// 获取订阅信息
pub async fn fetch_app_subscription(app_session: &str) -> Result<SubscriptionInfo, String> {
    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;
    let response = client
        .get("https://app.augmentcode.com/api/subscription")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch subscription info: {}", e))?;

    response
        .json::<SubscriptionInfo>()
        .await
        .map_err(|e| format!("Failed to parse subscription info: {}", e))
}

/// 获取绑卡链接
/// 参数 cookie_string: 完整的Cookie字符串,包含 _session 和其他cookies
pub async fn fetch_payment_method_link(cookie_string: &str) -> Result<String, String> {
    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;

    // 创建空的JSON body
    let body = "{}";

    let cookie_header_value = format!("Cookie={}", cookie_string);

    let response = client
        .post("https://app.augmentcode.com/api/setup-trial-payment-method")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", cookie_header_value)
        .header("Content-Type", "application/json")
        .header("Content-Length", body.len().to_string())
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch payment method link: {}", e))?;

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response text: {}", e))?;

    let payment_response: PaymentMethodLinkResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse payment method link response: {}. Response was: {}", e, response_text))?;

    Ok(payment_response.url)
}

/// 使用已有的 app_session 获取完整的用户信息
pub async fn get_user_info_with_app_session(app_session: &str) -> Result<CompleteUserInfo, String> {
    // 只获取用户信息
    let user_info = fetch_app_user(app_session).await.ok();

    // 计算 ban_status
    let ban_status = if let Some(ref user) = user_info {
        if let Some(ref suspensions) = user.suspensions {
            if let Some(arr) = suspensions.as_array() {
                if !arr.is_empty() {
                    if let Some(first) = arr.first() {
                        if let Some(suspension_type) = first.get("suspensionType").and_then(|v| v.as_str()) {
                            format!("BANNED-{}", suspension_type)
                        } else {
                            "BANNED".to_string()
                        }
                    } else {
                        "BANNED".to_string()
                    }
                } else {
                    "ACTIVE".to_string()
                }
            } else {
                "ACTIVE".to_string()
            }
        } else {
            "ACTIVE".to_string()
        }
    } else {
        "ACTIVE".to_string()
    };

    Ok(CompleteUserInfo {
        suspensions: user_info.and_then(|u| u.suspensions),
        ban_status,
    })
}

/// 获取完整的用户信息 (使用缓存的 app_session)
pub async fn get_user_info(
    auth_session: &str,
    app_session_cache: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, crate::AppSessionCache>>>,
) -> Result<CompleteUserInfo, String> {
    // 1. 检查缓存中是否有有效的 app_session
    let cached_app_session = {
        let cache = app_session_cache.lock().unwrap();
        cache.get(auth_session).map(|c| c.app_session.clone())
    };

    // 2. 尝试使用缓存的 app_session
    if let Some(app_session) = cached_app_session {
        match get_user_info_with_app_session(&app_session).await {
            Ok(user_info) => return Ok(user_info),
            Err(e) => println!("Cached app_session failed: {}, will refresh", e),
        }
    }

    // 3. 交换新的 app_session
    let app_session = exchange_auth_session_for_app_session(auth_session).await?;

    println!("App session obtained: {}", &app_session[..20.min(app_session.len())]);

    // 4. 更新缓存
    {
        let mut cache = app_session_cache.lock().unwrap();
        cache.insert(auth_session.to_string(), crate::AppSessionCache {
            app_session: app_session.clone(),
            created_at: SystemTime::now(),
        });
    }

    // 5. 获取用户信息
    get_user_info_with_app_session(&app_session).await
}

// ============ 团队管理相关函数 ============

/// 邀请团队成员
pub async fn invite_team_members(app_session: &str, emails: Vec<String>) -> Result<(), String> {
    let client = create_proxy_client()?;

    let response = client
        .post("https://app.augmentcode.com/api/team/invite")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "emails": emails }))
        .send()
        .await
        .map_err(|e| format!("Failed to invite team members: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Failed to invite team members (HTTP {}): {}", status, error_body));
    }

    Ok(())
}

/// 修改团队席位数
pub async fn update_team_seats(app_session: &str, seats: u32) -> Result<serde_json::Value, String> {
    let client = create_proxy_client()?;

    let response = client
        .request(reqwest::Method::PATCH, "https://app.augmentcode.com/api/team")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "seats": seats }))
        .send()
        .await
        .map_err(|e| format!("Failed to update team seats: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Failed to update team seats (HTTP {}): {}", status, error_body));
    }

    response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse update team seats response: {}", e))
}

/// 删除团队邀请
pub async fn delete_team_invitation(app_session: &str, invitation_id: &str) -> Result<(), String> {
    let client = create_proxy_client()?;

    let url = format!("https://app.augmentcode.com/api/team/invite/{}", invitation_id);

    let response = client
        .delete(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
        .send()
        .await
        .map_err(|e| format!("Failed to delete team invitation: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Failed to delete team invitation (HTTP {}): {}", status, error_body));
    }

    Ok(())
}

/// 删除团队成员
pub async fn delete_team_member(app_session: &str, user_id: &str) -> Result<(), String> {
    let client = create_proxy_client()?;

    let url = format!("https://app.augmentcode.com/api/team/user/{}", user_id);

    let response = client
        .delete(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
        .send()
        .await
        .map_err(|e| format!("Failed to delete team member: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Failed to delete team member (HTTP {}): {}", status, error_body));
    }

    Ok(())
}

/// 获取团队信息
pub async fn fetch_team_info(app_session: &str) -> Result<serde_json::Value, String> {
    let client = create_proxy_client()?;

    let response = client
        .get("https://app.augmentcode.com/api/team")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch team info: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Failed to fetch team info (HTTP {}): {}", status, error_body));
    }

    response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse team info response: {}", e))
}
