use super::traits::{AccountStorage};
use crate::database::{DatabaseManager, DbPool};
use crate::platforms::antigravity::models::{Account, TokenData, QuotaData};
use chrono::Utc;
use std::sync::Arc;

pub struct AntigravityPostgreSQLStorage {
    pub db_manager: Arc<DatabaseManager>,
}

impl AntigravityPostgreSQLStorage {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    async fn get_pool(&self) -> Result<Arc<DbPool>, Box<dyn std::error::Error + Send + Sync>> {
        self.db_manager.get_pool()
            .ok_or_else(|| "Database not connected".into())
    }

    pub async fn get_next_version(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        let rows = client.query(
            "SELECT nextval('antigravity_account_version_seq')",
            &[],
        ).await?;

        if let Some(row) = rows.first() {
            let version: i64 = row.get(0);
            Ok(version)
        } else {
            Err("Failed to get next version".into())
        }
    }

    pub async fn load_accounts_since_version(&self, since_version: i64) -> Result<Vec<Account>, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        let rows = client.query(
            r#"
            SELECT id, email, name, access_token, refresh_token, expires_in, expiry_timestamp, token_type,
                   project_id, session_id, quota, disabled, disabled_reason, disabled_at,
                   created_at, last_used, updated_at, version
            FROM antigravity_accounts
            WHERE version > $1 AND deleted IS NOT TRUE
            ORDER BY version
            "#,
            &[&since_version],
        ).await?;

        let mut accounts = Vec::new();
        for row in rows {
            let quota_value: Option<serde_json::Value> = row.get(10);
            let quota = match quota_value {
                Some(value) => serde_json::from_value::<QuotaData>(value).ok(),
                None => None,
            };

            let email: String = row.get(1);
            let account = Account {
                id: row.get(0),
                email: email.clone(),
                name: row.get(2),
                token: TokenData {
                    access_token: row.get(3),
                    refresh_token: row.get(4),
                    expires_in: row.get(5),
                    expiry_timestamp: row.get(6),
                    token_type: row.get(7),
                    email: Some(email),
                    project_id: row.get(8),
                    session_id: row.get(9),
                },
                quota,
                disabled: row.get(11),
                disabled_reason: row.get(12),
                disabled_at: row.get(13),
                created_at: row.get(14),
                last_used: row.get(15),
                updated_at: row.get(16),
                version: row.get(17),
                deleted: false,
            };
            accounts.push(account);
        }

        Ok(accounts)
    }

    pub async fn load_tombstones_since_version(&self, since_version: i64) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        let rows = client.query(
            "SELECT id FROM antigravity_accounts WHERE deleted IS TRUE AND version > $1",
            &[&since_version],
        ).await?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.get(0));
        }

        Ok(ids)
    }

    pub async fn get_max_version(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        let rows = client.query(
            "SELECT COALESCE(MAX(version), 0) FROM antigravity_accounts",
            &[],
        ).await?;

        if let Some(row) = rows.first() {
            let version: i64 = row.get(0);
            Ok(version)
        } else {
            Ok(0)
        }
    }

    pub async fn save_account_with_version(&self, account: &Account) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;
        let new_version = self.get_next_version().await?;

        let quota_value = match &account.quota {
            Some(quota) => Some(serde_json::to_value(quota)?),
            None => None,
        };

        client.execute(
            r#"
            INSERT INTO antigravity_accounts (
                id, email, name, access_token, refresh_token, expires_in, expiry_timestamp, token_type,
                project_id, session_id, quota, disabled, disabled_reason, disabled_at,
                created_at, last_used, updated_at, deleted, version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            ON CONFLICT (id) DO UPDATE SET
                email = EXCLUDED.email,
                name = EXCLUDED.name,
                access_token = EXCLUDED.access_token,
                refresh_token = EXCLUDED.refresh_token,
                expires_in = EXCLUDED.expires_in,
                expiry_timestamp = EXCLUDED.expiry_timestamp,
                token_type = EXCLUDED.token_type,
                project_id = EXCLUDED.project_id,
                session_id = EXCLUDED.session_id,
                quota = EXCLUDED.quota,
                disabled = EXCLUDED.disabled,
                disabled_reason = EXCLUDED.disabled_reason,
                disabled_at = EXCLUDED.disabled_at,
                last_used = EXCLUDED.last_used,
                updated_at = EXCLUDED.updated_at,
                deleted = FALSE,
                version = EXCLUDED.version
            "#,
            &[
                &account.id,
                &account.email,
                &account.name,
                &account.token.access_token,
                &account.token.refresh_token,
                &account.token.expires_in,
                &account.token.expiry_timestamp,
                &account.token.token_type,
                &account.token.project_id,
                &account.token.session_id,
                &quota_value,
                &account.disabled,
                &account.disabled_reason,
                &account.disabled_at,
                &account.created_at,
                &account.last_used,
                &account.updated_at,
                &false,
                &new_version,
            ],
        ).await?;

        Ok(new_version)
    }

    pub async fn delete_account_with_tombstone(&self, account_id: &str) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;
        let new_version = self.get_next_version().await?;

        let rows_affected = client.execute(
            r#"
            UPDATE antigravity_accounts SET
                deleted = TRUE,
                version = $2,
                updated_at = $3
            WHERE id = $1
            "#,
            &[&account_id, &new_version, &Utc::now().timestamp()],
        ).await?;

        if rows_affected == 0 {
            Ok(0)
        } else {
            Ok(new_version)
        }
    }
}

#[async_trait::async_trait]
impl AccountStorage for AntigravityPostgreSQLStorage {
    async fn save_account(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        let quota_value = match &account.quota {
            Some(quota) => Some(serde_json::to_value(quota)?),
            None => None,
        };

        client.execute(
            r#"
            INSERT INTO antigravity_accounts (
                id, email, name, access_token, refresh_token, expires_in, expiry_timestamp, token_type,
                project_id, session_id, quota, disabled, disabled_reason, disabled_at,
                created_at, last_used, updated_at, deleted
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            ON CONFLICT (id) DO UPDATE SET
                email = EXCLUDED.email,
                name = EXCLUDED.name,
                access_token = EXCLUDED.access_token,
                refresh_token = EXCLUDED.refresh_token,
                expires_in = EXCLUDED.expires_in,
                expiry_timestamp = EXCLUDED.expiry_timestamp,
                token_type = EXCLUDED.token_type,
                project_id = EXCLUDED.project_id,
                session_id = EXCLUDED.session_id,
                quota = EXCLUDED.quota,
                disabled = EXCLUDED.disabled,
                disabled_reason = EXCLUDED.disabled_reason,
                disabled_at = EXCLUDED.disabled_at,
                last_used = EXCLUDED.last_used,
                updated_at = EXCLUDED.updated_at,
                deleted = FALSE
            "#,
            &[
                &account.id,
                &account.email,
                &account.name,
                &account.token.access_token,
                &account.token.refresh_token,
                &account.token.expires_in,
                &account.token.expiry_timestamp,
                &account.token.token_type,
                &account.token.project_id,
                &account.token.session_id,
                &quota_value,
                &account.disabled,
                &account.disabled_reason,
                &account.disabled_at,
                &account.created_at,
                &account.last_used,
                &account.updated_at,
                &false,
            ],
        ).await?;

        Ok(())
    }

    async fn load_accounts(&self) -> Result<Vec<Account>, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        let rows = client.query(
            r#"
            SELECT id, email, name, access_token, refresh_token, expires_in, expiry_timestamp, token_type,
                   project_id, session_id, quota, disabled, disabled_reason, disabled_at,
                   created_at, last_used, updated_at, COALESCE(version, 0) as version
            FROM antigravity_accounts
            WHERE deleted IS NOT TRUE
            ORDER BY created_at DESC
            "#,
            &[],
        ).await?;

        let mut accounts = Vec::new();
        for row in rows {
            let quota_value: Option<serde_json::Value> = row.get(10);
            let quota = match quota_value {
                Some(value) => serde_json::from_value::<QuotaData>(value).ok(),
                None => None,
            };

            let email: String = row.get(1);
            let account = Account {
                id: row.get(0),
                email: email.clone(),
                name: row.get(2),
                token: TokenData {
                    access_token: row.get(3),
                    refresh_token: row.get(4),
                    expires_in: row.get(5),
                    expiry_timestamp: row.get(6),
                    token_type: row.get(7),
                    email: Some(email),
                    project_id: row.get(8),
                    session_id: row.get(9),
                },
                quota,
                disabled: row.get(11),
                disabled_reason: row.get(12),
                disabled_at: row.get(13),
                created_at: row.get(14),
                last_used: row.get(15),
                updated_at: row.get(16),
                version: row.get(17),
                deleted: false,
            };
            accounts.push(account);
        }

        Ok(accounts)
    }

    async fn update_account(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.save_account(account).await
    }

    async fn delete_account(&self, account_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let version = self.delete_account_with_tombstone(account_id).await?;
        Ok(version > 0)
    }

    async fn get_account(&self, account_id: &str) -> Result<Option<Account>, Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        let rows = client.query(
            r#"
            SELECT id, email, name, access_token, refresh_token, expires_in, expiry_timestamp, token_type,
                   project_id, session_id, quota, disabled, disabled_reason, disabled_at,
                   created_at, last_used, updated_at, COALESCE(version, 0) as version
            FROM antigravity_accounts
            WHERE id = $1 AND deleted IS NOT TRUE
            "#,
            &[&account_id],
        ).await?;

        if let Some(row) = rows.first() {
            let quota_value: Option<serde_json::Value> = row.get(10);
            let quota = match quota_value {
                Some(value) => serde_json::from_value::<QuotaData>(value).ok(),
                None => None,
            };

            let email: String = row.get(1);
            let account = Account {
                id: row.get(0),
                email: email.clone(),
                name: row.get(2),
                token: TokenData {
                    access_token: row.get(3),
                    refresh_token: row.get(4),
                    expires_in: row.get(5),
                    expiry_timestamp: row.get(6),
                    token_type: row.get(7),
                    email: Some(email),
                    project_id: row.get(8),
                    session_id: row.get(9),
                },
                quota,
                disabled: row.get(11),
                disabled_reason: row.get(12),
                disabled_at: row.get(13),
                created_at: row.get(14),
                last_used: row.get(15),
                updated_at: row.get(16),
                version: row.get(17),
                deleted: false,
            };
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    async fn clear_all_accounts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pool = self.get_pool().await?;
        let client = pool.get().await?;

        client.execute(
            "DELETE FROM antigravity_accounts",
            &[],
        ).await?;

        Ok(())
    }

    fn storage_type(&self) -> &'static str {
        "antigravity_postgres"
    }

    async fn is_available(&self) -> bool {
        self.db_manager.is_connected()
    }
}
