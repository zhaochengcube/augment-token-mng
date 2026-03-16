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
            AND table_name = 'bookmarks'
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
            "CREATE SEQUENCE IF NOT EXISTS bookmark_account_version_seq START 1",
            &[],
        )
        .await?;

    client
        .execute(
            r#"
        CREATE TABLE IF NOT EXISTS bookmarks (
            id VARCHAR(255) PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT,
            description TEXT,
            tag TEXT,
            tag_color TEXT,
            created_at BIGINT NOT NULL,
            updated_at BIGINT NOT NULL,
            deleted BOOLEAN NOT NULL DEFAULT FALSE,
            version BIGINT NOT NULL DEFAULT nextval('bookmark_account_version_seq')
        )
        "#,
            &[],
        )
        .await?;

    client
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_bookmarks_version ON bookmarks(version)",
            &[],
        )
        .await?;

    Ok(())
}

pub async fn add_new_fields_if_not_exist(
    client: &Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 添加 tag 字段（如果不存在）
    client
        .execute(
            "ALTER TABLE bookmarks ADD COLUMN IF NOT EXISTS tag TEXT",
            &[],
        )
        .await
        .ok();

    // 添加 tag_color 字段（如果不存在）
    client
        .execute(
            "ALTER TABLE bookmarks ADD COLUMN IF NOT EXISTS tag_color TEXT",
            &[],
        )
        .await
        .ok();

    Ok(())
}

#[allow(dead_code)]
pub async fn drop_tables(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client
        .execute("DROP TABLE IF EXISTS bookmarks CASCADE", &[])
        .await?;
    client
        .execute(
            "DROP SEQUENCE IF EXISTS bookmark_account_version_seq CASCADE",
            &[],
        )
        .await?;
    Ok(())
}
