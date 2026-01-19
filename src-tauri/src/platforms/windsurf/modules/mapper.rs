use crate::data::storage::common::{AccountDbMapper, StorageError};
use crate::platforms::windsurf::models::{Account, TokenData, QuotaData};
use tokio_postgres::Row;

/// Windsurf 账号数据库映射器
pub struct WindsurfAccountMapper;

impl AccountDbMapper<Account> for WindsurfAccountMapper {
    fn from_row(row: &Row) -> Result<Account, StorageError> {
        let email: String = row.get(1);

        // 解析 quota JSON
        let quota_value: Option<serde_json::Value> = row.get(14);
        let quota = match quota_value {
            Some(value) => serde_json::from_value::<QuotaData>(value).ok(),
            None => None,
        };

        Ok(Account {
            id: row.get(0),
            email: email.clone(),
            name: row.get(2),
            token: TokenData {
                access_token: row.get(3),
                refresh_token: row.get(4),
                expiry_timestamp: row.get(5),
                email: Some(email),
                user_id: row.get(6),
            },
            api_key: row.get(7),
            api_server_url: row.get(8),
            quota,
            tag: row.get(15),
            tag_color: row.get(16),
            disabled: row.get(9),
            disabled_reason: row.get(10),
            disabled_at: row.get(11),
            created_at: row.get(12),
            last_used: row.get(13),
            updated_at: row.get(17),
            version: row.get(18),
            deleted: false,
        })
    }

    fn select_columns() -> &'static str {
        "id, email, name, access_token, refresh_token, expiry_timestamp, user_id, \
         api_key, api_server_url, disabled, disabled_reason, disabled_at, \
         created_at, last_used, quota, tag, tag_color, updated_at, version"
    }

    fn insert_sql() -> &'static str {
        r#"
        INSERT INTO windsurf_accounts
            (id, email, name, access_token, refresh_token, expiry_timestamp, user_id,
             api_key, api_server_url, quota, tag, tag_color, disabled, disabled_reason, disabled_at, created_at,
             last_used, updated_at, version, deleted)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        ON CONFLICT (id) DO UPDATE SET
            email = EXCLUDED.email,
            name = EXCLUDED.name,
            access_token = EXCLUDED.access_token,
            refresh_token = EXCLUDED.refresh_token,
            expiry_timestamp = EXCLUDED.expiry_timestamp,
            user_id = EXCLUDED.user_id,
            api_key = EXCLUDED.api_key,
            api_server_url = EXCLUDED.api_server_url,
            quota = EXCLUDED.quota,
            tag = EXCLUDED.tag,
            tag_color = EXCLUDED.tag_color,
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
        // 将 quota 序列化为 JSON
        let quota_json: Option<serde_json::Value> = account.quota.as_ref()
            .and_then(|q| serde_json::to_value(q).ok());

        vec![
            Box::new(account.id.clone()),
            Box::new(account.email.clone()),
            Box::new(account.name.clone()),
            Box::new(account.token.access_token.clone()),
            Box::new(account.token.refresh_token.clone()),
            Box::new(account.token.expiry_timestamp),
            Box::new(account.token.user_id.clone()),
            Box::new(account.api_key.clone()),
            Box::new(account.api_server_url.clone()),
            Box::new(quota_json),
            Box::new(account.tag.clone()),
            Box::new(account.tag_color.clone()),
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
