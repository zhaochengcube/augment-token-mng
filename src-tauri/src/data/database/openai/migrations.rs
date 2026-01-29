use tokio_postgres::Client;

pub async fn check_tables_exist(client: &Client) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let rows = client.query(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'openai_accounts'
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
        "CREATE SEQUENCE IF NOT EXISTS openai_account_version_seq START 1",
        &[],
    ).await?;

    client.execute(
        r#"
        CREATE TABLE IF NOT EXISTS openai_accounts (
            id VARCHAR(255) PRIMARY KEY,
            email TEXT NOT NULL,
            access_token TEXT NOT NULL,
            refresh_token TEXT,
            id_token TEXT,
            expires_in BIGINT NOT NULL,
            expires_at BIGINT NOT NULL,
            token_type TEXT,
            chatgpt_account_id TEXT,
            chatgpt_user_id TEXT,
            organization_id TEXT,
            created_at BIGINT NOT NULL,
            last_used BIGINT NOT NULL,
            updated_at BIGINT NOT NULL,
            deleted BOOLEAN NOT NULL DEFAULT FALSE,
            version BIGINT NOT NULL DEFAULT nextval('openai_account_version_seq')
        )
        "#,
        &[],
    ).await?;

    client.execute(
        "CREATE INDEX IF NOT EXISTS idx_openai_accounts_version ON openai_accounts(version)",
        &[],
    ).await?;

    Ok(())
}

pub async fn add_new_fields_if_not_exist(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 添加配额字段
    let quota_columns = vec![
        "codex_5h_used_percent",
        "codex_5h_reset_after_seconds",
        "codex_5h_window_minutes",
        "codex_7d_used_percent",
        "codex_7d_reset_after_seconds",
        "codex_7d_window_minutes",
        "codex_primary_over_secondary_percent",
        "codex_usage_updated_at",
    ];

    // 添加标签字段
    let tag_columns = vec![
        ("tag", "TEXT"),
        ("tag_color", "TEXT"),
    ];

    for column in &quota_columns {
        let check_column = client.query_one(
            &format!(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_name = 'openai_accounts'
                    AND column_name = '{}'
                )",
                column
            ),
            &[],
        ).await?;

        let exists: bool = check_column.get(0);
        if !exists {
            match *column {
                "codex_5h_used_percent" | "codex_7d_used_percent" | "codex_primary_over_secondary_percent" => {
                    client.execute(
                        &format!("ALTER TABLE openai_accounts ADD COLUMN {} DOUBLE PRECISION", column),
                        &[],
                    ).await?;
                }
                "codex_usage_updated_at" => {
                    client.execute(
                        "ALTER TABLE openai_accounts ADD COLUMN codex_usage_updated_at BIGINT",
                        &[],
                    ).await?;
                }
                _ => {
                    client.execute(
                        &format!("ALTER TABLE openai_accounts ADD COLUMN {} BIGINT", column),
                        &[],
                    ).await?;
                }
            }
            println!("Added column {} to openai_accounts", column);
        }
    }

    // 添加标签字段
    for (column, data_type) in &tag_columns {
        let check_column = client.query_one(
            &format!(
                "SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_name = 'openai_accounts'
                    AND column_name = '{}'
                )",
                column
            ),
            &[],
        ).await?;

        let exists: bool = check_column.get(0);
        if !exists {
            client.execute(
                &format!("ALTER TABLE openai_accounts ADD COLUMN {} {}", column, data_type),
                &[],
            ).await?;
            println!("Added column {} to openai_accounts", column);
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn drop_tables(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client.execute("DROP TABLE IF EXISTS openai_accounts CASCADE", &[]).await?;
    client.execute("DROP SEQUENCE IF EXISTS openai_account_version_seq CASCADE", &[]).await?;
    Ok(())
}

