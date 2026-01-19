use tokio_postgres::Client;

pub async fn check_tables_exist(client: &Client) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let rows = client.query(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'antigravity_accounts'
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
        "CREATE SEQUENCE IF NOT EXISTS antigravity_account_version_seq START 1",
        &[],
    ).await?;

    client.execute(
        r#"
        CREATE TABLE IF NOT EXISTS antigravity_accounts (
            id VARCHAR(255) PRIMARY KEY,
            email TEXT NOT NULL,
            name TEXT,
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            expires_in BIGINT NOT NULL,
            expiry_timestamp BIGINT NOT NULL,
            token_type TEXT,
            project_id TEXT,
            session_id TEXT,
            quota JSONB,
            tag TEXT,
            tag_color TEXT,
            disabled BOOLEAN NOT NULL DEFAULT FALSE,
            disabled_reason TEXT,
            disabled_at BIGINT,
            created_at BIGINT NOT NULL,
            last_used BIGINT NOT NULL,
            updated_at BIGINT NOT NULL,
            deleted BOOLEAN NOT NULL DEFAULT FALSE,
            version BIGINT NOT NULL DEFAULT nextval('antigravity_account_version_seq')
        )
        "#,
        &[],
    ).await?;

    client.execute(
        "CREATE INDEX IF NOT EXISTS idx_antigravity_accounts_version ON antigravity_accounts(version)",
        &[],
    ).await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn drop_tables(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client.execute("DROP TABLE IF EXISTS antigravity_accounts CASCADE", &[]).await?;
    client.execute("DROP SEQUENCE IF EXISTS antigravity_account_version_seq CASCADE", &[]).await?;
    Ok(())
}

pub async fn add_new_fields_if_not_exist(_client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    _client.execute(
        "ALTER TABLE antigravity_accounts ADD COLUMN IF NOT EXISTS disabled BOOLEAN NOT NULL DEFAULT FALSE",
        &[],
    ).await?;
    _client.execute(
        "ALTER TABLE antigravity_accounts ADD COLUMN IF NOT EXISTS disabled_reason TEXT",
        &[],
    ).await?;
    _client.execute(
        "ALTER TABLE antigravity_accounts ADD COLUMN IF NOT EXISTS disabled_at BIGINT",
        &[],
    ).await?;
    _client.execute(
        "ALTER TABLE antigravity_accounts ADD COLUMN IF NOT EXISTS tag TEXT",
        &[],
    ).await?;
    _client.execute(
        "ALTER TABLE antigravity_accounts ADD COLUMN IF NOT EXISTS tag_color TEXT",
        &[],
    ).await?;
    Ok(())
}
