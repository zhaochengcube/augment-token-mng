//! Codex 相关数据模型
//!
//! 定义请求/响应格式转换所需的数据结构

use serde::{Deserialize, Serialize};

// ==================== 号池相关 ====================

/// Codex 号池中的账号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexPoolAccount {
    // ===== 基本信息 =====
    pub id: String,
    pub email: String,

    // ===== Token 信息 =====
    pub access_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    pub expires_at: i64,

    // ===== OpenAI 账号信息 =====
    pub chatgpt_account_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chatgpt_user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,

    // ===== 状态信息 =====
    pub is_active: bool,
    #[serde(default)]
    pub is_forbidden: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_refresh: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooldown_until: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error_status: Option<u16>,

    // ===== 配额统计 =====
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_quota: Option<i64>,
    #[serde(default)]
    pub used_quota: i64,
    #[serde(default)]
    pub total_tokens_used: i64,

    // ===== 配额信息（Smart 策略打分用） =====
    /// 5h 窗口已用百分比 (0-100)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_5h_used_percent: Option<f64>,
    /// 7d 窗口已用百分比 (0-100)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_7d_used_percent: Option<f64>,
    /// 订阅计划类型 (team / plus / pro / free)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,
    /// 订阅到期时间 (Unix timestamp)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_expires_at: Option<i64>,

    // ===== 标签 =====
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_color: Option<String>,
}

impl CodexPoolAccount {
    /// 从 OpenAI Account 转换为 CodexPoolAccount
    /// 返回 None 表示账号不可用（API 账号、无 token、被禁用等）
    pub fn from_openai_account(
        account: &crate::platforms::openai::models::Account,
    ) -> Option<Self> {
        let token = account.token.as_ref()?;

        // 跳过被禁用的账号
        if account.quota.as_ref().map(|q| q.is_forbidden).unwrap_or(false) {
            return None;
        }

        // 解析 openai_auth_json 中的订阅信息
        let (plan_type, subscription_expires_at) = account
            .openai_auth_json
            .as_deref()
            .and_then(|json_str| serde_json::from_str::<serde_json::Value>(json_str).ok())
            .map(|v| {
                let plan = v
                    .get("chatgpt_plan_type")
                    .and_then(|p| p.as_str())
                    .map(String::from);
                let expires = v
                    .get("chatgpt_subscription_active_until")
                    .and_then(|e| e.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.timestamp());
                (plan, expires)
            })
            .unwrap_or((None, None));

        // 提取配额百分比
        let codex_5h_used_percent = account
            .quota
            .as_ref()
            .and_then(|q| q.codex_5h_used_percent);
        let codex_7d_used_percent = account
            .quota
            .as_ref()
            .and_then(|q| q.codex_7d_used_percent);

        // 从 quota 获取 is_forbidden
        let is_forbidden = account
            .quota
            .as_ref()
            .map(|q| q.is_forbidden)
            .unwrap_or(false);

        Some(Self {
            id: account.id.clone(),
            email: account.email.clone(),
            access_token: token.access_token.clone(),
            refresh_token: token.refresh_token.clone(),
            id_token: token.id_token.clone(),
            expires_at: token.expires_at,
            chatgpt_account_id: account
                .chatgpt_account_id
                .clone()
                .unwrap_or_else(|| account.email.clone()),
            chatgpt_user_id: account.chatgpt_user_id.clone(),
            organization_id: account.organization_id.clone(),
            is_active: !Self::token_is_expired(token),
            is_forbidden,
            last_used: Some(account.last_used),
            last_refresh: None,
            cooldown_until: None,
            unavailable_reason: None,
            last_error_status: None,
            daily_quota: None,
            used_quota: 0,
            total_tokens_used: 0,
            codex_5h_used_percent,
            codex_7d_used_percent,
            plan_type,
            subscription_expires_at,
            tag: account.tag.clone(),
            tag_color: account.tag_color.clone(),
        })
    }

    /// 检查是否需要刷新 token
    pub fn needs_refresh(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        // 提前 5 分钟刷新
        self.expires_at - now < 300
    }

    /// 检查 token 是否已过期
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expires_at <= now
    }

    /// 检查账号是否处于冷却状态
    pub fn is_in_cooldown(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.cooldown_until.map(|ts| ts > now).unwrap_or(false)
    }

    /// 检查账号是否可用于请求
    pub fn is_available(&self) -> bool {
        self.is_active && !self.is_forbidden && !self.is_expired() && !self.is_in_cooldown()
    }

    fn token_is_expired(token: &crate::platforms::openai::models::TokenData) -> bool {
        let now = chrono::Utc::now().timestamp();
        token.expires_at <= now
    }

    /// 更新使用统计
    pub fn add_usage(&mut self, tokens: i64) {
        self.used_quota += tokens;
        self.total_tokens_used += tokens;
        self.last_used = Some(chrono::Utc::now().timestamp());
    }

    /// Smart 策略打分 (0-100)
    pub fn compute_score(&self) -> f64 {
        let now = chrono::Utc::now().timestamp();
        let mut score = 0.0;

        // 1. 5h 配额剩余 (40 分)
        score += match self.codex_5h_used_percent {
            Some(used) => (100.0 - used).max(0.0) / 100.0 * 40.0,
            None => 20.0, // 无数据给中间值
        };

        // 2. 7d 配额剩余 (15 分)
        score += match self.codex_7d_used_percent {
            Some(used) => (100.0 - used).max(0.0) / 100.0 * 15.0,
            None => 8.0,
        };

        // 3. 订阅到期距今 (40 分) - 已过期或快到期的优先消耗
        score += match self.subscription_expires_at {
            Some(expires) => {
                let days_left = (expires - now) as f64 / 86400.0;
                if days_left <= 0.0 {
                    // 已过期（宽限期内），最高优先级
                    40.0
                } else if days_left <= 3.0 {
                    38.0
                } else if days_left <= 7.0 {
                    30.0
                } else if days_left <= 15.0 {
                    18.0
                } else if days_left <= 30.0 {
                    8.0
                } else {
                    0.0
                }
            }
            None => 0.0,
        };

        // 4. 最近使用时间 (5 分) - 越久没用分越高，均衡负载
        score += match self.last_used {
            Some(ts) => {
                let idle_secs = (now - ts).max(0) as f64;
                // 空闲超过 5 分钟得满分，线性插值
                (idle_secs / 300.0).min(1.0) * 5.0
            }
            None => 5.0, // 从未使用，满分
        };

        score
    }
}

/// 号池选择策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoolStrategy {
    RoundRobin, // 轮询
    Single,     // 单个（固定账号）
    Smart,      // 智能选号（打分）
}

/// 号池状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatus {
    pub total_accounts: usize,
    pub active_accounts: usize,
    pub expired_accounts: usize,
    pub cooling_accounts: usize,
    pub unauthorized_accounts: usize,
    pub payment_required_accounts: usize,
    pub total_requests_today: u64,
    pub total_tokens_used: u64,
    pub strategy: PoolStrategy,
    #[serde(default)]
    pub selected_account_email: Option<String>, // Single 策略时选中的账号邮箱
}

// ==================== 日志相关 ====================

/// 请求日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    pub id: String,
    pub timestamp: i64,
    pub account_id: String,
    pub account_email: String,
    pub model: String,
    pub format: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub status: String,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub request_duration_ms: Option<i64>,
}

/// Token 统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub start_date: String,
    pub end_date: String,
    pub total_requests: u64,
    pub total_tokens: u64,
    pub per_account: Vec<AccountTokenStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTokenStats {
    pub account_id: String,
    pub email: String,
    pub requests: u64,
    pub tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTokenStats {
    pub model: String,
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodTokenStats {
    pub today_requests: u64,
    pub today_tokens: u64,
    pub week_requests: u64,
    pub week_tokens: u64,
    pub month_requests: u64,
    pub month_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub requests: u64,
    pub tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStatsResponse {
    pub stats: Vec<DailyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogQuery {
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub start_ts: Option<i64>,
    #[serde(default)]
    pub end_ts: Option<i64>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogPage {
    pub total: usize,
    pub items: Vec<RequestLog>,
}

// ==================== 错误处理 ====================

#[derive(Debug, thiserror::Error)]
pub enum CodexError {
    #[error("No available account in pool")]
    NoAvailableAccount,

    #[error("All accounts are expired")]
    AllAccountsExpired,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Translation error: {0}")]
    TranslationError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Token refresh required for account: {0}")]
    TokenRefreshRequired(String),
}

// Warp Rejection implementation
impl warp::reject::Reject for CodexError {}
