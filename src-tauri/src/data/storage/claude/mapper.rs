use crate::data::storage::common::{AccountDbMapper, StorageError};
use crate::platforms::claude::Account;
use tokio_postgres::Row;

/// Claude 账户数据库映射器
pub struct ClaudeAccountMapper;

impl AccountDbMapper<Account> for ClaudeAccountMapper {
    fn from_row(row: &Row) -> Result<Account, StorageError> {
        Ok(Account {
            id: row.get(0),
            service_name: row.get(1),
            website_url: row.try_get(2).ok().flatten(),
            start_date: row.get(3),
            duration_days: row.get(4),
            expiry_date: row.get(5),
            tag: row.try_get(6).ok().flatten(),
            tag_color: row.try_get(7).ok().flatten(),
            notes: row.try_get(8).ok().flatten(),
            base_url: row.get(9),
            auth_token: row.get(10),
            default_opus_model: row.get(11),
            default_sonnet_model: row.get(12),
            default_haiku_model: row.get(13),
            use_model: row.get(14),
            created_at: row.get(15),
            updated_at: row.get(16),
            deleted: row.get(17),
            version: row.get(18),
        })
    }

    fn select_columns() -> &'static str {
        "id, service_name, website_url, start_date, duration_days, expiry_date, \
         tag, tag_color, notes, base_url, auth_token, \
         default_opus_model, default_sonnet_model, default_haiku_model, use_model, \
         created_at, updated_at, deleted, version"
    }

    fn insert_sql() -> &'static str {
        r#"
        INSERT INTO claude_accounts
            (id, service_name, website_url, start_date, duration_days, expiry_date,
             tag, tag_color, notes, base_url, auth_token,
             default_opus_model, default_sonnet_model, default_haiku_model, use_model,
             created_at, updated_at, deleted, version)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
        ON CONFLICT (id) DO UPDATE SET
            service_name = EXCLUDED.service_name,
            website_url = EXCLUDED.website_url,
            start_date = EXCLUDED.start_date,
            duration_days = EXCLUDED.duration_days,
            expiry_date = EXCLUDED.expiry_date,
            tag = EXCLUDED.tag,
            tag_color = EXCLUDED.tag_color,
            notes = EXCLUDED.notes,
            base_url = EXCLUDED.base_url,
            auth_token = EXCLUDED.auth_token,
            default_opus_model = EXCLUDED.default_opus_model,
            default_sonnet_model = EXCLUDED.default_sonnet_model,
            default_haiku_model = EXCLUDED.default_haiku_model,
            use_model = EXCLUDED.use_model,
            updated_at = EXCLUDED.updated_at,
            deleted = EXCLUDED.deleted,
            version = EXCLUDED.version
        "#
    }

    fn to_params(account: &Account, version: i64) -> Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> {
        vec![
            Box::new(account.id.clone()),
            Box::new(account.service_name.clone()),
            Box::new(account.website_url.clone()),
            Box::new(account.start_date),
            Box::new(account.duration_days),
            Box::new(account.expiry_date),
            Box::new(account.tag.clone()),
            Box::new(account.tag_color.clone()),
            Box::new(account.notes.clone()),
            Box::new(account.base_url.clone()),
            Box::new(account.auth_token.clone()),
            Box::new(account.default_opus_model.clone()),
            Box::new(account.default_sonnet_model.clone()),
            Box::new(account.default_haiku_model.clone()),
            Box::new(account.use_model.clone()),
            Box::new(account.created_at),
            Box::new(account.updated_at),
            Box::new(account.deleted),
            Box::new(version),
        ]
    }
}
