use crate::data::storage::common::SyncableAccount;
use serde::{Deserialize, Serialize};

/// 订阅数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    /// 网站/服务名称
    pub website: String,
    /// 网站地址 (URL)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    /// 订阅开始时间 (ISO 8601 格式，如 "2025-01-01")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    /// 订阅时长 (月数)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_months: Option<i32>,
    /// 过期时间 (ISO 8601 格式，如 "2025-12-31")，可手动设置或根据 start_date + duration_months 自动计算
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry_date: Option<String>,
    /// 订阅费用
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    /// 标签
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// 标签颜色
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_color: Option<String>,
    /// 备注
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// 创建时间
    pub created_at: i64,
    /// 更新时间
    pub updated_at: i64,
    /// 版本号 (用于同步)
    #[serde(default)]
    pub version: i64,
    /// 是否已删除 (软删除)
    #[serde(default)]
    pub deleted: bool,
}

impl SyncableAccount for Subscription {
    fn id(&self) -> &str {
        &self.id
    }

    fn email(&self) -> &str {
        // 返回 website 作为标识
        &self.website
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
        "subscription"
    }

    /// 重写表名，因为数据库表是 `subscriptions` 而不是 `subscription_accounts`
    fn table_name() -> String {
        "subscriptions".to_string()
    }
}

impl Subscription {
    pub fn new(id: String, website: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            website,
            website_url: None,
            start_date: None,
            duration_months: None,
            expiry_date: None,
            cost: None,
            tag: None,
            tag_color: None,
            notes: None,
            created_at: now,
            updated_at: now,
            version: 0,
            deleted: false,
        }
    }
}
