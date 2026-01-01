use super::traits::{
    AccountStorage,
    AccountSyncManager,
    AccountSyncStatus,
    ClientAccountSyncRequest,
    ServerAccountSyncResponse,
};
use super::{AntigravityLocalStorage, AntigravityPostgreSQLStorage};
use crate::platforms::antigravity::models::Account;
use chrono::Utc;
use std::sync::Arc;

pub struct AntigravityDualStorage {
    local_storage: Arc<AntigravityLocalStorage>,
    postgres_storage: Option<Arc<AntigravityPostgreSQLStorage>>,
    prefer_database: bool,
}

impl AntigravityDualStorage {
    pub fn new(
        local_storage: Arc<AntigravityLocalStorage>,
        postgres_storage: Option<Arc<AntigravityPostgreSQLStorage>>,
        prefer_database: bool,
    ) -> Self {
        Self {
            local_storage,
            postgres_storage,
            prefer_database,
        }
    }

    pub fn set_prefer_database(&mut self, prefer: bool) {
        self.prefer_database = prefer;
    }

    pub fn is_database_available(&self) -> bool {
        self.postgres_storage.is_some()
    }

    pub async fn get_current_account_id(&self) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        self.local_storage.get_current_account_id().await
    }

    pub async fn set_current_account_id(&self, account_id: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.local_storage.set_current_account_id(account_id).await
    }

    async fn save_to_local_storage(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Err(e) = self.local_storage.save_account(account).await {
            eprintln!("Failed to save to local storage: {}", e);
        }

        Ok(())
    }

    async fn delete_from_local_storage(&self, account_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let deleted = self.local_storage.delete_account(account_id).await?;
        Ok(deleted)
    }
}

#[async_trait::async_trait]
impl AccountStorage for AntigravityDualStorage {
    async fn save_account(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.save_to_local_storage(account).await
    }

    async fn load_accounts(&self) -> Result<Vec<Account>, Box<dyn std::error::Error + Send + Sync>> {
        self.local_storage.load_accounts().await
    }

    async fn update_account(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.save_account(account).await
    }

    async fn delete_account(&self, account_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        self.delete_from_local_storage(account_id).await
    }

    async fn get_account(&self, account_id: &str) -> Result<Option<Account>, Box<dyn std::error::Error + Send + Sync>> {
        self.local_storage.get_account(account_id).await
    }

    async fn clear_all_accounts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.local_storage.clear_all_accounts().await?;

        if let Some(postgres) = &self.postgres_storage {
            if postgres.is_available().await {
                let _ = postgres.clear_all_accounts().await;
            }
        }

        Ok(())
    }

    fn storage_type(&self) -> &'static str {
        if self.postgres_storage.is_some() {
            "antigravity_dual_storage"
        } else {
            "antigravity_local_only"
        }
    }

    async fn is_available(&self) -> bool {
        self.local_storage.is_available().await
    }
}

#[async_trait::async_trait]
impl AccountSyncManager for AntigravityDualStorage {
    async fn sync_local_to_remote(&self) -> Result<AccountSyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        let postgres = self.postgres_storage.as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        let local_accounts = self.local_storage.load_accounts().await?;
        let mut synced_count = 0;
        let mut errors = Vec::new();

        for account in &local_accounts {
            if let Err(e) = postgres.save_account(account).await {
                errors.push(format!("Failed to sync account {}: {}", account.id, e));
            } else {
                synced_count += 1;
            }
        }

        let status = if errors.is_empty() { "success" } else { "partial_success" };

        Ok(AccountSyncStatus {
            last_sync_at: Some(Utc::now()),
            sync_direction: "local_to_remote".to_string(),
            status: status.to_string(),
            error_message: if errors.is_empty() { None } else { Some(errors.join("; ")) },
            accounts_synced: synced_count,
        })
    }

    async fn sync_remote_to_local(&self) -> Result<AccountSyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        let postgres = self.postgres_storage.as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        let remote_accounts = postgres.load_accounts().await?;
        let remote_accounts_len = remote_accounts.len();
        let current_account_id = self.local_storage.get_current_account_id().await.ok().flatten();
        let deletions = postgres.load_tombstones_since_version(0).await.unwrap_or_default();
        let new_version = postgres.get_max_version().await.unwrap_or(0);

        let selected_current = if let Some(id) = current_account_id {
            if remote_accounts.iter().any(|a| a.id == id) {
                Some(id)
            } else {
                None
            }
        } else {
            None
        };

        self.local_storage.replace_all(remote_accounts, deletions, new_version, selected_current).await?;

        Ok(AccountSyncStatus {
            last_sync_at: Some(Utc::now()),
            sync_direction: "remote_to_local".to_string(),
            status: "success".to_string(),
            error_message: None,
            accounts_synced: remote_accounts_len as i32,
        })
    }

    async fn bidirectional_sync(&self) -> Result<AccountSyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        let postgres = self.postgres_storage.as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        let local_accounts = self.local_storage.load_accounts().await?;
        let remote_accounts = postgres.load_accounts().await?;
        let resolved_accounts = self.resolve_conflicts(local_accounts.clone(), remote_accounts.clone()).await?;

        let mut synced_count = 0;
        for account in &resolved_accounts {
            let mut local_ok = false;
            let mut remote_ok = false;

            if self.local_storage.save_account(account).await.is_ok() {
                local_ok = true;
            }

            if postgres.save_account(account).await.is_ok() {
                remote_ok = true;
            }

            if local_ok || remote_ok {
                synced_count += 1;
            }
        }

        Ok(AccountSyncStatus {
            last_sync_at: Some(Utc::now()),
            sync_direction: "bidirectional".to_string(),
            status: "success".to_string(),
            error_message: None,
            accounts_synced: synced_count,
        })
    }

    async fn bidirectional_sync_with_accounts(&self, local_accounts: Vec<Account>) -> Result<AccountSyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        let postgres = self.postgres_storage.as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        let remote_accounts = postgres.load_accounts().await?;
        let resolved_accounts = self.resolve_conflicts(local_accounts.clone(), remote_accounts.clone()).await?;

        let mut synced_count = 0;
        for account in &resolved_accounts {
            let mut local_ok = false;
            let mut remote_ok = false;

            if self.local_storage.save_account(account).await.is_ok() {
                local_ok = true;
            }

            if postgres.save_account(account).await.is_ok() {
                remote_ok = true;
            }

            if local_ok || remote_ok {
                synced_count += 1;
            }
        }

        Ok(AccountSyncStatus {
            last_sync_at: Some(Utc::now()),
            sync_direction: "bidirectional_with_memory".to_string(),
            status: "success".to_string(),
            error_message: None,
            accounts_synced: synced_count,
        })
    }

    async fn get_sync_status(&self) -> Result<Option<AccountSyncStatus>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(None)
    }

    async fn resolve_conflicts(&self, local_accounts: Vec<Account>, remote_accounts: Vec<Account>) -> Result<Vec<Account>, Box<dyn std::error::Error + Send + Sync>> {
        use std::collections::HashMap;

        let mut resolved = HashMap::new();

        for remote_account in remote_accounts {
            resolved.insert(remote_account.id.clone(), remote_account);
        }

        for local_account in local_accounts {
            if let Some(remote_account) = resolved.get(&local_account.id) {
                if local_account.updated_at > remote_account.updated_at {
                    resolved.insert(local_account.id.clone(), local_account);
                }
            } else {
                resolved.insert(local_account.id.clone(), local_account);
            }
        }

        Ok(resolved.into_values().collect())
    }

    async fn sync_accounts(&self, req: ClientAccountSyncRequest) -> Result<ServerAccountSyncResponse, Box<dyn std::error::Error + Send + Sync>> {
        let postgres = self.postgres_storage.as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        for change in &req.upserts {
            let account = &change.account;
            if let Ok(Some(existing)) = postgres.get_account(&account.id).await {
                if account.updated_at > existing.updated_at {
                    let _ = postgres.save_account_with_version(account).await;
                }
            } else {
                let _ = postgres.save_account_with_version(account).await;
            }
        }

        for deletion in &req.deletions {
            let _ = postgres.delete_account_with_tombstone(&deletion.id).await;
        }

        let server_upserts = postgres.load_accounts_since_version(req.last_version).await?;
        let server_deletions = postgres.load_tombstones_since_version(req.last_version).await?;
        let new_version = postgres.get_max_version().await?;

        let all_accounts = postgres.load_accounts().await?;
        let all_deletions = postgres.load_tombstones_since_version(0).await.unwrap_or_default();
        let current_account_id = self.local_storage.get_current_account_id().await.ok().flatten();
        let selected_current = if let Some(id) = current_account_id {
            if all_accounts.iter().any(|a| a.id == id) {
                Some(id)
            } else {
                None
            }
        } else {
            None
        };

        if let Err(e) = self.local_storage.replace_all(all_accounts, all_deletions, new_version, selected_current).await {
            eprintln!("Failed to replace local accounts: {}", e);
        }

        Ok(ServerAccountSyncResponse {
            upserts: server_upserts,
            deletions: server_deletions,
            new_version,
        })
    }
}
