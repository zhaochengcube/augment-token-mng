use crate::data::storage::common::SyncableAccount;
use serde::{Deserialize, Serialize};

/// 书签数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    /// 书签名称
    pub name: String,
    /// 书签地址 (URL)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// 描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 标签
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// 标签颜色
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_color: Option<String>,
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

impl SyncableAccount for Bookmark {
    fn id(&self) -> &str {
        &self.id
    }

    fn email(&self) -> &str {
        // 返回 name 作为标识
        &self.name
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
        "bookmark"
    }

    /// 重写表名
    fn table_name() -> String {
        "bookmarks".to_string()
    }
}

impl Bookmark {
    pub fn new(id: String, name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            name,
            url: None,
            description: None,
            tag: None,
            tag_color: None,
            created_at: now,
            updated_at: now,
            version: 0,
            deleted: false,
        }
    }
}
