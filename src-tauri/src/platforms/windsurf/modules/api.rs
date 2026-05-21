//! Windsurf API 模块
//! 实现配额查询、获取 API Key 等功能

use crate::http_client::create_proxy_client;
use crate::proxy_helper::ProxyClient;
use crate::windsurf::models::Account;
use crate::windsurf::modules::devin;
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Windsurf API 端点
const WINDSURF_REGISTER_API: &str =
    "https://register.windsurf.com/exa.seat_management_pb.SeatManagementService/RegisterUser";
const WINDSURF_PLAN_STATUS_API: &str =
    "https://web-backend.windsurf.com/exa.seat_management_pb.SeatManagementService/GetPlanStatus";

/// RegisterUser API 响应
#[derive(Debug, Deserialize)]
pub struct RegisterUserResponse {
    pub api_key: Option<String>,
    pub name: Option<String>,
    pub api_server_url: Option<String>,
}

/// PlanStatus API 响应
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStatusResponse {
    pub plan_status: Option<PlanStatus>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanStatus {
    pub plan_info: Option<PlanInfo>,
    pub available_prompt_credits: Option<i64>,
    pub available_flex_credits: Option<i64>,
    pub used_prompt_credits: Option<i64>,
    pub used_flex_credits: Option<i64>,
    pub plan_start: Option<String>,
    pub plan_end: Option<String>,
    #[serde(default, alias = "daily_quota_remaining_percent")]
    pub daily_quota_remaining_percent: Option<i64>,
    #[serde(default, alias = "weekly_quota_remaining_percent")]
    pub weekly_quota_remaining_percent: Option<i64>,
    #[serde(default, alias = "daily_quota_reset_at_unix")]
    pub daily_quota_reset_at_unix: Option<i64>,
    #[serde(default, alias = "weekly_quota_reset_at_unix")]
    pub weekly_quota_reset_at_unix: Option<i64>,
    #[serde(default, alias = "overage_balance_micros")]
    pub overage_balance_micros: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanInfo {
    pub plan_name: Option<String>,
    #[serde(default, alias = "billing_strategy")]
    pub billing_strategy: Option<i64>,
}

/// 配额查询结果
#[derive(Debug, Clone, Serialize)]
pub struct QuotaResult {
    pub plan_name: String,
    pub used_credits: i64,
    pub total_credits: i64,
    pub usage_percentage: i64,
    pub expires_at: Option<String>,
    pub plan_start: Option<String>,
    pub billing_strategy: Option<i64>,
    pub daily_quota_remaining_percent: Option<i64>,
    pub weekly_quota_remaining_percent: Option<i64>,
    pub daily_quota_reset_at_unix: Option<i64>,
    pub weekly_quota_reset_at_unix: Option<i64>,
    pub overage_balance_micros: Option<i64>,
}

/// 使用 Firebase ID Token 获取 Windsurf API Key
pub async fn get_api_key(id_token: &str) -> Result<RegisterUserResponse, String> {
    let client = create_proxy_client()?;

    let body = serde_json::json!({
        "firebase_id_token": id_token
    });

    let response = client
        .post(WINDSURF_REGISTER_API)
        .header("Content-Type", "application/json")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("RegisterUser request failed: {}", e))?;

    if response.status().is_success() {
        response
            .json::<RegisterUserResponse>()
            .await
            .map_err(|e| format!("Failed to parse RegisterUser response: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("RegisterUser failed: {}", error_text))
    }
}

/// 查询账号配额信息
pub async fn get_plan_status(access_token: &str) -> Result<QuotaResult, String> {
    let client = create_proxy_client()?;
    request_plan_status(client, access_token, None).await
}

pub async fn get_plan_status_for_account(account: &Account) -> Result<QuotaResult, String> {
    let client = create_proxy_client()?;
    request_plan_status(client, &account.token.access_token, Some(account)).await
}

async fn request_plan_status(
    client: ProxyClient,
    access_token: &str,
    account: Option<&Account>,
) -> Result<QuotaResult, String> {
    let mut body = Vec::new();
    devin::encode_proto_string_field(&mut body, 1, access_token);

    let request = client
        .post(WINDSURF_PLAN_STATUS_API)
        .body(body)
        .header("accept", "*/*")
        .header("accept-language", "zh-CN,zh;q=0.9")
        .header("cache-control", "no-cache")
        .header("connect-protocol-version", "1")
        .header("content-type", "application/proto")
        .header("pragma", "no-cache")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-site")
        .header("Referer", "https://windsurf.com/")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        );

    let request = if let Some(account) = account {
        devin::apply_account_auth_headers(request, account)
    } else {
        request.header("x-auth-token", access_token)
    };

    let response = request
        .send()
        .await
        .map_err(|e| format!("GetPlanStatus request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("GetPlanStatus failed ({}): {}", status, error_text));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read PlanStatus response: {}", e))?;

    parse_plan_status_proto(&bytes).or_else(|proto_err| {
        parse_plan_status_json(&bytes).map_err(|json_err| {
            format!(
                "Failed to parse PlanStatus response: {}; {}",
                proto_err, json_err
            )
        })
    })
}

fn parse_plan_status_json(bytes: &[u8]) -> Result<QuotaResult, String> {
    let resp: PlanStatusResponse =
        serde_json::from_slice(bytes).map_err(|e| format!("JSON parse failed: {}", e))?;
    let plan_status = resp.plan_status.unwrap_or_default();
    quota_result_from_json_plan_status(plan_status)
}

fn quota_result_from_json_plan_status(plan_status: PlanStatus) -> Result<QuotaResult, String> {
    let prompt_credits = plan_status.available_prompt_credits.unwrap_or(0) / 100;
    let flex_credits = plan_status.available_flex_credits.unwrap_or(0) / 100;
    let total_credits = prompt_credits + flex_credits;

    let used_prompt = plan_status.used_prompt_credits.unwrap_or(0) / 100;
    let used_flex = plan_status.used_flex_credits.unwrap_or(0) / 100;
    let used_credits = used_prompt + used_flex;

    let usage_percentage = if total_credits > 0 {
        (used_credits * 100) / total_credits
    } else {
        0
    };

    let plan_info = plan_status.plan_info;
    let plan_name = plan_info
        .as_ref()
        .and_then(|p| p.plan_name.clone())
        .unwrap_or_else(|| "Free".to_string());
    let billing_strategy = plan_info.and_then(|p| p.billing_strategy);

    Ok(QuotaResult {
        plan_name,
        used_credits,
        total_credits,
        usage_percentage,
        expires_at: plan_status.plan_end,
        plan_start: plan_status.plan_start,
        billing_strategy,
        daily_quota_remaining_percent: plan_status.daily_quota_remaining_percent,
        weekly_quota_remaining_percent: plan_status.weekly_quota_remaining_percent,
        daily_quota_reset_at_unix: plan_status.daily_quota_reset_at_unix,
        weekly_quota_reset_at_unix: plan_status.weekly_quota_reset_at_unix,
        overage_balance_micros: plan_status.overage_balance_micros,
    })
}

fn parse_plan_status_proto(bytes: &[u8]) -> Result<QuotaResult, String> {
    let decoded = decode_proto_response(bytes)?;
    let mut parser = devin::ProtobufParser::new(decoded);
    let parsed = parser.parse_message()?;
    let plan_status = parsed.get("subMesssage_1").unwrap_or(&parsed);
    let plan_info = plan_status.get("subMesssage_1");

    let prompt_credits = plan_status
        .get("int_8")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        / 100;
    let flex_credits = plan_status
        .get("int_4")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        / 100;
    let total_credits = prompt_credits + flex_credits;

    let used_prompt = plan_status
        .get("int_6")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        / 100;
    let used_flex = plan_status
        .get("int_7")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        / 100;
    let used_credits = used_prompt + used_flex;

    let usage_percentage = if total_credits > 0 {
        (used_credits * 100) / total_credits
    } else {
        0
    };

    let plan_name = plan_info
        .and_then(|p| p.get("string_2"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| "Free".to_string());
    let billing_strategy = plan_info
        .and_then(|p| p.get("int_35"))
        .and_then(Value::as_i64);

    Ok(QuotaResult {
        plan_name,
        used_credits,
        total_credits,
        usage_percentage,
        expires_at: plan_status
            .get("subMesssage_3")
            .and_then(|v| v.get("int_1"))
            .and_then(Value::as_i64)
            .and_then(timestamp_to_rfc3339),
        plan_start: plan_status
            .get("subMesssage_2")
            .and_then(|v| v.get("int_1"))
            .and_then(Value::as_i64)
            .and_then(timestamp_to_rfc3339),
        billing_strategy,
        daily_quota_remaining_percent: Some(
            plan_status
                .get("int_14")
                .and_then(Value::as_i64)
                .unwrap_or(0),
        ),
        weekly_quota_remaining_percent: Some(
            plan_status
                .get("int_15")
                .and_then(Value::as_i64)
                .unwrap_or(0),
        ),
        daily_quota_reset_at_unix: plan_status.get("int_17").and_then(Value::as_i64),
        weekly_quota_reset_at_unix: plan_status.get("int_18").and_then(Value::as_i64),
        overage_balance_micros: Some(
            plan_status
                .get("int_16")
                .and_then(Value::as_i64)
                .unwrap_or(0),
        ),
    })
}

fn decode_proto_response(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.starts_with(b"data:application/proto;base64,") {
        return general_purpose::STANDARD
            .decode(&bytes["data:application/proto;base64,".len()..])
            .map_err(|e| format!("Base64 proto decode failed: {}", e));
    }
    Ok(bytes.to_vec())
}

fn timestamp_to_rfc3339(timestamp: i64) -> Option<String> {
    chrono::DateTime::from_timestamp(timestamp, 0).map(|dt| dt.to_rfc3339())
}

impl Default for PlanStatus {
    fn default() -> Self {
        Self {
            plan_info: None,
            available_prompt_credits: None,
            available_flex_credits: None,
            used_prompt_credits: None,
            used_flex_credits: None,
            plan_start: None,
            plan_end: None,
            daily_quota_remaining_percent: None,
            weekly_quota_remaining_percent: None,
            daily_quota_reset_at_unix: None,
            weekly_quota_reset_at_unix: None,
            overage_balance_micros: None,
        }
    }
}
