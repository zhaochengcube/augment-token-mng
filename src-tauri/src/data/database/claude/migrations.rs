use tokio_postgres::Client;

pub async fn check_tables_exist(
    client: &Client,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let rows = client
        .query(
            r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'claude_accounts'
        )
        "#,
            &[],
        )
        .await?;

    if let Some(row) = rows.first() {
        let exists: bool = row.get(0);
        Ok(exists)
    } else {
        Ok(false)
    }
}

pub async fn create_tables(
    client: &Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client
        .execute(
            "CREATE SEQUENCE IF NOT EXISTS claude_account_version_seq START 1",
            &[],
        )
        .await?;

    client
        .execute(
            r#"
        CREATE TABLE IF NOT EXISTS claude_accounts (
            id VARCHAR(255) PRIMARY KEY,
            service_name TEXT NOT NULL,
            website_url TEXT,
            start_date BIGINT NOT NULL,
            duration_days BIGINT NOT NULL,
            expiry_date BIGINT NOT NULL,
            tag TEXT,
            tag_color TEXT,
            notes TEXT,
            base_url TEXT NOT NULL,
            auth_token TEXT NOT NULL,
            default_opus_model TEXT NOT NULL,
            default_sonnet_model TEXT NOT NULL,
            default_haiku_model TEXT NOT NULL,
            use_model TEXT NOT NULL,
            created_at BIGINT NOT NULL,
            updated_at BIGINT NOT NULL,
            deleted BOOLEAN NOT NULL DEFAULT FALSE,
            version BIGINT NOT NULL DEFAULT nextval('claude_account_version_seq')
        )
        "#,
            &[],
        )
        .await?;

    client
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_claude_accounts_version ON claude_accounts(version)",
            &[],
        )
        .await?;

    Ok(())
}

pub async fn add_new_fields_if_not_exist(
    client: &Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 添加 use_model 字段（如果不存在）
    client.execute(
        "ALTER TABLE claude_accounts ADD COLUMN IF NOT EXISTS use_model TEXT NOT NULL DEFAULT 'default'",
        &[],
    ).await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn drop_tables(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client
        .execute("DROP TABLE IF EXISTS claude_accounts CASCADE", &[])
        .await?;
    client
        .execute(
            "DROP SEQUENCE IF EXISTS claude_account_version_seq CASCADE",
            &[],
        )
        .await?;
    Ok(())
}
