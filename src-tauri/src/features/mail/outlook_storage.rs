use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlookCredentialRecord {
    pub email: String,
    pub refresh_token: String,
    pub client_id: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

pub struct OutlookStorage {
    db_path: PathBuf,
}

impl OutlookStorage {
    pub fn new(data_dir: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create Outlook data directory: {}", e))?;

        let db_path = data_dir.join("outlook_credentials.db");
        let storage = Self { db_path };

        let mut conn = storage
            .get_connection()
            .map_err(|e| format!("Failed to open Outlook database: {}", e))?;
        storage
            .init_schema(&mut conn)
            .map_err(|e| format!("Failed to initialize Outlook schema: {}", e))?;

        Ok(storage)
    }

    fn get_connection(&self) -> Result<Connection, String> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("Failed to open Outlook database connection: {}", e))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
            .map_err(|e| format!("Failed to set PRAGMA: {}", e))?;
        Ok(conn)
    }

    fn init_schema(&self, conn: &mut Connection) -> Result<(), String> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS outlook_credentials (
                email         TEXT PRIMARY KEY,
                refresh_token TEXT NOT NULL,
                client_id     TEXT NOT NULL,
                created_at    TEXT NOT NULL DEFAULT '',
                updated_at    TEXT NOT NULL DEFAULT '',
                tag           TEXT DEFAULT NULL,
                tag_color     TEXT DEFAULT NULL,
                status        TEXT DEFAULT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_outlook_created ON outlook_credentials(created_at DESC);",
        )
        .map_err(|e| format!("Failed to create Outlook table: {}", e))?;

        // 迁移: 为旧表添加 status 列
        let _ = conn.execute_batch("ALTER TABLE outlook_credentials ADD COLUMN status TEXT DEFAULT NULL;");

        Ok(())
    }

    pub fn load_all(&self) -> Result<Vec<OutlookCredentialRecord>, String> {
        let conn = self.get_connection()?;

        let mut stmt = conn
            .prepare(
                "SELECT email, refresh_token, client_id, created_at, updated_at, tag, tag_color, status
                 FROM outlook_credentials
                 ORDER BY created_at DESC",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], row_to_record)
            .map_err(|e| format!("Failed to query Outlook credentials: {}", e))?;

        let mut items = Vec::new();
        for row in rows {
            if let Ok(item) = row {
                items.push(item);
            }
        }
        Ok(items)
    }

    pub fn save(
        &self,
        email: &str,
        refresh_token: &str,
        client_id: &str,
    ) -> Result<OutlookCredentialRecord, String> {
        let conn = self.get_connection()?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO outlook_credentials (email, refresh_token, client_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(email) DO UPDATE SET
               refresh_token = excluded.refresh_token,
               client_id = excluded.client_id,
               updated_at = excluded.updated_at",
            params![email, refresh_token, client_id, &now, &now],
        )
        .map_err(|e| format!("Failed to save Outlook credential: {}", e))?;

        let mut stmt = conn
            .prepare(
                "SELECT email, refresh_token, client_id, created_at, updated_at, tag, tag_color, status
                 FROM outlook_credentials WHERE email = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        stmt.query_row(params![email], row_to_record)
            .map_err(|e| format!("Failed to fetch saved record: {}", e))
    }

    pub fn delete(&self, email: &str) -> Result<bool, String> {
        let conn = self.get_connection()?;
        let affected = conn
            .execute(
                "DELETE FROM outlook_credentials WHERE email = ?1",
                params![email],
            )
            .map_err(|e| format!("Failed to delete Outlook credential: {}", e))?;
        Ok(affected > 0)
    }

    pub fn update_refresh_token(
        &self,
        email: &str,
        new_refresh_token: &str,
    ) -> Result<(), String> {
        let conn = self.get_connection()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE outlook_credentials SET refresh_token = ?1, updated_at = ?2 WHERE email = ?3",
            params![new_refresh_token, &now, email],
        )
        .map_err(|e| format!("Failed to update refresh token: {}", e))?;
        Ok(())
    }

    pub fn update_status(&self, email: &str, status: &str) -> Result<(), String> {
        let conn = self.get_connection()?;
        conn.execute(
            "UPDATE outlook_credentials SET status = ?1 WHERE email = ?2",
            params![status, email],
        )
        .map_err(|e| format!("Failed to update status: {}", e))?;
        Ok(())
    }

    pub fn load_all_statuses(&self) -> Result<Vec<(String, String)>, String> {
        let conn = self.get_connection()?;
        let mut stmt = conn
            .prepare("SELECT email, status FROM outlook_credentials WHERE status IS NOT NULL")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| format!("Failed to query statuses: {}", e))?;
        let mut result = Vec::new();
        for row in rows {
            if let Ok(item) = row {
                result.push(item);
            }
        }
        Ok(result)
    }

    pub fn update_tag(
        &self,
        email: &str,
        tag: Option<&str>,
        tag_color: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.get_connection()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE outlook_credentials SET tag = ?1, tag_color = ?2, updated_at = ?3 WHERE email = ?4",
            params![tag, tag_color, &now, email],
        )
        .map_err(|e| format!("Failed to update tag: {}", e))?;
        Ok(())
    }
}

fn row_to_record(row: &rusqlite::Row) -> rusqlite::Result<OutlookCredentialRecord> {
    Ok(OutlookCredentialRecord {
        email: row.get(0)?,
        refresh_token: row.get(1)?,
        client_id: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        tag: row.get(5)?,
        tag_color: row.get(6)?,
        status: row.get(7)?,
    })
}
