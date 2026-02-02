use tokio_postgres::Client;

pub async fn check_tables_exist(client: &Client) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let rows = client.query(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'cursor_accounts'
        )
        "#,
        &[],
    ).await?;

    if let Some(row) = rows.first() {
        let exists: bool = row.get(0);
        Ok(exists)
    } else {
        Ok(false)
    }
}

pub async fn create_tables(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client.execute(
        "CREATE SEQUENCE IF NOT EXISTS cursor_account_version_seq START 1",
        &[],
    ).await?;

    client.execute(
        r#"
        CREATE TABLE IF NOT EXISTS cursor_accounts (
            id VARCHAR(255) PRIMARY KEY,
            email TEXT NOT NULL,
            name TEXT,
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            expiry_timestamp BIGINT NOT NULL,
            user_id TEXT,
            workos_cursor_session_token TEXT,
            session_expiry_timestamp BIGINT,
            disabled BOOLEAN NOT NULL DEFAULT FALSE,
            disabled_reason TEXT,
            disabled_at BIGINT,
            tag TEXT,
            tag_color TEXT,
            created_at BIGINT NOT NULL,
            last_used BIGINT NOT NULL,
            updated_at BIGINT NOT NULL,
            deleted BOOLEAN NOT NULL DEFAULT FALSE,
            version BIGINT NOT NULL DEFAULT nextval('cursor_account_version_seq')
        )
        "#,
        &[],
    ).await?;

    client.execute(
        "CREATE INDEX IF NOT EXISTS idx_cursor_accounts_version ON cursor_accounts(version)",
        &[],
    ).await?;

    Ok(())
}

pub async fn add_new_fields_if_not_exist(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 添加 workos_cursor_session_token 字段（如果不存在）
    client.execute(
        "ALTER TABLE cursor_accounts ADD COLUMN IF NOT EXISTS workos_cursor_session_token TEXT",
        &[],
    ).await?;

    // 添加 session_expiry_timestamp 字段（如果不存在）
    client.execute(
        "ALTER TABLE cursor_accounts ADD COLUMN IF NOT EXISTS session_expiry_timestamp BIGINT",
        &[],
    ).await?;

    // 添加 machine_info 字段（如果不存在）- 存储为 JSONB
    client.execute(
        "ALTER TABLE cursor_accounts ADD COLUMN IF NOT EXISTS machine_info JSONB",
        &[],
    ).await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn drop_tables(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client.execute("DROP TABLE IF EXISTS cursor_accounts CASCADE", &[]).await?;
    client.execute("DROP SEQUENCE IF EXISTS cursor_account_version_seq CASCADE", &[]).await?;
    Ok(())
}
