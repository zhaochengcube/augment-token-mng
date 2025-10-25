use tokio_postgres::Client;

pub async fn check_tables_exist(client: &Client) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    // 检查tokens表是否存在
    let rows = client.query(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'tokens'
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
    // 创建tokens表
    client.execute(
        r#"
        CREATE TABLE IF NOT EXISTS tokens (
            id VARCHAR(255) PRIMARY KEY,
            tenant_url TEXT NOT NULL,
            access_token TEXT NOT NULL,
            created_at TIMESTAMP WITH TIME ZONE NOT NULL,
            updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
            portal_url TEXT,
            email_note TEXT,
            ban_status JSONB,
            portal_info JSONB,
            auth_session TEXT,
            suspensions JSONB
        )
        "#,
        &[],
    ).await?;

    // 创建索引
    client.execute(
        "CREATE INDEX IF NOT EXISTS idx_tokens_created_at ON tokens(created_at)",
        &[],
    ).await?;

    client.execute(
        "CREATE INDEX IF NOT EXISTS idx_tokens_updated_at ON tokens(updated_at)",
        &[],
    ).await?;

    // 创建sync_status表
    client.execute(
        r#"
        CREATE TABLE IF NOT EXISTS sync_status (
            id SERIAL PRIMARY KEY,
            last_sync_at TIMESTAMP WITH TIME ZONE,
            sync_direction VARCHAR(50),
            status VARCHAR(50),
            error_message TEXT,
            tokens_synced INTEGER DEFAULT 0,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
        &[],
    ).await?;

    // 创建updated_at触发器函数
    client.execute(
        r#"
        CREATE OR REPLACE FUNCTION update_updated_at_column()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at = NOW();
            RETURN NEW;
        END;
        $$ language 'plpgsql'
        "#,
        &[],
    ).await?;

    // 删除现有触发器（如果存在）
    client.execute(
        "DROP TRIGGER IF EXISTS update_tokens_updated_at ON tokens",
        &[],
    ).await?;

    // 为tokens表创建触发器
    client.execute(
        r#"
        CREATE TRIGGER update_tokens_updated_at
            BEFORE UPDATE ON tokens
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column()
        "#,
        &[],
    ).await?;

    Ok(())
}

pub async fn drop_tables(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client.execute("DROP TABLE IF EXISTS sync_status CASCADE", &[]).await?;
    client.execute("DROP TABLE IF EXISTS tokens CASCADE", &[]).await?;
    client.execute("DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE", &[]).await?;
    Ok(())
}

// 添加新字段的迁移函数
pub async fn add_new_fields_if_not_exist(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 检查 auth_session 字段是否存在
    let auth_session_exists = client.query(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.columns
            WHERE table_schema = 'public'
            AND table_name = 'tokens'
            AND column_name = 'auth_session'
        )
        "#,
        &[],
    ).await?;

    if let Some(row) = auth_session_exists.first() {
        let exists: bool = row.get(0);
        if !exists {
            client.execute(
                "ALTER TABLE tokens ADD COLUMN auth_session TEXT",
                &[],
            ).await?;
            println!("Added auth_session column to tokens table");
        }
    }

    // 检查 suspensions 字段是否存在
    let suspensions_exists = client.query(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.columns
            WHERE table_schema = 'public'
            AND table_name = 'tokens'
            AND column_name = 'suspensions'
        )
        "#,
        &[],
    ).await?;

    if let Some(row) = suspensions_exists.first() {
        let exists: bool = row.get(0);
        if !exists {
            client.execute(
                "ALTER TABLE tokens ADD COLUMN suspensions JSONB",
                &[],
            ).await?;
            println!("Added suspensions column to tokens table");
        }
    }

    // 检查 balance_color_mode 字段是否存在
    let balance_color_mode_exists = client.query(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.columns
            WHERE table_schema = 'public'
            AND table_name = 'tokens'
            AND column_name = 'balance_color_mode'
        )
        "#,
        &[],
    ).await?;

    if let Some(row) = balance_color_mode_exists.first() {
        let exists: bool = row.get(0);
        if !exists {
            client.execute(
                "ALTER TABLE tokens ADD COLUMN balance_color_mode TEXT",
                &[],
            ).await?;
            println!("Added balance_color_mode column to tokens table");
        }
    }

    // 检查 skip_check 字段是否存在
    let skip_check_exists = client.query(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.columns
            WHERE table_schema = 'public'
            AND table_name = 'tokens'
            AND column_name = 'skip_check'
        )
        "#,
        &[],
    ).await?;

    if let Some(row) = skip_check_exists.first() {
        let exists: bool = row.get(0);
        if !exists {
            client.execute(
                "ALTER TABLE tokens ADD COLUMN skip_check BOOLEAN",
                &[],
            ).await?;
            println!("Added skip_check column to tokens table");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_postgres::{NoTls, Config};

    async fn get_test_client() -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        // 这里需要一个测试数据库连接
        // 在实际测试中，你需要设置一个测试数据库
        let mut config = Config::new();
        config.host("localhost");
        config.port(5432);
        config.dbname("test_augment_tokens");
        config.user("postgres");
        config.password("password");

        let (client, connection) = config.connect(NoTls).await?;
        
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(client)
    }

    #[tokio::test]
    #[ignore] // 忽略这个测试，因为它需要真实的数据库连接
    async fn test_create_tables() {
        let client = get_test_client().await.unwrap();
        let result = create_tables(&client).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // 忽略这个测试，因为它需要真实的数据库连接
    async fn test_drop_tables() {
        let client = get_test_client().await.unwrap();
        let _ = create_tables(&client).await;
        let result = drop_tables(&client).await;
        assert!(result.is_ok());
    }
}
