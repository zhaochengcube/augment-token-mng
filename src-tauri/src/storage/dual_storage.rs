use super::traits::{TokenStorage, TokenData, SyncManager, SyncStatus};
use super::{LocalFileStorage, PostgreSQLStorage};
use std::sync::Arc;
use chrono::Utc;

pub struct DualStorage {
    local_storage: Arc<LocalFileStorage>,
    postgres_storage: Option<Arc<PostgreSQLStorage>>,
    prefer_database: bool,
}

impl DualStorage {
    pub fn new(
        local_storage: Arc<LocalFileStorage>,
        postgres_storage: Option<Arc<PostgreSQLStorage>>,
    ) -> Self {
        Self {
            local_storage,
            postgres_storage,
            prefer_database: true,
        }
    }

    pub fn set_prefer_database(&mut self, prefer: bool) {
        self.prefer_database = prefer;
    }

    pub fn is_database_available(&self) -> bool {
        self.postgres_storage.is_some()
    }

    async fn sync_to_both_storages(&self, token: &TokenData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 总是保存到本地存储
        if let Err(e) = self.local_storage.save_token(token).await {
            eprintln!("Failed to save to local storage: {}", e);
        }

        // 如果数据库可用，也保存到数据库
        if let Some(postgres) = &self.postgres_storage {
            if postgres.is_available().await {
                if let Err(e) = postgres.save_token(token).await {
                    eprintln!("Failed to save to database: {}", e);
                    // 数据库保存失败不应该影响整体操作
                }
            }
        }

        Ok(())
    }

    async fn delete_from_both_storages(&self, token_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut local_deleted = false;
        let mut db_deleted = false;

        // 从本地存储删除
        if let Ok(deleted) = self.local_storage.delete_token(token_id).await {
            local_deleted = deleted;
        }

        // 从数据库删除
        if let Some(postgres) = &self.postgres_storage {
            if postgres.is_available().await {
                if let Ok(deleted) = postgres.delete_token(token_id).await {
                    db_deleted = deleted;
                }
            }
        }

        Ok(local_deleted || db_deleted)
    }

    /// 删除token及其在数据库中的重复项
    async fn delete_token_and_duplicates(&self, token_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        eprintln!("Starting delete_token_and_duplicates for token_id: {}", token_id);

        // 首先获取要删除的token信息，用于查找重复项
        let token_info = if let Some(postgres) = &self.postgres_storage {
            if postgres.is_available().await {
                eprintln!("Database is available, trying to get token from database");
                match postgres.get_token(token_id).await {
                    Ok(Some(token)) => {
                        eprintln!("Found token in database: tenant_url={}, access_token={}", token.tenant_url, token.access_token);
                        Some(token)
                    },
                    Ok(None) => {
                        eprintln!("Token not found in database, trying local storage");
                        // 如果数据库中没有，尝试从本地存储获取
                        match self.local_storage.get_token(token_id).await {
                            Ok(token) => {
                                if let Some(token) = token {
                                    eprintln!("Found token in local storage: tenant_url={}, access_token={}", token.tenant_url, token.access_token);
                                    Some(token)
                                } else {
                                    eprintln!("Token not found in local storage either");
                                    None
                                }
                            },
                            Err(e) => {
                                eprintln!("Error getting token from local storage: {}", e);
                                None
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Error getting token from database: {}, trying local storage", e);
                        // 数据库查询失败，尝试从本地存储获取
                        match self.local_storage.get_token(token_id).await {
                            Ok(token) => {
                                if let Some(token) = token {
                                    eprintln!("Found token in local storage: tenant_url={}, access_token={}", token.tenant_url, token.access_token);
                                    Some(token)
                                } else {
                                    eprintln!("Token not found in local storage either");
                                    None
                                }
                            },
                            Err(e) => {
                                eprintln!("Error getting token from local storage: {}", e);
                                None
                            }
                        }
                    }
                }
            } else {
                eprintln!("Database not available, trying local storage");
                // 数据库不可用，从本地存储获取
                match self.local_storage.get_token(token_id).await {
                    Ok(token) => {
                        if let Some(token) = token {
                            eprintln!("Found token in local storage: tenant_url={}, access_token={}", token.tenant_url, token.access_token);
                            Some(token)
                        } else {
                            eprintln!("Token not found in local storage");
                            None
                        }
                    },
                    Err(e) => {
                        eprintln!("Error getting token from local storage: {}", e);
                        None
                    }
                }
            }
        } else {
            eprintln!("No database storage configured, trying local storage");
            // 没有数据库存储，从本地存储获取
            match self.local_storage.get_token(token_id).await {
                Ok(token) => {
                    if let Some(token) = token {
                        eprintln!("Found token in local storage: tenant_url={}, access_token={}", token.tenant_url, token.access_token);
                        Some(token)
                    } else {
                        eprintln!("Token not found in local storage");
                        None
                    }
                },
                Err(e) => {
                    eprintln!("Error getting token from local storage: {}", e);
                    None
                }
            }
        };

        // 执行原有的删除逻辑
        eprintln!("Executing main delete operation");
        let main_deleted = self.delete_from_both_storages(token_id).await?;
        eprintln!("Main delete result: {}", main_deleted);

        // 如果是双重存储且数据库可用，查找并删除重复的token（基于邮箱）
        if let Some(postgres) = &self.postgres_storage {
            if postgres.is_available().await {
                if let Some(token) = token_info {
                    // 只有当token有邮箱时才查找重复项
                    let email_note = token.email_note.as_deref();
                    if email_note.is_some() && !email_note.unwrap().trim().is_empty() {
                        eprintln!("Looking for duplicate tokens with email_note={:?}", email_note);
                        // 查找数据库中的重复token（基于邮箱）
                        match postgres.find_duplicate_tokens(email_note, token_id).await {
                            Ok(duplicate_tokens) => {
                                eprintln!("Found {} duplicate tokens", duplicate_tokens.len());
                                let mut duplicate_deleted_count = 0;
                                for duplicate_token in duplicate_tokens {
                                    eprintln!("Attempting to delete duplicate token with ID: {}", duplicate_token.id);
                                    // 删除每个重复的token
                                    if let Ok(deleted) = postgres.delete_token(&duplicate_token.id).await {
                                        if deleted {
                                            duplicate_deleted_count += 1;
                                            eprintln!("Successfully deleted duplicate token with ID: {}", duplicate_token.id);
                                        } else {
                                            eprintln!("Failed to delete duplicate token with ID: {} (not found)", duplicate_token.id);
                                        }
                                    } else {
                                        eprintln!("Error deleting duplicate token with ID: {}", duplicate_token.id);
                                    }
                                }
                                if duplicate_deleted_count > 0 {
                                    eprintln!("Deleted {} duplicate tokens", duplicate_deleted_count);
                                } else {
                                    eprintln!("No duplicate tokens were deleted");
                                }
                            },
                            Err(e) => {
                                eprintln!("Failed to find duplicate tokens: {}", e);
                            }
                        }
                    } else {
                        eprintln!("Token has no email_note, skipping duplicate check");
                    }
                } else {
                    eprintln!("No token info available for finding duplicates");
                }
            } else {
                eprintln!("Database not available for duplicate deletion");
            }
        } else {
            eprintln!("No database storage for duplicate deletion");
        }

        Ok(main_deleted)
    }

    async fn load_from_preferred_storage(&self) -> Result<Vec<TokenData>, Box<dyn std::error::Error + Send + Sync>> {
        if self.prefer_database {
            if let Some(postgres) = &self.postgres_storage {
                if postgres.is_available().await {
                    match postgres.load_tokens().await {
                        Ok(tokens) => return Ok(tokens),
                        Err(e) => {
                            eprintln!("Failed to load from database, falling back to local: {}", e);
                        }
                    }
                }
            }
        }

        // 回退到本地存储
        self.local_storage.load_tokens().await
    }
}

#[async_trait::async_trait]
impl TokenStorage for DualStorage {
    async fn save_token(&self, token: &TokenData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.sync_to_both_storages(token).await
    }

    async fn load_tokens(&self) -> Result<Vec<TokenData>, Box<dyn std::error::Error + Send + Sync>> {
        self.load_from_preferred_storage().await
    }

    async fn update_token(&self, token: &TokenData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut updated_token = token.clone();
        updated_token.update_timestamp();
        self.sync_to_both_storages(&updated_token).await
    }

    async fn delete_token(&self, token_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        self.delete_token_and_duplicates(token_id).await
    }

    async fn get_token(&self, token_id: &str) -> Result<Option<TokenData>, Box<dyn std::error::Error + Send + Sync>> {
        // 优先从首选存储获取
        if self.prefer_database {
            if let Some(postgres) = &self.postgres_storage {
                if postgres.is_available().await {
                    match postgres.get_token(token_id).await {
                        Ok(Some(token)) => return Ok(Some(token)),
                        Ok(None) => {}, // 继续尝试本地存储
                        Err(e) => {
                            eprintln!("Failed to get token from database: {}", e);
                        }
                    }
                }
            }
        }

        // 从本地存储获取
        self.local_storage.get_token(token_id).await
    }

    async fn clear_all_tokens(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 清空本地存储
        if let Err(e) = self.local_storage.clear_all_tokens().await {
            eprintln!("Failed to clear local storage: {}", e);
        }

        // 清空数据库
        if let Some(postgres) = &self.postgres_storage {
            if postgres.is_available().await {
                if let Err(e) = postgres.clear_all_tokens().await {
                    eprintln!("Failed to clear database: {}", e);
                }
            }
        }

        Ok(())
    }

    fn storage_type(&self) -> &'static str {
        if self.is_database_available() {
            "dual_storage"
        } else {
            "local_only"
        }
    }

    async fn is_available(&self) -> bool {
        // 至少本地存储应该可用
        self.local_storage.is_available().await
    }
}

#[async_trait::async_trait]
impl SyncManager for DualStorage {
    async fn sync_local_to_remote(&self) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        let postgres = self.postgres_storage.as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        let local_tokens = self.local_storage.load_tokens().await?;
        let mut synced_count = 0;
        let mut errors = Vec::new();

        for token in local_tokens {
            match postgres.save_token(&token).await {
                Ok(_) => synced_count += 1,
                Err(e) => errors.push(format!("Token {}: {}", token.id, e)),
            }
        }

        let status = if errors.is_empty() { "success" } else { "partial_success" };
        let error_message = if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        };

        let sync_status = SyncStatus {
            last_sync_at: Some(Utc::now()),
            sync_direction: "local_to_remote".to_string(),
            status: status.to_string(),
            error_message: error_message.clone(),
            tokens_synced: synced_count,
        };

        // 记录同步状态到数据库
        if let Some(pool) = postgres.db_manager.get_pool() {
            let _ = super::postgres_storage::record_sync_status(
                &pool,
                &sync_status.sync_direction,
                &sync_status.status,
                error_message.as_deref(),
                sync_status.tokens_synced,
            ).await;
        }

        Ok(sync_status)
    }

    async fn sync_remote_to_local(&self) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        let postgres = self.postgres_storage.as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        let remote_tokens = postgres.load_tokens().await?;
        let mut synced_count = 0;
        let mut errors = Vec::new();

        for token in remote_tokens {
            match self.local_storage.save_token(&token).await {
                Ok(_) => synced_count += 1,
                Err(e) => errors.push(format!("Token {}: {}", token.id, e)),
            }
        }

        let status = if errors.is_empty() { "success" } else { "partial_success" };
        let error_message = if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        };

        let sync_status = SyncStatus {
            last_sync_at: Some(Utc::now()),
            sync_direction: "remote_to_local".to_string(),
            status: status.to_string(),
            error_message: error_message.clone(),
            tokens_synced: synced_count,
        };

        // 记录同步状态到数据库
        if let Some(pool) = postgres.db_manager.get_pool() {
            let _ = super::postgres_storage::record_sync_status(
                &pool,
                &sync_status.sync_direction,
                &sync_status.status,
                error_message.as_deref(),
                sync_status.tokens_synced,
            ).await;
        }

        Ok(sync_status)
    }

    async fn bidirectional_sync(&self) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        let postgres = self.postgres_storage.as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        let local_tokens = self.local_storage.load_tokens().await?;
        let remote_tokens = postgres.load_tokens().await?;

        let resolved_tokens = self.resolve_conflicts(local_tokens.clone(), remote_tokens.clone()).await?;

        let mut synced_count = 0;
        let mut errors = Vec::new();

        // 找出被去重淘汰的 token IDs
        use std::collections::HashSet;
        let resolved_ids: HashSet<String> = resolved_tokens.iter()
            .map(|t| t.id.clone())
            .collect();

        let all_ids: HashSet<String> = local_tokens.iter()
            .chain(remote_tokens.iter())
            .map(|t| t.id.clone())
            .collect();

        let removed_ids: Vec<String> = all_ids.difference(&resolved_ids)
            .cloned()
            .collect();

        // 从数据库中删除被淘汰的 tokens
        for removed_id in &removed_ids {
            eprintln!("Removing duplicate token from database: {}", removed_id);
            if let Err(e) = postgres.delete_token(removed_id).await {
                eprintln!("Failed to remove duplicate token {}: {}", removed_id, e);
                errors.push(format!("Failed to remove duplicate token {}: {}", removed_id, e));
            }
        }

        // 先清空本地存储，然后重新写入解决后的tokens
        if let Err(e) = self.local_storage.clear_all_tokens().await {
            errors.push(format!("Failed to clear local storage: {}", e));
        }

        // 同步解决后的tokens到两个存储
        for token in resolved_tokens {
            let mut local_ok = false;
            let mut remote_ok = false;

            if let Ok(_) = self.local_storage.save_token(&token).await {
                local_ok = true;
            }

            if let Ok(_) = postgres.save_token(&token).await {
                remote_ok = true;
            }

            if local_ok || remote_ok {
                synced_count += 1;
            } else {
                errors.push(format!("Failed to sync token {}", token.id));
            }
        }

        let status = if errors.is_empty() { "success" } else { "partial_success" };
        let error_message = if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        };

        let sync_status = SyncStatus {
            last_sync_at: Some(Utc::now()),
            sync_direction: "bidirectional".to_string(),
            status: status.to_string(),
            error_message: error_message.clone(),
            tokens_synced: synced_count,
        };

        // 记录同步状态到数据库
        if let Some(pool) = postgres.db_manager.get_pool() {
            let _ = super::postgres_storage::record_sync_status(
                &pool,
                &sync_status.sync_direction,
                &sync_status.status,
                error_message.as_deref(),
                sync_status.tokens_synced,
            ).await;
        }

        Ok(sync_status)
    }

    async fn bidirectional_sync_with_tokens(&self, local_tokens: Vec<TokenData>) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        let postgres = self.postgres_storage.as_ref()
            .ok_or("Database storage not available")?;

        if !postgres.is_available().await {
            return Err("Database not available".into());
        }

        // 使用传入的 local_tokens 而不是从文件读取
        let remote_tokens = postgres.load_tokens().await?;

        let resolved_tokens = self.resolve_conflicts(local_tokens.clone(), remote_tokens.clone()).await?;

        let mut synced_count = 0;
        let mut errors = Vec::new();

        // 找出被去重淘汰的 token IDs
        use std::collections::HashSet;
        let resolved_ids: HashSet<String> = resolved_tokens.iter()
            .map(|t| t.id.clone())
            .collect();

        let all_ids: HashSet<String> = local_tokens.iter()
            .chain(remote_tokens.iter())
            .map(|t| t.id.clone())
            .collect();

        let removed_ids: Vec<String> = all_ids.difference(&resolved_ids)
            .cloned()
            .collect();

        // 从数据库中删除被淘汰的 tokens
        for removed_id in &removed_ids {
            eprintln!("Removing duplicate token from database: {}", removed_id);
            if let Err(e) = postgres.delete_token(removed_id).await {
                eprintln!("Failed to remove duplicate token {}: {}", removed_id, e);
                errors.push(format!("Failed to remove duplicate token {}: {}", removed_id, e));
            }
        }

        // 先清空本地存储，然后重新写入解决后的tokens
        if let Err(e) = self.local_storage.clear_all_tokens().await {
            errors.push(format!("Failed to clear local storage: {}", e));
        }

        // 同步解决后的tokens到两个存储
        for token in resolved_tokens {
            let mut local_ok = false;
            let mut remote_ok = false;

            if let Ok(_) = self.local_storage.save_token(&token).await {
                local_ok = true;
            }

            if let Ok(_) = postgres.save_token(&token).await {
                remote_ok = true;
            }

            if local_ok || remote_ok {
                synced_count += 1;
            } else {
                errors.push(format!("Failed to sync token {}", token.id));
            }
        }

        let status = if errors.is_empty() { "success" } else { "partial_success" };
        let error_message = if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        };

        let sync_status = SyncStatus {
            last_sync_at: Some(Utc::now()),
            sync_direction: "bidirectional_with_memory".to_string(),
            status: status.to_string(),
            error_message: error_message.clone(),
            tokens_synced: synced_count,
        };

        // 记录同步状态到数据库
        if let Some(pool) = postgres.db_manager.get_pool() {
            let _ = super::postgres_storage::record_sync_status(
                &pool,
                &sync_status.sync_direction,
                &sync_status.status,
                error_message.as_deref(),
                sync_status.tokens_synced,
            ).await;
        }

        Ok(sync_status)
    }

    async fn get_sync_status(&self) -> Result<Option<SyncStatus>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(postgres) = &self.postgres_storage {
            if postgres.is_available().await {
                if let Some(pool) = postgres.db_manager.get_pool() {
                    return super::postgres_storage::get_latest_sync_status(&pool).await;
                }
            }
        }
        // 如果数据库不可用，返回None
        Ok(None)
    }

    async fn resolve_conflicts(&self, local_tokens: Vec<TokenData>, remote_tokens: Vec<TokenData>) -> Result<Vec<TokenData>, Box<dyn std::error::Error + Send + Sync>> {
        use std::collections::HashMap;

        let mut resolved = HashMap::new();

        // 合并策略：保留所有tokens，以最新更新时间为准

        // 首先添加所有远程tokens
        for remote_token in remote_tokens {
            resolved.insert(remote_token.id.clone(), remote_token);
        }

        // 然后处理本地tokens
        for local_token in local_tokens {
            if let Some(remote_token) = resolved.get(&local_token.id) {
                // 远程也存在此token，比较更新时间，使用更新的版本
                if local_token.updated_at > remote_token.updated_at {
                    resolved.insert(local_token.id.clone(), local_token);
                }
                // 否则保持远程版本
            } else {
                // 远程不存在此token，添加本地token（新增的token）
                resolved.insert(local_token.id.clone(), local_token);
            }
        }

        // 邮箱去重：如果多个token有相同邮箱，只保留最新的一个
        let mut email_map: HashMap<String, TokenData> = HashMap::new();
        let mut tokens_without_email = Vec::new();

        for token in resolved.into_values() {
            if let Some(email) = &token.email_note {
                let email_key = email.trim().to_lowercase();
                if !email_key.is_empty() {
                    // 有邮箱的token，按邮箱去重
                    if let Some(existing) = email_map.get(&email_key) {
                        // 比较更新时间，保留更新的
                        if token.updated_at > existing.updated_at {
                            eprintln!("Email conflict: keeping newer token {} (updated_at={:?}) over {} (updated_at={:?}) for email {}",
                                token.id, token.updated_at, existing.id, existing.updated_at, email_key);
                            email_map.insert(email_key, token);
                        } else {
                            eprintln!("Email conflict: keeping existing token {} (updated_at={:?}) over {} (updated_at={:?}) for email {}",
                                existing.id, existing.updated_at, token.id, token.updated_at, email_key);
                        }
                    } else {
                        email_map.insert(email_key, token);
                    }
                } else {
                    // 邮箱为空字符串，不去重
                    tokens_without_email.push(token);
                }
            } else {
                // 没有邮箱的token，不去重
                tokens_without_email.push(token);
            }
        }

        // 合并有邮箱的token和没有邮箱的token
        let mut final_tokens: Vec<TokenData> = email_map.into_values().collect();
        final_tokens.extend(tokens_without_email);

        Ok(final_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_dual_storage_local_only() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("test_tokens.json");
        let local_storage = Arc::new(LocalFileStorage::new_with_path(storage_path));
        
        let dual_storage = DualStorage::new(local_storage, None);
        
        assert!(!dual_storage.is_database_available());
        assert_eq!(dual_storage.storage_type(), "local_only");
        assert!(dual_storage.is_available().await);

        // 测试基本操作
        let token = TokenData::new(
            "test_id".to_string(),
            "https://example.com".to_string(),
            "test_token".to_string(),
            None,
            None,
        );

        assert!(dual_storage.save_token(&token).await.is_ok());
        let loaded_tokens = dual_storage.load_tokens().await.unwrap();
        assert_eq!(loaded_tokens.len(), 1);
    }
}
