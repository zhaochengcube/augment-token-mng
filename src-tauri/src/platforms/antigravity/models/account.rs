use super::{QuotaData, TokenData};
use crate::data::storage::common::SyncableAccount;
use serde::{Deserialize, Serialize};

/// Antigravity 账号数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub token: TokenData,
    /// 账号绑定的设备指纹，用于切换时同步 Antigravity telemetry。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_profile: Option<DeviceProfile>,
    pub quota: Option<QuotaData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
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
        "antigravity"
    }

    fn merge_missing_fields(&mut self, source: &Self) {
        if self.device_profile.is_none() {
            self.device_profile = source.device_profile.clone();
        }
        if self.quota.is_none() {
            self.quota = source.quota.clone();
        }
        if self.tag.is_none() {
            self.tag = source.tag.clone();
        }
        if self.tag_color.is_none() {
            self.tag_color = source.tag_color.clone();
        }
        if self.token.project_id.is_none() {
            self.token.project_id = source.token.project_id.clone();
        }
        if self.token.oauth_client_key.is_none() {
            self.token.oauth_client_key = source.token.oauth_client_key.clone();
        }
        if self.token.session_id.is_none() {
            self.token.session_id = source.token.session_id.clone();
        }
        if !self.token.is_gcp_tos && source.token.is_gcp_tos {
            self.token.is_gcp_tos = true;
        }
        if self.token.id_token.is_none() {
            self.token.id_token = source.token.id_token.clone();
        }
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
            device_profile: None,
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
        }
    }

    pub fn update_last_used(&mut self) {
        self.last_used = chrono::Utc::now().timestamp();
        self.updated_at = self.last_used;
    }

    pub fn update_quota(&mut self, quota: QuotaData) {
        self.quota = Some(quota);
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

/// Antigravity storage.json 中 telemetry 相关设备指纹。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub machine_id: String,
    pub mac_machine_id: String,
    pub dev_device_id: String,
    pub sqm_id: String,
}

/// 账号索引数据（antigravity_accounts.json）
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
