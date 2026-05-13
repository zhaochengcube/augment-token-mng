use crate::data::storage::common::{AccountDbMapper, StorageError};
use crate::platforms::antigravity::models::{Account, QuotaData, TokenData};
use tokio_postgres::Row;

/// Antigravity 账号数据库映射器
pub struct AntigravityAccountMapper;

impl AccountDbMapper<Account> for AntigravityAccountMapper {
    fn from_row(row: &Row) -> Result<Account, StorageError> {
        let device_profile_value: Option<serde_json::Value> = row.get(13);
        let device_profile = match device_profile_value {
            Some(value) => serde_json::from_value(value).ok(),
            None => None,
        };

        let quota_value: Option<serde_json::Value> = row.get(14);
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
                oauth_client_key: row.get(9),
                session_id: row.get(10),
                is_gcp_tos: row.get(11),
                id_token: row.get(12),
            },
            device_profile,
            quota,
            tag: row.get(15),
            tag_color: row.get(16),
            disabled: row.get(17),
            disabled_reason: row.get(18),
            disabled_at: row.get(19),
            created_at: row.get(20),
            last_used: row.get(21),
            updated_at: row.get(22),
            version: row.get(23),
            deleted: false,
        })
    }

    fn select_columns() -> &'static str {
        "id, email, name, access_token, refresh_token, expires_in, expiry_timestamp, token_type, \
         project_id, oauth_client_key, session_id, is_gcp_tos, id_token, device_profile, quota, \
         tag, tag_color, disabled, disabled_reason, disabled_at, created_at, last_used, updated_at, version"
    }

    fn insert_sql() -> &'static str {
        r#"
        INSERT INTO antigravity_accounts 
            (id, email, name, access_token, refresh_token, expires_in, expiry_timestamp, token_type,
             project_id, oauth_client_key, session_id, is_gcp_tos, id_token, device_profile, quota,
             tag, tag_color, disabled, disabled_reason, disabled_at, created_at, last_used, updated_at, version, deleted)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
        ON CONFLICT (id) DO UPDATE SET
            email = EXCLUDED.email,
            name = EXCLUDED.name,
            access_token = EXCLUDED.access_token,
            refresh_token = EXCLUDED.refresh_token,
            expires_in = EXCLUDED.expires_in,
            expiry_timestamp = EXCLUDED.expiry_timestamp,
            token_type = EXCLUDED.token_type,
            project_id = EXCLUDED.project_id,
            oauth_client_key = EXCLUDED.oauth_client_key,
            session_id = EXCLUDED.session_id,
            is_gcp_tos = EXCLUDED.is_gcp_tos,
            id_token = EXCLUDED.id_token,
            device_profile = EXCLUDED.device_profile,
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

    fn to_params(
        account: &Account,
        version: i64,
    ) -> Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> {
        let quota_json = account
            .quota
            .as_ref()
            .and_then(|q| serde_json::to_value(q).ok());
        let device_profile_json = account
            .device_profile
            .as_ref()
            .and_then(|profile| serde_json::to_value(profile).ok());

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
            Box::new(account.token.oauth_client_key.clone()),
            Box::new(account.token.session_id.clone()),
            Box::new(account.token.is_gcp_tos),
            Box::new(account.token.id_token.clone()),
            Box::new(device_profile_json),
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
