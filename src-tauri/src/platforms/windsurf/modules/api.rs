//! Windsurf API 模块
//! 实现配额查询、获取 API Key 等功能

use crate::http_client::create_proxy_client;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanInfo {
    pub plan_name: Option<String>,
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

    let body = serde_json::json!({
        "auth_token": access_token
    });

    let response = client
        .post(WINDSURF_PLAN_STATUS_API)
        .header("Content-Type", "application/json")
        .header("X-Auth-Token", access_token)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .header("x-client-version", "Chrome/JsCore/11.0.0/FirebaseCore-web")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("GetPlanStatus request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("GetPlanStatus failed ({}): {}", status, error_text));
    }

    let resp: PlanStatusResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse PlanStatus response: {}", e))?;

    let plan_status = resp.plan_status.unwrap_or_default();

    // 计算积分 (原始值除以 100)
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

    let plan_name = plan_status
        .plan_info
        .and_then(|p| p.plan_name)
        .unwrap_or_else(|| "Free".to_string());

    Ok(QuotaResult {
        plan_name,
        used_credits,
        total_credits,
        usage_percentage,
        expires_at: plan_status.plan_end,
        plan_start: plan_status.plan_start,
    })
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
        }
    }
}
