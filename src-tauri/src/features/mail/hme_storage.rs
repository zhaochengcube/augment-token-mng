use rusqlite::{Connection, params};
use std::collections::HashSet;
use std::path::PathBuf;

use super::hme::HmeEmailItem;

pub struct HmeStorage {
    db_path: PathBuf,
}

impl HmeStorage {
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create HME data directory: {}", e))?;

        let db_path = data_dir.join("hme_emails.db");
        let storage = Self { db_path };

        let mut conn = storage
            .get_connection()
            .map_err(|e| format!("Failed to open HME database: {}", e))?;
        storage
            .init_schema(&mut conn)
            .map_err(|e| format!("Failed to initialize HME schema: {}", e))?;

        Ok(storage)
    }

    fn get_connection(&self) -> Result<Connection, String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open HME database connection: {}", e))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
            .map_err(|e| format!("Failed to set PRAGMA: {}", e))?;
        Ok(conn)
    }

    fn init_schema(&self, conn: &mut Connection) -> Result<(), String> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS hme_emails (
                anonymous_id TEXT PRIMARY KEY,
                label        TEXT NOT NULL DEFAULT '',
                hme          TEXT NOT NULL,
                is_active    INTEGER NOT NULL DEFAULT 1,
                created_at   INTEGER NOT NULL DEFAULT 0,
                updated_at   INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_hme_active ON hme_emails(is_active);
            CREATE INDEX IF NOT EXISTS idx_hme_created ON hme_emails(created_at DESC);",
        )
        .map_err(|e| format!("Failed to create HME table: {}", e))?;

        // Migration: add tag columns if not exist
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(hme_emails)")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| row.get::<_, String>(1))
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        if !columns.iter().any(|c| c == "tag") {
            conn.execute_batch("ALTER TABLE hme_emails ADD COLUMN tag TEXT DEFAULT NULL")
                .map_err(|e| format!("Failed to add tag column: {}", e))?;
        }
        if !columns.iter().any(|c| c == "tag_color") {
            conn.execute_batch("ALTER TABLE hme_emails ADD COLUMN tag_color TEXT DEFAULT NULL")
                .map_err(|e| format!("Failed to add tag_color column: {}", e))?;
        }

        Ok(())
    }

    pub fn load_all(&self, is_active: Option<bool>) -> Result<Vec<HmeEmailItem>, String> {
        let conn = self.get_connection()?;

        let (sql, active_val) = match is_active {
            Some(active) => (
                "SELECT anonymous_id, label, hme, is_active, created_at, tag, tag_color FROM hme_emails WHERE is_active = ?1 ORDER BY created_at DESC",
                Some(active as i32),
            ),
            None => (
                "SELECT anonymous_id, label, hme, is_active, created_at, tag, tag_color FROM hme_emails ORDER BY created_at DESC",
                None,
            ),
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = if let Some(val) = active_val {
            stmt.query_map(params![val], row_to_item)
        } else {
            stmt.query_map([], row_to_item)
        }
        .map_err(|e| format!("Failed to query HME emails: {}", e))?;

        let mut items = Vec::new();
        for row in rows {
            if let Ok(item) = row {
                items.push(item);
            }
        }
        Ok(items)
    }

    pub fn upsert_batch(&self, items: &[HmeEmailItem]) -> Result<(), String> {
        if items.is_empty() {
            return Ok(());
        }
        let mut conn = self.get_connection()?;
        let tx = conn
            .transaction()
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        let now_ms = chrono::Utc::now().timestamp_millis();
        for item in items {
            tx.execute(
                "INSERT INTO hme_emails (anonymous_id, label, hme, is_active, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(anonymous_id) DO UPDATE SET
                   label = excluded.label,
                   hme = excluded.hme,
                   is_active = excluded.is_active,
                   created_at = excluded.created_at,
                   updated_at = excluded.updated_at",
                params![
                    item.anonymous_id,
                    item.label,
                    item.hme,
                    item.is_active as i32,
                    item.create_timestamp,
                    now_ms,
                ],
            )
            .map_err(|e| format!("Failed to upsert HME email: {}", e))?;
        }

        tx.commit()
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;
        Ok(())
    }

    pub fn delete_batch(&self, anonymous_ids: &[String]) -> Result<(), String> {
        if anonymous_ids.is_empty() {
            return Ok(());
        }
        let mut conn = self.get_connection()?;
        let tx = conn
            .transaction()
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        for id in anonymous_ids {
            tx.execute(
                "DELETE FROM hme_emails WHERE anonymous_id = ?1",
                params![id],
            )
            .map_err(|e| format!("Failed to delete HME email: {}", e))?;
        }

        tx.commit()
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;
        Ok(())
    }

    pub fn set_inactive(&self, anonymous_ids: &[String]) -> Result<(), String> {
        if anonymous_ids.is_empty() {
            return Ok(());
        }
        let mut conn = self.get_connection()?;
        let now_ms = chrono::Utc::now().timestamp_millis();
        let tx = conn
            .transaction()
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        for id in anonymous_ids {
            tx.execute(
                "UPDATE hme_emails SET is_active = 0, updated_at = ?1 WHERE anonymous_id = ?2",
                params![now_ms, id],
            )
            .map_err(|e| format!("Failed to deactivate HME email: {}", e))?;
        }

        tx.commit()
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;
        Ok(())
    }

    pub fn update_tag(
        &self,
        anonymous_id: &str,
        tag: Option<&str>,
        tag_color: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.get_connection()?;
        let now_ms = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "UPDATE hme_emails SET tag = ?1, tag_color = ?2, updated_at = ?3 WHERE anonymous_id = ?4",
            params![tag, tag_color, now_ms, anonymous_id],
        )
        .map_err(|e| format!("Failed to update tag: {}", e))?;
        Ok(())
    }

    /// Full sync: upsert all API items, delete local rows not in API response
    pub fn sync_from_api(&self, api_items: &[HmeEmailItem]) -> Result<(), String> {
        let conn = self.get_connection()?;

        let mut existing_ids: HashSet<String> = HashSet::new();
        {
            let mut stmt = conn
                .prepare("SELECT anonymous_id FROM hme_emails")
                .map_err(|e| format!("Failed to prepare: {}", e))?;
            let rows = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| format!("Failed to query existing IDs: {}", e))?;
            for row in rows {
                if let Ok(id) = row {
                    existing_ids.insert(id);
                }
            }
        }
        drop(conn);

        let api_ids: HashSet<String> = api_items.iter().map(|i| i.anonymous_id.clone()).collect();

        self.upsert_batch(api_items)?;

        let to_delete: Vec<String> = existing_ids.difference(&api_ids).cloned().collect();
        self.delete_batch(&to_delete)?;

        Ok(())
    }
}

fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<HmeEmailItem> {
    let anonymous_id: String = row.get(0)?;
    let label: String = row.get(1)?;
    let hme: String = row.get(2)?;
    let is_active: i32 = row.get(3)?;
    let create_timestamp: i64 = row.get(4)?;
    let tag: Option<String> = row.get(5)?;
    let tag_color: Option<String> = row.get(6)?;

    let created_at = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(
        if create_timestamp.abs() < 1_000_000_000_000 {
            create_timestamp.saturating_mul(1000)
        } else {
            create_timestamp
        },
    )
    .map(|dt| dt.to_rfc3339())
    .unwrap_or_default();

    Ok(HmeEmailItem {
        anonymous_id,
        label,
        hme,
        is_active: is_active != 0,
        create_timestamp,
        created_at,
        tag,
        tag_color,
    })
}
