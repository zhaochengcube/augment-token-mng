use tokio_postgres::Client;

pub async fn check_tables_exist(client: &Client) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let rows = client.query(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'windsurf_accounts'
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
        "CREATE SEQUENCE IF NOT EXISTS windsurf_account_version_seq START 1",
        &[],
    ).await?;

    client.execute(
        r#"
        CREATE TABLE IF NOT EXISTS windsurf_accounts (
            id VARCHAR(255) PRIMARY KEY,
            email TEXT NOT NULL,
            name TEXT,
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            expiry_timestamp BIGINT NOT NULL,
            user_id TEXT,
            api_key TEXT,
            api_server_url TEXT,
            disabled BOOLEAN NOT NULL DEFAULT FALSE,
            disabled_reason TEXT,
            disabled_at BIGINT,
            created_at BIGINT NOT NULL,
            last_used BIGINT NOT NULL,
            quota JSONB,
            tag TEXT,
            tag_color TEXT,
            updated_at BIGINT NOT NULL,
            deleted BOOLEAN NOT NULL DEFAULT FALSE,
            version BIGINT NOT NULL DEFAULT nextval('windsurf_account_version_seq')
        )
        "#,
        &[],
    ).await?;

    client.execute(
        "CREATE INDEX IF NOT EXISTS idx_windsurf_accounts_version ON windsurf_accounts(version)",
        &[],
    ).await?;

    Ok(())
}

pub async fn add_new_fields_if_not_exist(_client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Ok(())
}

#[allow(dead_code)]
pub async fn drop_tables(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client.execute("DROP TABLE IF EXISTS windsurf_accounts CASCADE", &[]).await?;
    client.execute("DROP SEQUENCE IF EXISTS windsurf_account_version_seq CASCADE", &[]).await?;
    Ok(())
}
