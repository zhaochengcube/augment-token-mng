use serde::{Deserialize, Serialize};
use crate::platforms::antigravity::models::Account;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSyncStatus {
    pub last_sync_at: Option<DateTime<Utc>>,
    pub sync_direction: String,
    pub status: String,
    pub error_message: Option<String>,
    pub accounts_synced: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAccountChange {
    pub account: Account,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAccountDelete {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAccountSyncRequest {
    pub last_version: i64,
    pub upserts: Vec<ClientAccountChange>,
    pub deletions: Vec<ClientAccountDelete>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerAccountSyncResponse {
    pub upserts: Vec<Account>,
    pub deletions: Vec<String>,
    pub new_version: i64,
}

#[async_trait::async_trait]
pub trait AccountStorage: Send + Sync {
    async fn save_account(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn load_accounts(&self) -> Result<Vec<Account>, Box<dyn std::error::Error + Send + Sync>>;

    async fn update_account(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    async fn delete_account(&self, account_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_account(&self, account_id: &str) -> Result<Option<Account>, Box<dyn std::error::Error + Send + Sync>>;

    async fn clear_all_accounts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn storage_type(&self) -> &'static str;

    async fn is_available(&self) -> bool;
}

#[async_trait::async_trait]
pub trait AccountSyncManager: Send + Sync {
    async fn sync_local_to_remote(&self) -> Result<AccountSyncStatus, Box<dyn std::error::Error + Send + Sync>>;

    async fn sync_remote_to_local(&self) -> Result<AccountSyncStatus, Box<dyn std::error::Error + Send + Sync>>;

    async fn bidirectional_sync(&self) -> Result<AccountSyncStatus, Box<dyn std::error::Error + Send + Sync>>;

    async fn bidirectional_sync_with_accounts(&self, local_accounts: Vec<Account>) -> Result<AccountSyncStatus, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_sync_status(&self) -> Result<Option<AccountSyncStatus>, Box<dyn std::error::Error + Send + Sync>>;

    async fn resolve_conflicts(&self, local_accounts: Vec<Account>, remote_accounts: Vec<Account>) -> Result<Vec<Account>, Box<dyn std::error::Error + Send + Sync>>;

    async fn sync_accounts(&self, req: ClientAccountSyncRequest) -> Result<ServerAccountSyncResponse, Box<dyn std::error::Error + Send + Sync>>;
}
