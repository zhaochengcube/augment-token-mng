use super::traits::{TokenStorage, TokenData};
use crate::database::{DatabaseManager, DbPool};
use std::sync::Arc;

pub struct PostgreSQLStorage {
    pub db_manager: Arc<DatabaseManager>,
}

impl PostgreSQLStorage {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    async fn get_pool(&self) -> Result<Arc<DbPool>, Box<dyn std::error::Error + Send + Sync>> {
        self.db_manager.get_pool()
            .ok_or_else(|| "Database not connected".into())
    }
}

#[async_trait::async_trait]
impl TokenStorage for PostgreSQLStorage {
    async fn save_token(&self, token: &TokenData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        // 使用UPSERT (INSERT ... ON CONFLICT)
        // 注意：我们需要保留传入的 updated_at，而不是让触发器自动更新
        // 因此在 UPDATE 时显式设置 updated_at，触发器会被这个值覆盖
        client.execute(
            r#"
            INSERT INTO tokens (id, tenant_url, access_token, created_at, updated_at, portal_url, email_note, tag_name, tag_color, ban_status, portal_info, auth_session, suspensions, balance_color_mode, skip_check)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (id) DO UPDATE SET
                tenant_url = EXCLUDED.tenant_url,
                access_token = EXCLUDED.access_token,
                updated_at = EXCLUDED.updated_at,
                portal_url = EXCLUDED.portal_url,
                email_note = EXCLUDED.email_note,
                tag_name = EXCLUDED.tag_name,
                tag_color = EXCLUDED.tag_color,
                ban_status = EXCLUDED.ban_status,
                portal_info = EXCLUDED.portal_info,
                auth_session = EXCLUDED.auth_session,
                suspensions = EXCLUDED.suspensions,
                balance_color_mode = EXCLUDED.balance_color_mode,
                skip_check = EXCLUDED.skip_check
            "#,
            &[
                &token.id,
                &token.tenant_url,
                &token.access_token,
                &token.created_at,
                &token.updated_at,
                &token.portal_url,
                &token.email_note,
                &token.tag_name,
                &token.tag_color,
                &token.ban_status,
                &token.portal_info,
                &token.auth_session,
                &token.suspensions,
                &token.balance_color_mode,
                &token.skip_check,
            ],
        ).await?;

        Ok(())
    }

    async fn load_tokens(&self) -> Result<Vec<TokenData>, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        let rows = client.query(
            "SELECT id, tenant_url, access_token, created_at, updated_at, portal_url, email_note, tag_name, tag_color, ban_status, portal_info, auth_session, suspensions, balance_color_mode, skip_check FROM tokens ORDER BY created_at DESC",
            &[],
        ).await?;

        let mut tokens = Vec::new();
        for row in rows {
            let token = TokenData {
                id: row.get(0),
                tenant_url: row.get(1),
                access_token: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
                portal_url: row.get(5),
                email_note: row.get(6),
                tag_name: row.get(7),
                tag_color: row.get(8),
                ban_status: row.get(9),
                portal_info: row.get(10),
                auth_session: row.get(11),
                suspensions: row.get(12),
                balance_color_mode: row.get(13),
                skip_check: row.get(14),
            };
            tokens.push(token);
        }

        Ok(tokens)
    }

    async fn update_token(&self, token: &TokenData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        // 使用 token 中的 updated_at，而不是自动生成新的时间戳
        // 这样可以保持双向同步时的时间戳一致性
        let rows_affected = client.execute(
            r#"
            UPDATE tokens SET
                tenant_url = $2,
                access_token = $3,
                updated_at = $4,
                portal_url = $5,
                email_note = $6,
                tag_name = $7,
                tag_color = $8,
                ban_status = $9,
                portal_info = $10,
                auth_session = $11,
                suspensions = $12,
                balance_color_mode = $13,
                skip_check = $14
            WHERE id = $1
            "#,
            &[
                &token.id,
                &token.tenant_url,
                &token.access_token,
                &token.updated_at,
                &token.portal_url,
                &token.email_note,
                &token.tag_name,
                &token.tag_color,
                &token.ban_status,
                &token.portal_info,
                &token.auth_session,
                &token.suspensions,
                &token.balance_color_mode,
                &token.skip_check,
            ],
        ).await?;

        if rows_affected == 0 {
            return Err("Token not found for update".into());
        }

        Ok(())
    }

    async fn delete_token(&self, token_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        let rows_affected = client.execute(
            "DELETE FROM tokens WHERE id = $1",
            &[&token_id],
        ).await?;

        Ok(rows_affected > 0)
    }

    async fn get_token(&self, token_id: &str) -> Result<Option<TokenData>, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        let rows = client.query(
            "SELECT id, tenant_url, access_token, created_at, updated_at, portal_url, email_note, tag_name, tag_color, ban_status, portal_info, auth_session, suspensions, balance_color_mode, skip_check FROM tokens WHERE id = $1",
            &[&token_id],
        ).await?;

        if let Some(row) = rows.first() {
            let token = TokenData {
                id: row.get(0),
                tenant_url: row.get(1),
                access_token: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
                portal_url: row.get(5),
                email_note: row.get(6),
                tag_name: row.get(7),
                tag_color: row.get(8),
                ban_status: row.get(9),
                portal_info: row.get(10),
                auth_session: row.get(11),
                suspensions: row.get(12),
                balance_color_mode: row.get(13),
                skip_check: row.get(14),
            };
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    async fn clear_all_tokens(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        client.execute("DELETE FROM tokens", &[]).await?;
        Ok(())
    }

    fn storage_type(&self) -> &'static str {
        "postgresql"
    }

    async fn is_available(&self) -> bool {
        self.db_manager.is_connected() && self.db_manager.test_connection().await.is_ok()
    }
}

impl PostgreSQLStorage {
    /// 查找具有相同邮箱但不同ID的token
    /// 如果邮箱为空，则不查找重复项
    pub async fn find_duplicate_tokens(&self, email_note: Option<&str>, exclude_token_id: &str) -> Result<Vec<TokenData>, Box<dyn std::error::Error + Send + Sync>> {
        // 如果邮箱为空，直接返回空列表
        let email = match email_note {
            Some(e) if !e.trim().is_empty() => e.trim(),
            _ => return Ok(Vec::new()),
        };

        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        // 使用 LOWER() 进行不区分大小写的比较
        let rows = client.query(
            "SELECT id, tenant_url, access_token, created_at, updated_at, portal_url, email_note, tag_name, tag_color, ban_status, portal_info, auth_session, suspensions, balance_color_mode, skip_check FROM tokens WHERE LOWER(TRIM(email_note)) = LOWER($1) AND id != $2",
            &[&email, &exclude_token_id],
        ).await?;

        let mut tokens = Vec::new();
        for row in rows {
            let token = TokenData {
                id: row.get(0),
                tenant_url: row.get(1),
                access_token: row.get(2),
                created_at: row.get(3),
                updated_at: row.get(4),
                portal_url: row.get(5),
                email_note: row.get(6),
                tag_name: row.get(7),
                tag_color: row.get(8),
                ban_status: row.get(9),
                portal_info: row.get(10),
                auth_session: row.get(11),
                suspensions: row.get(12),
                balance_color_mode: row.get(13),
                skip_check: row.get(14),
            };
            tokens.push(token);
        }

        Ok(tokens)
    }
}

// 辅助函数：记录同步状态
pub async fn record_sync_status(
    pool: &DbPool,
    sync_direction: &str,
    status: &str,
    error_message: Option<&str>,
    tokens_synced: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    client.execute(
        r#"
        INSERT INTO sync_status (last_sync_at, sync_direction, status, error_message, tokens_synced)
        VALUES (NOW(), $1, $2, $3, $4)
        "#,
        &[&sync_direction, &status, &error_message, &tokens_synced],
    ).await?;

    Ok(())
}

// 辅助函数：获取最新的同步状态
pub async fn get_latest_sync_status(
    pool: &DbPool,
) -> Result<Option<super::traits::SyncStatus>, Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    
    let rows = client.query(
        "SELECT last_sync_at, sync_direction, status, error_message, tokens_synced FROM sync_status ORDER BY created_at DESC LIMIT 1",
        &[],
    ).await?;

    if let Some(row) = rows.first() {
        let sync_status = super::traits::SyncStatus {
            last_sync_at: row.get(0),
            sync_direction: row.get(1),
            status: row.get(2),
            error_message: row.get(3),
            tokens_synced: row.get(4),
        };
        Ok(Some(sync_status))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{DatabaseConfig, DatabaseManager};

    async fn create_test_storage() -> Option<PostgreSQLStorage> {
        // 这需要一个真实的测试数据库连接
        // 在实际测试中，你需要设置测试数据库
        let config = DatabaseConfig::new(
            "localhost".to_string(),
            5432,
            "test_augment_tokens".to_string(),
            "postgres".to_string(),
            "password".to_string(),
        );

        let mut db_manager = DatabaseManager::new(config);
        if db_manager.initialize().await.is_ok() {
            Some(PostgreSQLStorage::new(Arc::new(db_manager)))
        } else {
            None
        }
    }

    #[tokio::test]
    #[ignore] // 忽略这个测试，因为它需要真实的数据库连接
    async fn test_postgres_storage_operations() {
        if let Some(storage) = create_test_storage().await {
            // 测试保存token
            let token = TokenData::new(
                "test_id".to_string(),
                "https://example.com".to_string(),
                "test_token".to_string(),
                Some("https://portal.example.com".to_string()),
                Some("test note".to_string()),
            );

            assert!(storage.save_token(&token).await.is_ok());

            // 测试加载tokens
            let loaded_tokens = storage.load_tokens().await.unwrap();
            assert!(!loaded_tokens.is_empty());

            // 测试获取单个token
            let retrieved_token = storage.get_token("test_id").await.unwrap();
            assert!(retrieved_token.is_some());

            // 测试删除token
            let deleted = storage.delete_token("test_id").await.unwrap();
            assert!(deleted);
        }
    }
}
