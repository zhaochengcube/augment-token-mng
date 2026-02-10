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
            AND table_name = 'subscriptions'
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
            "CREATE SEQUENCE IF NOT EXISTS subscription_account_version_seq START 1",
            &[],
        )
        .await?;

    client
        .execute(
            r#"
        CREATE TABLE IF NOT EXISTS subscriptions (
            id VARCHAR(255) PRIMARY KEY,
            website TEXT NOT NULL,
            website_url TEXT,
            start_date TEXT,
            duration_months INTEGER,
            expiry_date TEXT,
            cost DOUBLE PRECISION,
            tag TEXT,
            tag_color TEXT,
            notes TEXT,
            created_at BIGINT NOT NULL,
            updated_at BIGINT NOT NULL,
            deleted BOOLEAN NOT NULL DEFAULT FALSE,
            version BIGINT NOT NULL DEFAULT nextval('subscription_account_version_seq')
        )
        "#,
            &[],
        )
        .await?;

    client
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_subscriptions_version ON subscriptions(version)",
            &[],
        )
        .await?;

    client.execute(
        "CREATE INDEX IF NOT EXISTS idx_subscriptions_expiry_date ON subscriptions(expiry_date)",
        &[],
    ).await?;

    Ok(())
}

pub async fn add_new_fields_if_not_exist(
    client: &Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 添加 website_url 字段（如果不存在）
    client
        .execute(
            "ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS website_url TEXT",
            &[],
        )
        .await
        .ok();

    // 添加 start_date 字段（如果不存在）
    client
        .execute(
            "ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS start_date TEXT",
            &[],
        )
        .await
        .ok();

    // 添加 duration_months 字段（如果不存在）
    client
        .execute(
            "ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS duration_months INTEGER",
            &[],
        )
        .await
        .ok();

    // 删除旧的 account_email 字段（如果存在）
    client
        .execute(
            "ALTER TABLE subscriptions DROP COLUMN IF EXISTS account_email",
            &[],
        )
        .await
        .ok();

    Ok(())
}

#[allow(dead_code)]
pub async fn drop_tables(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client
        .execute("DROP TABLE IF EXISTS subscriptions CASCADE", &[])
        .await?;
    client
        .execute(
            "DROP SEQUENCE IF EXISTS subscription_account_version_seq CASCADE",
            &[],
        )
        .await?;
    Ok(())
}
