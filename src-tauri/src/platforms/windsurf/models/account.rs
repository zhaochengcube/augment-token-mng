use super::TokenData;
use crate::data::storage::common::SyncableAccount;
use serde::{Deserialize, Serialize};

/// 配额数据结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuotaData {
    /// 套餐名称: "Free", "Pro", "Enterprise"
    pub plan_name: String,
    /// 已使用积分
    pub used_credits: i64,
    /// 总积分
    pub total_credits: i64,
    /// 使用百分比
    pub usage_percentage: i64,
    /// 到期时间 (ISO 8601 格式)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// 套餐开始时间
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_start: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub billing_strategy: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_quota_remaining_percent: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weekly_quota_remaining_percent: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub daily_quota_reset_at_unix: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weekly_quota_reset_at_unix: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overage_balance_micros: Option<i64>,
    /// 最后更新时间
    pub last_updated: i64,
}

/// Windsurf 账号数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub token: TokenData,

    /// Windsurf API Key (用于切号)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// API 服务器地址
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_server_url: Option<String>,

    /// 配额信息
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<QuotaData>,

    /// 用户标签
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// 标签颜色
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_color: Option<String>,

    #[serde(default)]
    pub disabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_at: Option<i64>,
    pub created_at: i64,
    pub last_used: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub version: i64,
    #[serde(default)]
    pub deleted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub devin_auth1_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub devin_account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub devin_primary_org_id: Option<String>,
}

impl SyncableAccount for Account {
    fn id(&self) -> &str {
        &self.id
    }

    fn email(&self) -> &str {
        &self.email
    }

    fn updated_at(&self) -> i64 {
        self.updated_at
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn set_version(&mut self, version: i64) {
        self.version = version;
    }

    fn is_deleted(&self) -> bool {
        self.deleted
    }

    fn set_deleted(&mut self, deleted: bool) {
        self.deleted = deleted;
    }

    fn platform_name() -> &'static str {
        "windsurf"
    }
}

impl Account {
    pub fn new(id: String, email: String, token: TokenData) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            email,
            name: None,
            token,
            api_key: None,
            api_server_url: None,
            quota: None,
            tag: None,
            tag_color: None,
            disabled: false,
            disabled_reason: None,
            disabled_at: None,
            created_at: now,
            last_used: now,
            updated_at: now,
            version: 0,
            deleted: false,
            auth_provider: None,
            devin_auth1_token: None,
            devin_account_id: None,
            devin_primary_org_id: None,
        }
    }

    pub fn update_last_used(&mut self) {
        self.last_used = chrono::Utc::now().timestamp();
        self.updated_at = self.last_used;
    }

    pub fn is_devin_account(&self) -> bool {
        matches!(self.auth_provider.as_deref(), Some("devin"))
    }
}

/// 账号索引数据（windsurf_accounts.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountIndex {
    pub version: String,
    pub accounts: Vec<AccountSummary>,
    pub current_account_id: Option<String>,
}

/// 账号摘要信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub created_at: i64,
    pub last_used: i64,
}

impl AccountIndex {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            accounts: Vec::new(),
            current_account_id: None,
        }
    }
}

impl Default for AccountIndex {
    fn default() -> Self {
        Self::new()
    }
}
