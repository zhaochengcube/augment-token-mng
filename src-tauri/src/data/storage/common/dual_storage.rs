use super::traits::{
    AccountStorage, AccountSyncManager, AccountSyncStatus, ClientAccountSyncRequest,
    ServerAccountSyncResponse, StorageError, SyncableAccount,
};
use super::{AccountDbMapper, GenericLocalStorage, GenericPostgreSQLStorage};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

/// 通用双向存储
pub struct GenericDualStorage<T, M>
where
    T: SyncableAccount,
    M: AccountDbMapper<T>,
{
    local_storage: Arc<GenericLocalStorage<T>>,
    postgres_storage: Option<Arc<GenericPostgreSQLStorage<T, M>>>,
    prefer_database: bool,
}

impl<T, M> GenericDualStorage<T, M>
where
    T: SyncableAccount,
    M: AccountDbMapper<T>,
{
    pub fn new(
        local_storage: Arc<GenericLocalStorage<T>>,
        postgres_storage: Option<Arc<GenericPostgreSQLStorage<T, M>>>,
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

    pub async fn get_current_account_id(&self) -> Result<Option<String>, StorageError> {
        self.local_storage.get_current_account_id().await
    }

    pub async fn set_current_account_id(
        &self,
        account_id: Option<String>,
    ) -> Result<(), StorageError> {
        self.local_storage.set_current_account_id(account_id).await
    }

    fn resolve_conflicts_impl(local: Vec<T>, remote: Vec<T>) -> Vec<T> {
        let mut resolved: HashMap<String, T> = HashMap::new();

        for r in remote {
            resolved.insert(r.id().to_string(), r);
        }

        for l in local {
            if let Some(r) = resolved.get(l.id()) {
                if l.updated_at() > r.updated_at() {
                    resolved.insert(l.id().to_string(), l);
                }
            } else {
                resolved.insert(l.id().to_string(), l);
            }
        }

        resolved.into_values().collect()
    }
}

#[async_trait::async_trait]
impl<T, M> AccountStorage<T> for GenericDualStorage<T, M>
where
    T: SyncableAccount,
    M: AccountDbMapper<T>,
{
    async fn save_account(&self, account: &T) -> Result<(), StorageError> {
        self.local_storage.save_account(account).await
    }

    async fn load_accounts(&self) -> Result<Vec<T>, StorageError> {
        self.local_storage.load_accounts().await
    }

    async fn get_account(&self, account_id: &str) -> Result<Option<T>, StorageError> {
        self.local_storage.get_account(account_id).await
    }

    async fn update_account(&self, account: &T) -> Result<(), StorageError> {
        self.save_account(account).await
    }

    async fn delete_account(&self, account_id: &str) -> Result<bool, StorageError> {
        self.local_storage.delete_account(account_id).await
    }

    async fn clear_all_accounts(&self) -> Result<(), StorageError> {
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
            match T::platform_name() {
                "antigravity" => "antigravity_dual_storage",
                "windsurf" => "windsurf_dual_storage",
                _ => "generic_dual_storage",
            }
        } else {
            match T::platform_name() {
                "antigravity" => "antigravity_local_only",
                "windsurf" => "windsurf_local_only",
                _ => "generic_local_only",
            }
        }
    }

    async fn is_available(&self) -> bool {
        self.local_storage.is_available().await
    }
}

#[async_trait::async_trait]
impl<T, M> AccountSyncManager<T> for GenericDualStorage<T, M>
where
    T: SyncableAccount,
    M: AccountDbMapper<T>,
{
    async fn sync_local_to_remote(&self) -> Result<AccountSyncStatus, StorageError> {
        let postgres = self
            .postgres_storage
            .as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        let local_accounts = self.local_storage.load_accounts().await?;
        let mut synced_count = 0;
        let mut errors = Vec::new();

        for account in &local_accounts {
            if let Err(e) = postgres.save_account(account).await {
                errors.push(format!("Failed to sync account {}: {}", account.id(), e));
            } else {
                synced_count += 1;
            }
        }

        let status = if errors.is_empty() {
            "success"
        } else {
            "partial_success"
        };

        Ok(AccountSyncStatus {
            last_sync_at: Some(Utc::now()),
            sync_direction: "local_to_remote".to_string(),
            status: status.to_string(),
            error_message: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
            accounts_synced: synced_count,
        })
    }

    async fn sync_remote_to_local(&self) -> Result<AccountSyncStatus, StorageError> {
        let postgres = self
            .postgres_storage
            .as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        let remote_accounts = postgres.load_accounts().await?;
        let remote_accounts_len = remote_accounts.len();
        let current_account_id = self
            .local_storage
            .get_current_account_id()
            .await
            .ok()
            .flatten();
        let deletions = postgres
            .load_tombstones_since_version(0)
            .await
            .unwrap_or_default();
        let new_version = postgres.get_max_version().await.unwrap_or(0);

        let selected_current = if let Some(id) = current_account_id {
            if remote_accounts.iter().any(|a| a.id() == id) {
                Some(id)
            } else {
                None
            }
        } else {
            None
        };

        self.local_storage
            .replace_all(remote_accounts, deletions, new_version, selected_current)
            .await?;

        Ok(AccountSyncStatus {
            last_sync_at: Some(Utc::now()),
            sync_direction: "remote_to_local".to_string(),
            status: "success".to_string(),
            error_message: None,
            accounts_synced: remote_accounts_len as i32,
        })
    }

    async fn bidirectional_sync(&self) -> Result<AccountSyncStatus, StorageError> {
        let postgres = self
            .postgres_storage
            .as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        let local_accounts = self.local_storage.load_accounts().await?;
        let remote_accounts = postgres.load_accounts().await?;
        let resolved_accounts = self
            .resolve_conflicts(local_accounts.clone(), remote_accounts.clone())
            .await?;

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

    async fn bidirectional_sync_with_accounts(
        &self,
        local_accounts: Vec<T>,
    ) -> Result<AccountSyncStatus, StorageError> {
        let postgres = self
            .postgres_storage
            .as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        let remote_accounts = postgres.load_accounts().await?;
        let resolved_accounts = self
            .resolve_conflicts(local_accounts.clone(), remote_accounts.clone())
            .await?;

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

    async fn get_sync_status(&self) -> Result<Option<AccountSyncStatus>, StorageError> {
        Ok(None)
    }

    async fn resolve_conflicts(
        &self,
        local: Vec<T>,
        remote: Vec<T>,
    ) -> Result<Vec<T>, StorageError> {
        Ok(Self::resolve_conflicts_impl(local, remote))
    }

    async fn sync_accounts(
        &self,
        req: ClientAccountSyncRequest<T>,
    ) -> Result<ServerAccountSyncResponse<T>, StorageError> {
        let postgres = self
            .postgres_storage
            .as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        // 处理客户端上传的变更
        for change in &req.upserts {
            let account = &change.account;
            if let Ok(Some(existing)) = postgres.get_account(account.id()).await {
                if account.updated_at() > existing.updated_at() {
                    if let Err(e) = postgres.save_account_with_version(account).await {
                        eprintln!(
                            "Failed to save account {} to postgres: {:?}",
                            account.id(),
                            e
                        );
                    }
                }
            } else {
                if let Err(e) = postgres.save_account_with_version(account).await {
                    eprintln!(
                        "Failed to save account {} to postgres: {:?}",
                        account.id(),
                        e
                    );
                }
            }
        }

        // 处理删除
        for deletion in &req.deletions {
            if let Err(e) = postgres.delete_account_with_tombstone(&deletion.id).await {
                eprintln!(
                    "Failed to delete account {} from postgres: {:?}",
                    deletion.id, e
                );
            }
        }

        // 获取服务端变更
        let server_upserts = match postgres.load_accounts_since_version(req.last_version).await {
            Ok(upserts) => upserts,
            Err(e) => {
                eprintln!(
                    "Failed to load accounts since version {}: {:?}",
                    req.last_version, e
                );
                return Err(e);
            }
        };
        let server_deletions = match postgres
            .load_tombstones_since_version(req.last_version)
            .await
        {
            Ok(deletions) => deletions,
            Err(e) => {
                eprintln!(
                    "Failed to load tombstones since version {}: {:?}",
                    req.last_version, e
                );
                return Err(e);
            }
        };
        let new_version = match postgres.get_max_version().await {
            Ok(version) => version,
            Err(e) => {
                eprintln!("Failed to get max version: {:?}", e);
                return Err(e);
            }
        };

        // 更新本地存储
        let all_accounts = match postgres.load_accounts().await {
            Ok(accounts) => accounts,
            Err(e) => {
                eprintln!("Failed to load all accounts: {:?}", e);
                return Err(e);
            }
        };
        let all_deletions = postgres
            .load_tombstones_since_version(0)
            .await
            .unwrap_or_default();
        let current_account_id = self
            .local_storage
            .get_current_account_id()
            .await
            .ok()
            .flatten();
        let selected_current = if let Some(id) = current_account_id {
            if all_accounts.iter().any(|a| a.id() == id) {
                Some(id)
            } else {
                None
            }
        } else {
            None
        };

        if let Err(e) = self
            .local_storage
            .replace_all(all_accounts, all_deletions, new_version, selected_current)
            .await
        {
            eprintln!("Failed to replace local accounts: {}", e);
        }

        Ok(ServerAccountSyncResponse {
            upserts: server_upserts,
            deletions: server_deletions,
            new_version,
        })
    }
}
