//! Cursor 认证模块

use crate::http_client::create_proxy_client;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

/// 将 null 反序列化为 0.0
fn null_as_zero<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    Option::<f64>::deserialize(deserializer).map(|v| v.unwrap_or(0.0))
}

/// Cursor 用户信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorUserInfo {
    pub id: String,
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// Usage Summary 响应（cursor.com/api/usage-summary 返回结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummary {
    #[serde(default)]
    pub membership_type: Option<String>,
    #[serde(default)]
    pub individual_usage: Option<IndividualUsage>,
    #[serde(default)]
    pub billing_cycle_start: Option<String>,
    #[serde(default)]
    pub billing_cycle_end: Option<String>,
}

/// 个人用量
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndividualUsage {
    #[serde(default)]
    pub plan: Option<PlanUsage>,
    #[serde(rename = "onDemand", default)]
    pub on_demand: Option<OnDemandUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_cycle_start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_cycle_end: Option<String>,
}

/// Plan 用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanUsage {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, deserialize_with = "null_as_zero")]
    pub used: f64,
    #[serde(default, deserialize_with = "null_as_zero")]
    pub limit: f64,
    #[serde(default, deserialize_with = "null_as_zero")]
    pub remaining: f64,
    #[serde(default)]
    pub breakdown: Option<PlanBreakdown>,
    #[serde(rename = "autoPercentUsed", default, deserialize_with = "null_as_zero")]
    pub auto_percent_used: f64,
    #[serde(rename = "apiPercentUsed", default, deserialize_with = "null_as_zero")]
    pub api_percent_used: f64,
    #[serde(rename = "totalPercentUsed", default, deserialize_with = "null_as_zero")]
    pub total_percent_used: f64,
}

/// Plan 用量明细
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanBreakdown {
    #[serde(default, deserialize_with = "null_as_zero")]
    pub included: f64,
    #[serde(default, deserialize_with = "null_as_zero")]
    pub bonus: f64,
    #[serde(default, deserialize_with = "null_as_zero")]
    pub total: f64,
}

/// On-Demand 用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnDemandUsage {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default, deserialize_with = "null_as_zero")]
    pub used: f64,
    #[serde(default, deserialize_with = "null_as_zero")]
    pub limit: f64,
    #[serde(default, deserialize_with = "null_as_zero")]
    pub remaining: f64,
}

/// 聚合用量数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregatedUsageData {
    pub aggregations: Vec<ModelUsage>,
    pub total_input_tokens: String,
    pub total_output_tokens: String,
    pub total_cache_write_tokens: String,
    pub total_cache_read_tokens: String,
    pub total_cost_cents: f64,
}

/// 模型用量
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub model_intent: String,
    pub input_tokens: String,
    pub output_tokens: String,
    pub cache_write_tokens: String,
    pub cache_read_tokens: String,
    pub total_cents: f64,
}

/// 过滤的使用事件数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredUsageEventsData {
    #[serde(rename = "totalUsageEventsCount")]
    pub total_usage_events_count: i32,
    #[serde(rename = "usageEventsDisplay")]
    pub usage_events_display: Vec<UsageEventDisplay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEventDisplay {
    pub timestamp: String,
    pub model: String,
    pub kind: String,
    #[serde(rename = "requestsCosts", default)]
    pub requests_costs: Option<f64>,
    #[serde(rename = "usageBasedCosts")]
    pub usage_based_costs: String,
    #[serde(rename = "isTokenBasedCall")]
    pub is_token_based_call: bool,
    #[serde(rename = "tokenUsage", default)]
    pub token_usage: Option<TokenUsageDetail>,
    #[serde(rename = "owningUser")]
    pub owning_user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageDetail {
    #[serde(rename = "inputTokens")]
    pub input_tokens: Option<i32>,
    #[serde(rename = "outputTokens")]
    pub output_tokens: Option<i32>,
    #[serde(rename = "cacheWriteTokens")]
    pub cache_write_tokens: Option<i32>,
    #[serde(rename = "cacheReadTokens")]
    pub cache_read_tokens: Option<i32>,
    #[serde(rename = "totalCents")]
    pub total_cents: Option<f64>,
}

/// /api/auth/me 接口响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthMeResponse {
    pub email: String,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub sub: Option<String>,
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub picture: Option<String>,
}

/// 使用 session_token 获取用户信息（调用 cursor.com/api/auth/me）
pub async fn get_user_info(session_token: &str) -> Result<CursorUserInfo, String> {
    let client = create_proxy_client()?;

    let response = client
        .get("https://cursor.com/api/auth/me")
        .header(
            "Cookie",
            format!("WorkosCursorSessionToken={}", session_token),
        )
        .header("Accept", "*/*")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("User info request failed: {}", e))?;

    let status_code = response.status().as_u16();
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    if status_code != 200 {
        return Err(format!(
            "Get user info failed (HTTP {}): {}",
            status_code, body
        ));
    }

    // 解析响应 JSON
    let auth_me: AuthMeResponse = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse auth/me response: {}", e))?;

    // 提取 user_id：优先使用 sub 字段，否则使用 id
    let user_id = auth_me.sub.unwrap_or_else(|| {
        auth_me
            .id
            .map(|i| i.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    });

    Ok(CursorUserInfo {
        id: user_id,
        email: auth_me.email,
        name: auth_me.name,
    })
}

/// 获取用量摘要（使用 session_token + Cookie 认证）
pub async fn get_usage_summary(session_token: &str) -> Result<UsageSummary, String> {
    let client = create_proxy_client()?;

    let response = client
        .get("https://cursor.com/api/usage-summary")
        .header(
            "Cookie",
            format!("WorkosCursorSessionToken={}", session_token),
        )
        .header("Accept", "*/*")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Usage summary request failed: {}", e))?;

    let status = response.status();

    if status.is_success() {
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        serde_json::from_str::<UsageSummary>(&body)
            .map_err(|e| format!("Failed to parse usage summary: {}", e))
    } else {
        Ok(UsageSummary {
            membership_type: Some("free".to_string()),
            individual_usage: None,
            billing_cycle_start: None,
            billing_cycle_end: None,
        })
    }
}

/// 获取聚合用量数据
pub async fn get_aggregated_usage_data(
    workos_session_token: &str,
    start_date: u64,
    end_date: u64,
    team_id: i32,
) -> Result<Option<AggregatedUsageData>, String> {
    let client = create_proxy_client()?;

    let request_body = serde_json::json!({
        "startDate": start_date,
        "endDate": end_date,
        "teamId": team_id
    });

    let response = client
        .post("https://cursor.com/api/dashboard/get-aggregated-usage-events")
        .header(
            "Cookie",
            format!("WorkosCursorSessionToken={}", workos_session_token),
        )
        .header("Accept", "*/*")
        .header("Content-Type", "application/json")
        .header("Origin", "https://cursor.com")
        .header("Referer", "https://cursor.com/cn/dashboard")
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(40))
        .send()
        .await
        .map_err(|e| format!("Aggregated usage request failed: {}", e))?;

    if response.status().is_success() {
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&body) {
            let mut aggregations = Vec::new();

            if let Some(agg_array) = json_data.get("aggregations").and_then(|v| v.as_array()) {
                for agg in agg_array {
                    if let Some(model_intent) = agg.get("modelIntent").and_then(|v| v.as_str()) {
                        let model_usage = ModelUsage {
                            model_intent: model_intent.to_string(),
                            input_tokens: agg
                                .get("inputTokens")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0")
                                .to_string(),
                            output_tokens: agg
                                .get("outputTokens")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0")
                                .to_string(),
                            cache_write_tokens: agg
                                .get("cacheWriteTokens")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0")
                                .to_string(),
                            cache_read_tokens: agg
                                .get("cacheReadTokens")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0")
                                .to_string(),
                            total_cents: agg
                                .get("totalCents")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0),
                        };
                        aggregations.push(model_usage);
                    }
                }
            }

            let aggregated_usage = AggregatedUsageData {
                aggregations,
                total_input_tokens: json_data
                    .get("totalInputTokens")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0")
                    .to_string(),
                total_output_tokens: json_data
                    .get("totalOutputTokens")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0")
                    .to_string(),
                total_cache_write_tokens: json_data
                    .get("totalCacheWriteTokens")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0")
                    .to_string(),
                total_cache_read_tokens: json_data
                    .get("totalCacheReadTokens")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0")
                    .to_string(),
                total_cost_cents: json_data
                    .get("totalCostCents")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            };

            return Ok(Some(aggregated_usage));
        }
    }

    Ok(None)
}

/// 获取过滤的使用事件
pub async fn get_filtered_usage_events(
    workos_session_token: &str,
    team_id: i32,
    start_date: Option<&str>,
    end_date: Option<&str>,
    page: Option<i32>,
    page_size: Option<i32>,
) -> Result<Option<FilteredUsageEventsData>, String> {
    let client = create_proxy_client()?;

    let mut request_body = serde_json::json!({
        "teamId": team_id
    });
    if let Some(sd) = start_date {
        request_body["startDate"] = serde_json::Value::String(sd.to_string());
    }
    if let Some(ed) = end_date {
        request_body["endDate"] = serde_json::Value::String(ed.to_string());
    }
    if let Some(p) = page {
        request_body["page"] = serde_json::Value::Number(p.into());
    }
    if let Some(ps) = page_size {
        request_body["pageSize"] = serde_json::Value::Number(ps.into());
    }

    let response = client
        .post("https://cursor.com/api/dashboard/get-filtered-usage-events")
        .header(
            "Cookie",
            format!("WorkosCursorSessionToken={}", workos_session_token),
        )
        .header("Accept", "application/json, text/plain, */*")
        .header("Content-Type", "application/json")
        .header("Origin", "https://cursor.com")
        .header("Referer", "https://cursor.com/dashboard")
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(40))
        .send()
        .await
        .map_err(|e| format!("Filtered usage events request failed: {}", e))?;

    if response.status().is_success() {
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if let Ok(events_data) = serde_json::from_str::<FilteredUsageEventsData>(&body) {
            return Ok(Some(events_data));
        }
    }

    Ok(None)
}

// ============================================================================
// PKCE 授权流程：从 WorkosCursorSessionToken 获取 AccessToken
// ============================================================================

/// AccessToken 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessTokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken", default)]
    pub refresh_token: Option<String>,
    #[serde(rename = "authId", default)]
    pub auth_id: Option<String>,
}

/// 生成 PKCE verifier (32字节随机数，Base64URL编码)
fn generate_pkce_verifier() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// 生成 PKCE challenge (SHA256(verifier) 的 Base64URL编码)
fn generate_pkce_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(result)
}

/// 触发授权登录 - 用 session token 自动确认授权
async fn trigger_authorization_login(
    uuid: &str,
    challenge: &str,
    session_token: &str,
) -> Result<(), String> {
    let client = create_proxy_client()?;

    let payload = serde_json::json!({
        "challenge": challenge,
        "uuid": uuid,
    });

    let response = client
        .post("https://cursor.com/api/auth/loginDeepCallbackControl")
        .header(
            "Cookie",
            format!("WorkosCursorSessionToken={}", session_token),
        )
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Authorization request failed: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Authorization failed: {}", error_text))
    }
}

/// 轮询获取 AccessToken
async fn poll_for_access_token(
    uuid: &str,
    verifier: &str,
) -> Result<Option<AccessTokenResponse>, String> {
    let client = create_proxy_client()?;

    let url = format!(
        "https://api2.cursor.sh/auth/poll?uuid={}&verifier={}",
        uuid, verifier
    );

    let response = client
        .get(&url)
        .header("Accept", "*/*")
        .header("Content-Type", "application/json")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .map_err(|e| format!("Poll request failed: {}", e))?;

    if response.status().is_success() {
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read poll response: {}", e))?;

        // 尝试解析为 AccessTokenResponse
        if let Ok(token_response) = serde_json::from_str::<AccessTokenResponse>(&body) {
            return Ok(Some(token_response));
        }
    }

    Ok(None)
}

/// 从 WorkosCursorSessionToken 获取 AccessToken
///
/// 1. 生成 PKCE verifier 和 challenge
/// 2. 触发授权登录（用 session token 自动确认）
/// 3. 轮询获取 accessToken
///
/// # 参数
/// - `session_token`: WorkosCursorSessionToken (网页登录后的 cookie)
///
/// # 返回
/// - 成功: AccessTokenResponse 包含 accessToken 和 refreshToken
/// - 失败: 错误信息
pub async fn get_access_token_from_session(
    session_token: &str,
) -> Result<AccessTokenResponse, String> {
    // 1. 生成 PKCE 参数
    let uuid = uuid::Uuid::new_v4().to_string();
    let verifier = generate_pkce_verifier();
    let challenge = generate_pkce_challenge(&verifier);

    // 2. 触发授权登录
    trigger_authorization_login(&uuid, &challenge, session_token).await?;

    // 3. 轮询获取 token (最多尝试 20 次，每次间隔 1 秒)
    for _ in 0..20 {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        if let Some(token_response) = poll_for_access_token(&uuid, &verifier).await? {
            return Ok(token_response);
        }
    }

    Err("Timeout: Failed to get access token after 20 seconds".to_string())
}
