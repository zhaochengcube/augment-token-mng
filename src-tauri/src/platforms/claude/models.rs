use serde::{Deserialize, Serialize};
use crate::data::storage::common::SyncableAccount;

/// Claude 账号数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub service_name: String,
    pub website_url: Option<String>,
    pub start_date: i64,
    pub duration_days: i64,
    pub expiry_date: i64,
    pub tag: Option<String>,
    pub tag_color: Option<String>,
    pub notes: Option<String>,
    pub base_url: String,
    pub auth_token: String,
    pub default_opus_model: String,
    pub default_sonnet_model: String,
    pub default_haiku_model: String,
    #[serde(default)]
    pub use_model: String,
    pub created_at: i64,
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
        &self.service_name
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
        "claude"
    }
}

impl Account {
    pub fn new(
        service_name: String,
        website_url: Option<String>,
        start_date: i64,
        duration_days: i64,
        base_url: String,
        auth_token: String,
        default_opus_model: String,
        default_sonnet_model: String,
        default_haiku_model: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        let id = format!("claude_{}", now);
        let expiry_date = start_date + (duration_days * 86400);

        Self {
            id,
            service_name,
            website_url,
            start_date,
            duration_days,
            expiry_date,
            tag: None,
            tag_color: None,
            notes: None,
            base_url,
            auth_token,
            default_opus_model,
            default_sonnet_model,
            default_haiku_model,
            use_model: String::from("default"),
            created_at: now,
            updated_at: now,
            version: 0,
            deleted: false,
        }
    }

    /// 检查账号是否已存在（服务名称相同）
    pub fn is_duplicate(service_name: &str, existing_accounts: &[Self]) -> bool {
        existing_accounts.iter().any(|a| &a.service_name == service_name)
    }
}
