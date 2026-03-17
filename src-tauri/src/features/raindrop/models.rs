use serde::{Deserialize, Serialize};

/// Raindrop.io API 返回的书签对象
#[derive(Debug, Clone, Deserialize)]
pub struct RaindropItem {
    #[serde(rename = "_id")]
    pub id: i64,
    pub title: Option<String>,
    pub link: String,
    pub excerpt: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub cover: Option<String>,
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    #[serde(default)]
    pub important: bool,
    pub collection: Option<RaindropCollection>,
    pub created: Option<String>,
    pub last_update: Option<String>,
    #[serde(default)]
    pub removed: bool,
}

/// Raindrop 所属集合（嵌套对象）
#[derive(Debug, Clone, Deserialize)]
pub struct RaindropCollection {
    #[serde(rename = "$id")]
    pub id: i64,
}

/// Raindrop.io 集合对象
#[derive(Debug, Clone, Deserialize)]
pub struct RaindropCollectionInfo {
    #[serde(rename = "_id")]
    pub id: i64,
    pub title: String,
    pub color: Option<String>,
    pub count: Option<i64>,
}

/// GET /rest/v1/raindrops/{collectionId} 响应
#[derive(Debug, Clone, Deserialize)]
pub struct RaindropsResponse {
    pub result: bool,
    #[serde(default)]
    pub items: Vec<RaindropItem>,
    pub count: Option<i64>,
}

/// GET /rest/v1/collections 响应
#[derive(Debug, Clone, Deserialize)]
pub struct CollectionsResponse {
    pub result: bool,
    #[serde(default)]
    pub items: Vec<RaindropCollectionInfo>,
}

/// 同步配置（存储在本地 JSON 文件中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaindropConfig {
    /// Raindrop.io Test Token
    pub token: String,
    /// 上次同步时间（ISO 8601 格式），用于增量同步
    #[serde(default)]
    pub last_sync_at: Option<String>,
}

/// 同步结果
#[derive(Debug, Clone, Serialize)]
pub struct RaindropSyncResult {
    pub total_fetched: usize,
    pub created: usize,
    pub updated: usize,
    pub skipped: usize,
    pub failed: usize,
    /// 新增或更新的书签 ID 列表（用于前端触发同步队列）
    pub affected_ids: Vec<String>,
}
