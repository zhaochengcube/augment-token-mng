use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GptMailRecord {
    pub id: i64,
    pub email: String,
    pub label: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_color: Option<String>,
}

pub struct GptMailStorage {
    db_path: PathBuf,
}

impl GptMailStorage {
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create GPTMail data directory: {}", e))?;

        let db_path = data_dir.join("gptmail_emails.db");
        let storage = Self { db_path };

        let mut conn = storage
            .get_connection()
            .map_err(|e| format!("Failed to open GPTMail database: {}", e))?;
        storage
            .init_schema(&mut conn)
            .map_err(|e| format!("Failed to initialize GPTMail schema: {}", e))?;

        Ok(storage)
    }

    fn get_connection(&self) -> Result<Connection, String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open GPTMail database connection: {}", e))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
            .map_err(|e| format!("Failed to set PRAGMA: {}", e))?;
        Ok(conn)
    }

    fn init_schema(&self, conn: &mut Connection) -> Result<(), String> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS gptmail_emails (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                email       TEXT NOT NULL UNIQUE,
                label       TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                created_at  TEXT NOT NULL DEFAULT '',
                updated_at  TEXT NOT NULL DEFAULT '',
                tag         TEXT DEFAULT NULL,
                tag_color   TEXT DEFAULT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_gptmail_created ON gptmail_emails(created_at DESC);",
        )
        .map_err(|e| format!("Failed to create GPTMail table: {}", e))?;
        Ok(())
    }

    pub fn load_all(&self, search: Option<&str>) -> Result<Vec<GptMailRecord>, String> {
        let conn = self.get_connection()?;

        let (sql, search_param) = match search {
            Some(keyword) if !keyword.trim().is_empty() => {
                let pattern = format!("%{}%", keyword.trim());
                (
                    "SELECT id, email, label, description, created_at, updated_at, tag, tag_color
                     FROM gptmail_emails
                     WHERE email LIKE ?1 OR label LIKE ?1 OR description LIKE ?1
                     ORDER BY created_at DESC",
                    Some(pattern),
                )
            }
            _ => (
                "SELECT id, email, label, description, created_at, updated_at, tag, tag_color
                 FROM gptmail_emails
                 ORDER BY created_at DESC",
                None,
            ),
        };

        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = if let Some(ref pattern) = search_param {
            stmt.query_map(params![pattern], row_to_record)
        } else {
            stmt.query_map([], row_to_record)
        }
        .map_err(|e| format!("Failed to query GPTMail emails: {}", e))?;

        let mut items = Vec::new();
        for row in rows {
            if let Ok(item) = row {
                items.push(item);
            }
        }
        Ok(items)
    }

    /// Upsert: insert new or update existing when email conflicts.
    /// Returns the full record with generated id and timestamps.
    pub fn save(
        &self,
        email: &str,
        label: &str,
        description: &str,
    ) -> Result<GptMailRecord, String> {
        let conn = self.get_connection()?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO gptmail_emails (email, label, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(email) DO UPDATE SET
               label = excluded.label,
               description = excluded.description,
               updated_at = excluded.updated_at",
            params![email, label, description, &now, &now],
        )
        .map_err(|e| format!("Failed to save GPTMail email: {}", e))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, email, label, description, created_at, updated_at, tag, tag_color
                 FROM gptmail_emails WHERE email = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        stmt.query_row(params![email], row_to_record)
            .map_err(|e| format!("Failed to fetch saved record: {}", e))
    }

    pub fn update(&self, id: i64, label: &str, description: &str) -> Result<(), String> {
        let conn = self.get_connection()?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE gptmail_emails SET label = ?1, description = ?2, updated_at = ?3 WHERE id = ?4",
            params![label, description, &now, id],
        )
        .map_err(|e| format!("Failed to update GPTMail email: {}", e))?;
        Ok(())
    }

    pub fn delete_batch(&self, ids: &[i64]) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut conn = self.get_connection()?;
        let tx = conn
            .transaction()
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        for id in ids {
            tx.execute("DELETE FROM gptmail_emails WHERE id = ?1", params![id])
                .map_err(|e| format!("Failed to delete GPTMail email: {}", e))?;
        }

        tx.commit()
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;
        Ok(())
    }

    pub fn update_tag(
        &self,
        id: i64,
        tag: Option<&str>,
        tag_color: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.get_connection()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE gptmail_emails SET tag = ?1, tag_color = ?2, updated_at = ?3 WHERE id = ?4",
            params![tag, tag_color, &now, id],
        )
        .map_err(|e| format!("Failed to update tag: {}", e))?;
        Ok(())
    }
}

fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<GptMailRecord> {
    Ok(GptMailRecord {
        id: row.get(0)?,
        email: row.get(1)?,
        label: row.get(2)?,
        description: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        tag: row.get(6)?,
        tag_color: row.get(7)?,
    })
}
