use serde::{Serialize, de::DeserializeOwned};
use chrono::{DateTime, Utc};

pub type StorageError = Box<dyn std::error::Error + Send + Sync>;

/// 所有可同步账号必须实现的 trait
pub trait SyncableAccount: Clone + Send + Sync + Serialize + DeserializeOwned + 'static {
    fn id(&self) -> &str;
    fn email(&self) -> &str;
    fn updated_at(&self) -> i64;
    fn version(&self) -> i64;
    fn set_version(&mut self, version: i64);
    fn is_deleted(&self) -> bool;
    fn set_deleted(&mut self, deleted: bool);

    /// 平台标识符（用于表名、文件名等）
    fn platform_name() -> &'static str;

    /// 存储文件名
    fn storage_file_name() -> String {
        format!("{}_accounts.json", Self::platform_name())
    }

    /// 数据库表名
    fn table_name() -> String {
        format!("{}_accounts", Self::platform_name())
    }

    /// 版本序列名
    fn sequence_name() -> String {
        format!("{}_account_version_seq", Self::platform_name())
    }
}

/// 通用同步状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccountSyncStatus {
    pub last_sync_at: Option<DateTime<Utc>>,
    pub sync_direction: String,
    pub status: String,
    pub error_message: Option<String>,
    pub accounts_synced: i32,
}

/// 客户端账号变更
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientAccountChange<T> {
    pub account: T,
}

/// 客户端删除请求
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientAccountDelete {
    pub id: String,
}

/// 客户端同步请求
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientAccountSyncRequest<T> {
    pub last_version: i64,
    pub upserts: Vec<ClientAccountChange<T>>,
    pub deletions: Vec<ClientAccountDelete>,
}

/// 服务端同步响应
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerAccountSyncResponse<T> {
    pub upserts: Vec<T>,
    pub deletions: Vec<String>,
    pub new_version: i64,
}

/// 通用账号存储 trait
#[async_trait::async_trait]
pub trait AccountStorage<T: SyncableAccount>: Send + Sync {
    async fn save_account(&self, account: &T) -> Result<(), StorageError>;
    async fn load_accounts(&self) -> Result<Vec<T>, StorageError>;
    async fn get_account(&self, id: &str) -> Result<Option<T>, StorageError>;
    async fn update_account(&self, account: &T) -> Result<(), StorageError>;
    async fn delete_account(&self, id: &str) -> Result<bool, StorageError>;
    async fn clear_all_accounts(&self) -> Result<(), StorageError>;
    fn storage_type(&self) -> &'static str;
    async fn is_available(&self) -> bool;
}

/// 通用同步管理器 trait
#[async_trait::async_trait]
pub trait AccountSyncManager<T: SyncableAccount>: Send + Sync {
    async fn sync_local_to_remote(&self) -> Result<AccountSyncStatus, StorageError>;
    async fn sync_remote_to_local(&self) -> Result<AccountSyncStatus, StorageError>;
    async fn bidirectional_sync(&self) -> Result<AccountSyncStatus, StorageError>;
    async fn bidirectional_sync_with_accounts(&self, local_accounts: Vec<T>) -> Result<AccountSyncStatus, StorageError>;
    async fn get_sync_status(&self) -> Result<Option<AccountSyncStatus>, StorageError>;
    async fn resolve_conflicts(&self, local: Vec<T>, remote: Vec<T>) -> Result<Vec<T>, StorageError>;
    async fn sync_accounts(&self, req: ClientAccountSyncRequest<T>) -> Result<ServerAccountSyncResponse<T>, StorageError>;
}

