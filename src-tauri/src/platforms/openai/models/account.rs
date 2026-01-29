use serde::{Deserialize, Serialize};
use super::{TokenData, QuotaData};
use crate::data::storage::common::SyncableAccount;

/// OpenAI 账号数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub token: TokenData,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chatgpt_account_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chatgpt_user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    /// 配额信息
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quota: Option<QuotaData>,
    /// 标签
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// 标签颜色
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_color: Option<String>,
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
        "openai"
    }
}

impl Account {
    pub fn new(
        email: String,
        token: TokenData,
        chatgpt_account_id: Option<String>,
        chatgpt_user_id: Option<String>,
        organization_id: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();

        // 使用 chatgpt_account_id 作为 ID，如果没有则使用 email
        let id = chatgpt_account_id.clone().unwrap_or_else(|| email.clone());

        Self {
            id,
            email,
            token,
            chatgpt_account_id,
            chatgpt_user_id,
            organization_id,
            quota: None,
            tag: None,
            tag_color: None,
            created_at: now,
            last_used: now,
            updated_at: now,
            version: 0,
            deleted: false,
        }
    }

    /// 检查账号是否已存在（邮箱和 account_id 都相同）
    pub fn is_duplicate(base_email: &str, chatgpt_account_id: &Option<String>, existing_accounts: &[Self]) -> bool {
        existing_accounts.iter().any(|a| {
            &a.email == base_email && a.chatgpt_account_id == *chatgpt_account_id
        })
    }

    /// 生成唯一的邮箱（相同邮箱不同 account_id 时添加序号）
    pub fn generate_unique_email(base_email: &str, _chatgpt_account_id: &Option<String>, existing_accounts: &[Self]) -> String {
        // 检查是否已存在相同邮箱的账号（不管 account_id）
        let email_exists = existing_accounts.iter().any(|a| &a.email == base_email);

        if !email_exists {
            return base_email.to_string();
        }

        // 存在相同邮箱的账号，添加序号
        let mut suffix = 2;
        loop {
            let unique_email = format!("{} ({})", base_email, suffix);
            let exists = existing_accounts.iter().any(|a| &a.email == &unique_email);
            if !exists {
                return unique_email;
            }
            suffix += 1;
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

