use crate::platforms::claude::Account;
use tokio_postgres::Client as PgClient;

pub async fn list_accounts(client: &PgClient) -> Result<Vec<Account>, Box<dyn std::error::Error + Send + Sync>> {
    let rows = client.query(
        "SELECT id, service_name, website_url, start_date, duration_days, expiry_date, tag, tag_color, notes,
                base_url, auth_token, default_opus_model, default_sonnet_model, default_haiku_model, use_model,
                created_at, updated_at, deleted, version
         FROM claude_accounts WHERE deleted = FALSE ORDER BY created_at DESC",
        &[],
    ).await?;

    let mut accounts = Vec::new();
    for row in rows.iter() {
        let account = Account {
            id: row.get(0),
            service_name: row.get(1),
            website_url: row.get(2),
            start_date: row.get(3),
            duration_days: row.get(4),
            expiry_date: row.get(5),
            tag: row.get(6),
            tag_color: row.get(7),
            notes: row.get(8),
            base_url: row.get(9),
            auth_token: row.get(10),
            default_opus_model: row.get(11),
            default_sonnet_model: row.get(12),
            default_haiku_model: row.get(13),
            use_model: row.get(14),
            created_at: row.get(15),
            updated_at: row.get(16),
            deleted: row.get(17),
            version: row.get(18),
        };
        accounts.push(account);
    }

    Ok(accounts)
}

pub async fn get_account(client: &PgClient, id: &str) -> Result<Option<Account>, Box<dyn std::error::Error + Send + Sync>> {
    let rows = client.query(
        "SELECT id, service_name, website_url, start_date, duration_days, expiry_date, tag, tag_color, notes,
                base_url, auth_token, default_opus_model, default_sonnet_model, default_haiku_model, use_model,
                created_at, updated_at, deleted, version
         FROM claude_accounts WHERE id = $1 AND deleted = FALSE",
        &[&id],
    ).await?;

    if let Some(row) = rows.first() {
        Ok(Some(Account {
            id: row.get(0),
            service_name: row.get(1),
            website_url: row.get(2),
            start_date: row.get(3),
            duration_days: row.get(4),
            expiry_date: row.get(5),
            tag: row.get(6),
            tag_color: row.get(7),
            notes: row.get(8),
            base_url: row.get(9),
            auth_token: row.get(10),
            default_opus_model: row.get(11),
            default_sonnet_model: row.get(12),
            default_haiku_model: row.get(13),
            use_model: row.get(14),
            created_at: row.get(15),
            updated_at: row.get(16),
            deleted: row.get(17),
            version: row.get(18),
        }))
    } else {
        Ok(None)
    }
}

pub async fn insert_account(client: &PgClient, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client.execute(
        "INSERT INTO claude_accounts
            (id, service_name, website_url, start_date, duration_days, expiry_date, tag, tag_color, notes,
             base_url, auth_token, default_opus_model, default_sonnet_model, default_haiku_model, use_model,
             created_at, updated_at, deleted, version)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)",
        &[
            &account.id,
            &account.service_name,
            &account.website_url,
            &account.start_date,
            &account.duration_days,
            &account.expiry_date,
            &account.tag,
            &account.tag_color,
            &account.notes,
            &account.base_url,
            &account.auth_token,
            &account.default_opus_model,
            &account.default_sonnet_model,
            &account.default_haiku_model,
            &account.use_model,
            &account.created_at,
            &account.updated_at,
            &account.deleted,
            &account.version,
        ],
    ).await?;
    Ok(())
}

pub async fn update_account(client: &PgClient, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client.execute(
        "UPDATE claude_accounts
         SET service_name = $2, website_url = $3, start_date = $4, duration_days = $5, expiry_date = $6,
             tag = $7, tag_color = $8, notes = $9, base_url = $10, auth_token = $11,
             default_opus_model = $12, default_sonnet_model = $13, default_haiku_model = $14, use_model = $15,
             updated_at = $16, version = $17
         WHERE id = $1",
        &[
            &account.id,
            &account.service_name,
            &account.website_url,
            &account.start_date,
            &account.duration_days,
            &account.expiry_date,
            &account.tag,
            &account.tag_color,
            &account.notes,
            &account.base_url,
            &account.auth_token,
            &account.default_opus_model,
            &account.default_sonnet_model,
            &account.default_haiku_model,
            &account.use_model,
            &account.updated_at,
            &account.version,
        ],
    ).await?;
    Ok(())
}

pub async fn delete_account(client: &PgClient, id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    client.execute(
        "UPDATE claude_accounts SET deleted = TRUE, updated_at = $2 WHERE id = $1",
        &[&id, &chrono::Utc::now().timestamp()],
    ).await?;
    Ok(())
}

pub async fn get_pending_sync_operations(client: &PgClient) -> Result<(Vec<Account>, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
    // Get pending upserts (accounts with version > 0 or recently updated)
    let upserts = client.query(
        "SELECT id, service_name, website_url, start_date, duration_days, expiry_date, tag, tag_color, notes,
                base_url, auth_token, default_opus_model, default_sonnet_model, default_haiku_model, use_model,
                created_at, updated_at, deleted, version
         FROM claude_accounts WHERE deleted = FALSE ORDER BY updated_at DESC",
        &[],
    ).await?;

    let mut accounts = Vec::new();
    for row in upserts.iter() {
        let account = Account {
            id: row.get(0),
            service_name: row.get(1),
            website_url: row.get(2),
            start_date: row.get(3),
            duration_days: row.get(4),
            expiry_date: row.get(5),
            tag: row.get(6),
            tag_color: row.get(7),
            notes: row.get(8),
            base_url: row.get(9),
            auth_token: row.get(10),
            default_opus_model: row.get(11),
            default_sonnet_model: row.get(12),
            default_haiku_model: row.get(13),
            use_model: row.get(14),
            created_at: row.get(15),
            updated_at: row.get(16),
            deleted: row.get(17),
            version: row.get(18),
        };
        accounts.push(account);
    }

    // Get pending deletions
    let deletions = client.query(
        "SELECT id FROM claude_accounts WHERE deleted = TRUE ORDER BY updated_at DESC",
        &[],
    ).await?;

    let deletion_ids = deletions.iter().map(|row| row.get(0)).collect();

    Ok((accounts, deletion_ids))
}
