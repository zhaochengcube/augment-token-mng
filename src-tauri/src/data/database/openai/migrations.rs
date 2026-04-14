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
            AND table_name = 'openai_accounts'
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
            "CREATE SEQUENCE IF NOT EXISTS openai_account_version_seq START 1",
            &[],
        )
        .await?;

    client
        .execute(
            r#"
        CREATE TABLE IF NOT EXISTS openai_accounts (
            id VARCHAR(255) PRIMARY KEY,
            email TEXT NOT NULL,
            reverse_proxy_enabled BOOLEAN NOT NULL DEFAULT TRUE,
            access_token TEXT NOT NULL,
            refresh_token TEXT,
            id_token TEXT,
            expires_in BIGINT NOT NULL,
            expires_at BIGINT NOT NULL,
            token_type TEXT,
            chatgpt_account_id TEXT,
            chatgpt_user_id TEXT,
            organization_id TEXT,
            openai_auth_json TEXT,
            created_at BIGINT NOT NULL,
            last_used BIGINT NOT NULL,
            updated_at BIGINT NOT NULL,
            deleted BOOLEAN NOT NULL DEFAULT FALSE,
            version BIGINT NOT NULL DEFAULT nextval('openai_account_version_seq'),
            is_forbidden BOOLEAN NOT NULL DEFAULT FALSE
        )
        "#,
            &[],
        )
        .await?;

    client
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_openai_accounts_version ON openai_accounts(version)",
            &[],
        )
        .await?;

    Ok(())
}

pub async fn add_new_fields_if_not_exist(
    client: &Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    let tag_columns = vec![("tag", "TEXT"), ("tag_color", "TEXT")];

    // 添加账号类型和 API 配置字段
    let api_columns = vec![
        ("account_type", "TEXT"),
        ("model_provider", "TEXT"),
        ("model", "TEXT"),
        ("model_reasoning_effort", "TEXT"),
        ("wire_api", "TEXT"),
        ("base_url", "TEXT"),
        ("api_key", "TEXT"),
        ("openai_auth_json", "TEXT"),
    ];

    for column in &quota_columns {
        let check_column = client
            .query_one(
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
            )
            .await?;

        let exists: bool = check_column.get(0);
        if !exists {
            match *column {
                "codex_5h_used_percent"
                | "codex_7d_used_percent"
                | "codex_primary_over_secondary_percent" => {
                    client
                        .execute(
                            &format!(
                                "ALTER TABLE openai_accounts ADD COLUMN {} DOUBLE PRECISION",
                                column
                            ),
                            &[],
                        )
                        .await?;
                }
                "codex_usage_updated_at" => {
                    client
                        .execute(
                            "ALTER TABLE openai_accounts ADD COLUMN codex_usage_updated_at BIGINT",
                            &[],
                        )
                        .await?;
                }
                _ => {
                    client
                        .execute(
                            &format!("ALTER TABLE openai_accounts ADD COLUMN {} BIGINT", column),
                            &[],
                        )
                        .await?;
                }
            }
            println!("Added column {} to openai_accounts", column);
        }
    }

    // 添加标签字段
    for (column, data_type) in &tag_columns {
        let check_column = client
            .query_one(
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
            )
            .await?;

        let exists: bool = check_column.get(0);
        if !exists {
            client
                .execute(
                    &format!(
                        "ALTER TABLE openai_accounts ADD COLUMN {} {}",
                        column, data_type
                    ),
                    &[],
                )
                .await?;
            println!("Added column {} to openai_accounts", column);
        }
    }

    // 添加 is_forbidden 字段
    let check_forbidden = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_name = 'openai_accounts'
                AND column_name = 'is_forbidden'
            )",
            &[],
        )
        .await?;

    let forbidden_exists: bool = check_forbidden.get(0);
    if !forbidden_exists {
        client
            .execute(
                "ALTER TABLE openai_accounts ADD COLUMN is_forbidden BOOLEAN NOT NULL DEFAULT FALSE",
                &[],
            )
            .await?;
        println!("Added column is_forbidden to openai_accounts");
    }

    // 添加 rt_invalid 字段
    let check_rt_invalid = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_name = 'openai_accounts'
                AND column_name = 'rt_invalid'
            )",
            &[],
        )
        .await?;

    let rt_invalid_exists: bool = check_rt_invalid.get(0);
    if !rt_invalid_exists {
        client
            .execute(
                "ALTER TABLE openai_accounts ADD COLUMN rt_invalid BOOLEAN NOT NULL DEFAULT FALSE",
                &[],
            )
            .await?;
        println!("Added column rt_invalid to openai_accounts");
    }

    // 添加 rt_invalid_reason 字段
    let check_rt_invalid_reason = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_name = 'openai_accounts'
                AND column_name = 'rt_invalid_reason'
            )",
            &[],
        )
        .await?;

    let rt_invalid_reason_exists: bool = check_rt_invalid_reason.get(0);
    if !rt_invalid_reason_exists {
        client
            .execute(
                "ALTER TABLE openai_accounts ADD COLUMN rt_invalid_reason TEXT",
                &[],
            )
            .await?;
        println!("Added column rt_invalid_reason to openai_accounts");
    }

    let check_reverse_proxy_enabled = client
        .query_one(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_name = 'openai_accounts'
                AND column_name = 'reverse_proxy_enabled'
            )",
            &[],
        )
        .await?;

    let reverse_proxy_enabled_exists: bool = check_reverse_proxy_enabled.get(0);
    if !reverse_proxy_enabled_exists {
        client
            .execute(
                "ALTER TABLE openai_accounts ADD COLUMN reverse_proxy_enabled BOOLEAN NOT NULL DEFAULT TRUE",
                &[],
            )
            .await?;
        println!("Added column reverse_proxy_enabled to openai_accounts");
    }

    // 添加 API 账号字段
    for (column, data_type) in &api_columns {
        let check_column = client
            .query_one(
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
            )
            .await?;

        let exists: bool = check_column.get(0);
        if !exists {
            client
                .execute(
                    &format!(
                        "ALTER TABLE openai_accounts ADD COLUMN {} {}",
                        column, data_type
                    ),
                    &[],
                )
                .await?;
            println!("Added column {} to openai_accounts", column);
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn drop_tables(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client
        .execute("DROP TABLE IF EXISTS openai_accounts CASCADE", &[])
        .await?;
    client
        .execute(
            "DROP SEQUENCE IF EXISTS openai_account_version_seq CASCADE",
            &[],
        )
        .await?;
    Ok(())
}
