use crate::data::storage::common::{AccountDbMapper, StorageError};
use crate::platforms::antigravity::models::{Account, TokenData, QuotaData};
use tokio_postgres::Row;

/// Antigravity 账号数据库映射器
pub struct AntigravityAccountMapper;

impl AccountDbMapper<Account> for AntigravityAccountMapper {
    fn from_row(row: &Row) -> Result<Account, StorageError> {
        let quota_value: Option<serde_json::Value> = row.get(10);
        let quota = match quota_value {
            Some(value) => serde_json::from_value::<QuotaData>(value).ok(),
            None => None,
        };

        let email: String = row.get(1);
        Ok(Account {
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
        })
    }

    fn select_columns() -> &'static str {
        "id, email, name, access_token, refresh_token, expires_in, expiry_timestamp, token_type, \
         project_id, session_id, quota, disabled, disabled_reason, disabled_at, \
         created_at, last_used, updated_at, version"
    }

    fn insert_sql() -> &'static str {
        r#"
        INSERT INTO antigravity_accounts 
            (id, email, name, access_token, refresh_token, expires_in, expiry_timestamp, token_type,
             project_id, session_id, quota, disabled, disabled_reason, disabled_at,
             created_at, last_used, updated_at, version, deleted)
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
            version = EXCLUDED.version,
            deleted = EXCLUDED.deleted
        "#
    }

    fn to_params(account: &Account, version: i64) -> Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> {
        let quota_json = account.quota.as_ref()
            .and_then(|q| serde_json::to_value(q).ok());

        vec![
            Box::new(account.id.clone()),
            Box::new(account.email.clone()),
            Box::new(account.name.clone()),
            Box::new(account.token.access_token.clone()),
            Box::new(account.token.refresh_token.clone()),
            Box::new(account.token.expires_in),
            Box::new(account.token.expiry_timestamp),
            Box::new(account.token.token_type.clone()),
            Box::new(account.token.project_id.clone()),
            Box::new(account.token.session_id.clone()),
            Box::new(quota_json),
            Box::new(account.disabled),
            Box::new(account.disabled_reason.clone()),
            Box::new(account.disabled_at),
            Box::new(account.created_at),
            Box::new(account.last_used),
            Box::new(account.updated_at),
            Box::new(version),
            Box::new(account.deleted),
        ]
    }
}

