use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountStatus {
    pub status: String, // "ACTIVE" | "INVALID_TOKEN" | "SUSPENDED" | "ERROR"
    pub error_message: Option<String>, // 仅在ERROR时使用
}

// 批量检测相关结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub access_token: String,
    pub tenant_url: String,
    pub id: Option<String>,                   // 用于前端识别是哪个token
    pub portal_url: Option<String>,           // Portal URL用于获取使用次数信息
    pub auth_session: Option<String>,         // Auth session用于自动刷新token
    pub email_note: Option<String>,           // 邮箱备注,用于判断是否需要获取邮箱
    pub should_refresh_session: Option<bool>, // 前端标记是否需要刷新 session
}

// Portal信息结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct PortalInfo {
    pub credits_balance: i32,
    pub credit_total: Option<i32>,
    pub expiry_date: Option<String>,
}

// get-models API 响应结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub user: UserData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub id: String,
    pub email: String,
    pub tenant_id: String,
    pub tenant_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenStatusResult {
    pub token_id: Option<String>, // 对应输入的id
    pub access_token: String,     // 保留token用于前端更新 (如果被刷新,这里是新token)
    pub tenant_url: String,       // 保留tenant_url用于前端更新 (如果被刷新,这里是新url)
    pub status_result: AccountStatus,
    pub portal_info: Option<PortalInfo>, // Portal信息（如果有）
    pub portal_error: Option<String>,    // Portal获取错误（如果有）
    pub suspensions: Option<Value>,      // 封禁详情（如果有）
    pub email_note: Option<String>,      // 邮箱备注（如果获取到）
    pub portal_url: Option<String>,      // Portal URL（如果获取到）
    pub auth_session: Option<String>,    // 刷新后的 auth_session（如果有）
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ChatMessage {
    pub response_text: String,
    pub request_message: String,
}

// ============ Credit Consumption API ============

/// Credit 消费数据点
#[derive(Debug, Serialize, Deserialize)]
pub struct CreditDataPoint {
    #[serde(rename(serialize = "group_key", deserialize = "groupKey"))]
    pub group_key: Option<String>, // 模型名称
    #[serde(rename(serialize = "date_range", deserialize = "dateRange"))]
    pub date_range: Option<DateRange>,
    #[serde(
        rename(serialize = "credits_consumed", deserialize = "creditsConsumed"),
        default = "default_credits_consumed"
    )]
    pub credits_consumed: String,
}

/// 默认值函数：当 creditsConsumed 字段缺失时返回 "0"
fn default_credits_consumed() -> String {
    "0".to_string()
}

/// 日期范围
#[derive(Debug, Serialize, Deserialize)]
pub struct DateRange {
    #[serde(
        rename(serialize = "start_date_iso", deserialize = "startDate"),
        alias = "startDateIso"
    )]
    pub start_date_iso: String,
    #[serde(
        rename(serialize = "end_date_iso", deserialize = "endDate"),
        alias = "endDateIso"
    )]
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
    pub portal_url: Option<String>, // 添加 portal_url 字段
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenFromSessionResponse {
    pub access_token: String,
    pub tenant_url: String,
    pub email: Option<String>,
}

// Session 刷新结果
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRefreshResult {
    pub token_id: String,
    pub success: bool,
    pub error: Option<String>,
    pub new_session: Option<String>,
}
