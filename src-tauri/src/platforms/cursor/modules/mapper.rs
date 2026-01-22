use crate::data::storage::common::{AccountDbMapper, StorageError};
use crate::platforms::cursor::models::{Account, TokenData};
use tokio_postgres::Row;

/// Cursor 账号数据库映射器
pub struct CursorAccountMapper;

impl AccountDbMapper<Account> for CursorAccountMapper {
    fn from_row(row: &Row) -> Result<Account, StorageError> {
        Ok(Account {
            id: row.get(0),
            email: row.get(1),
            name: row.get(2),
            token: TokenData {
                access_token: row.get(3),
                refresh_token: row.get(4),
                expiry_timestamp: row.get(5),
                user_id: row.get(6),
                workos_cursor_session_token: row.get(7),
                session_expiry_timestamp: row.get(8),
            },
            tag: row.get(9),
            tag_color: row.get(10),
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
        "id, email, name, access_token, refresh_token, expiry_timestamp, user_id, \
         workos_cursor_session_token, session_expiry_timestamp, \
         tag, tag_color, disabled, disabled_reason, disabled_at, \
         created_at, last_used, updated_at, version"
    }

    fn insert_sql() -> &'static str {
        r#"
        INSERT INTO cursor_accounts
            (id, email, name, access_token, refresh_token, expiry_timestamp, user_id,
             workos_cursor_session_token, session_expiry_timestamp,
             tag, tag_color, disabled, disabled_reason, disabled_at, created_at,
             last_used, updated_at, version, deleted)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
        ON CONFLICT (id) DO UPDATE SET
            email = EXCLUDED.email,
            name = EXCLUDED.name,
            access_token = EXCLUDED.access_token,
            refresh_token = EXCLUDED.refresh_token,
            expiry_timestamp = EXCLUDED.expiry_timestamp,
            user_id = EXCLUDED.user_id,
            workos_cursor_session_token = EXCLUDED.workos_cursor_session_token,
            session_expiry_timestamp = EXCLUDED.session_expiry_timestamp,
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
        vec![
            Box::new(account.id.clone()),
            Box::new(account.email.clone()),
            Box::new(account.name.clone()),
            Box::new(account.token.access_token.clone()),
            Box::new(account.token.refresh_token.clone()),
            Box::new(account.token.expiry_timestamp),
            Box::new(account.token.user_id.clone()),
            Box::new(account.token.workos_cursor_session_token.clone()),
            Box::new(account.token.session_expiry_timestamp),
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

