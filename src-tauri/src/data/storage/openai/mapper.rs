use crate::data::storage::common::{AccountDbMapper, StorageError};
use crate::platforms::openai::models::{Account, QuotaData, TokenData};
use tokio_postgres::Row;

/// OpenAI 账号数据库映射器
pub struct OpenAIAccountMapper;

impl AccountDbMapper<Account> for OpenAIAccountMapper {
    fn from_row(row: &Row) -> Result<Account, StorageError> {
        // 检查是否有配额字段（通过检查 codex_usage_updated_at 字段是否存在）
        let quota = if row.len() > 18 {
            // 有配额字段
            let quota_data = QuotaData {
                codex_5h_used_percent: row.try_get(18).ok().flatten(),
                codex_5h_reset_after_seconds: row.try_get(19).ok().flatten(),
                codex_5h_window_minutes: row.try_get(20).ok().flatten(),
                codex_7d_used_percent: row.try_get(21).ok().flatten(),
                codex_7d_reset_after_seconds: row.try_get(22).ok().flatten(),
                codex_7d_window_minutes: row.try_get(23).ok().flatten(),
                codex_primary_over_secondary_percent: row.try_get(24).ok().flatten(),
                codex_usage_updated_at: row.try_get(25).unwrap_or(chrono::Utc::now().timestamp()),
                is_forbidden: false,
            };
            if quota_data.is_valid() {
                Some(quota_data)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Account {
            id: row.get(0),
            email: row.get(1),
            token: TokenData {
                access_token: row.get(2),
                refresh_token: row.get(3),
                id_token: row.get(4),
                expires_in: row.get(5),
                expires_at: row.get(6),
                token_type: row.get(7),
            },
            chatgpt_account_id: row.get(8),
            chatgpt_user_id: row.get(9),
            organization_id: row.get(10),
            quota,
            tag: row.try_get(16).ok().flatten(),
            tag_color: row.try_get(17).ok().flatten(),
            created_at: row.get(11),
            last_used: row.get(12),
            updated_at: row.get(13),
            deleted: row.get(14),
            version: row.get(15),
        })
    }

    fn select_columns() -> &'static str {
        "id, email, access_token, refresh_token, id_token, expires_in, expires_at, token_type, \
         chatgpt_account_id, chatgpt_user_id, organization_id, created_at, last_used, updated_at, deleted, version, \
         tag, tag_color, \
         codex_5h_used_percent, codex_5h_reset_after_seconds, codex_5h_window_minutes, \
         codex_7d_used_percent, codex_7d_reset_after_seconds, codex_7d_window_minutes, \
         codex_primary_over_secondary_percent, codex_usage_updated_at"
    }

    fn insert_sql() -> &'static str {
        r#"
        INSERT INTO openai_accounts
            (id, email, access_token, refresh_token, id_token, expires_in, expires_at, token_type,
             chatgpt_account_id, chatgpt_user_id, organization_id, created_at, last_used, updated_at,
             version, deleted, tag, tag_color, codex_5h_used_percent, codex_5h_reset_after_seconds, codex_5h_window_minutes,
             codex_7d_used_percent, codex_7d_reset_after_seconds, codex_7d_window_minutes,
             codex_primary_over_secondary_percent, codex_usage_updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26)
        ON CONFLICT (id) DO UPDATE SET
            email = EXCLUDED.email,
            access_token = EXCLUDED.access_token,
            refresh_token = EXCLUDED.refresh_token,
            id_token = EXCLUDED.id_token,
            expires_in = EXCLUDED.expires_in,
            expires_at = EXCLUDED.expires_at,
            token_type = EXCLUDED.token_type,
            chatgpt_account_id = EXCLUDED.chatgpt_account_id,
            chatgpt_user_id = EXCLUDED.chatgpt_user_id,
            organization_id = EXCLUDED.organization_id,
            last_used = EXCLUDED.last_used,
            updated_at = EXCLUDED.updated_at,
            version = EXCLUDED.version,
            deleted = EXCLUDED.deleted,
            tag = EXCLUDED.tag,
            tag_color = EXCLUDED.tag_color,
            codex_5h_used_percent = EXCLUDED.codex_5h_used_percent,
            codex_5h_reset_after_seconds = EXCLUDED.codex_5h_reset_after_seconds,
            codex_5h_window_minutes = EXCLUDED.codex_5h_window_minutes,
            codex_7d_used_percent = EXCLUDED.codex_7d_used_percent,
            codex_7d_reset_after_seconds = EXCLUDED.codex_7d_reset_after_seconds,
            codex_7d_window_minutes = EXCLUDED.codex_7d_window_minutes,
            codex_primary_over_secondary_percent = EXCLUDED.codex_primary_over_secondary_percent,
            codex_usage_updated_at = EXCLUDED.codex_usage_updated_at
        "#
    }

    fn to_params(account: &Account, version: i64) -> Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> {
        let (codex_5h_used_percent, codex_5h_reset_after_seconds, codex_5h_window_minutes,
             codex_7d_used_percent, codex_7d_reset_after_seconds, codex_7d_window_minutes,
             codex_primary_over_secondary_percent, codex_usage_updated_at) = if let Some(ref quota) = account.quota {
            (
                quota.codex_5h_used_percent,
                quota.codex_5h_reset_after_seconds,
                quota.codex_5h_window_minutes,
                quota.codex_7d_used_percent,
                quota.codex_7d_reset_after_seconds,
                quota.codex_7d_window_minutes,
                quota.codex_primary_over_secondary_percent,
                quota.codex_usage_updated_at,
            )
        } else {
            (None, None, None, None, None, None, None, chrono::Utc::now().timestamp())
        };

        vec![
            Box::new(account.id.clone()),
            Box::new(account.email.clone()),
            Box::new(account.token.access_token.clone()),
            Box::new(account.token.refresh_token.clone()),
            Box::new(account.token.id_token.clone()),
            Box::new(account.token.expires_in),
            Box::new(account.token.expires_at),
            Box::new(account.token.token_type.clone()),
            Box::new(account.chatgpt_account_id.clone()),
            Box::new(account.chatgpt_user_id.clone()),
            Box::new(account.organization_id.clone()),
            Box::new(account.created_at),
            Box::new(account.last_used),
            Box::new(account.updated_at),
            Box::new(version),
            Box::new(account.deleted),
            Box::new(account.tag.clone()),
            Box::new(account.tag_color.clone()),
            Box::new(codex_5h_used_percent),
            Box::new(codex_5h_reset_after_seconds),
            Box::new(codex_5h_window_minutes),
            Box::new(codex_7d_used_percent),
            Box::new(codex_7d_reset_after_seconds),
            Box::new(codex_7d_window_minutes),
            Box::new(codex_primary_over_secondary_percent),
            Box::new(codex_usage_updated_at),
        ]
    }
}
