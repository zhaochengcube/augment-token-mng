use crate::data::bookmark::models::Bookmark;
use crate::data::storage::common::{AccountStorage, StorageError, SyncableLocalStorage};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

/// 书签本地 SQLite 存储（原生 SQL 列，非 JSON blob）
pub struct BookmarkLocalStorage {
    db_path: PathBuf,
    lock: Mutex<()>,
}

impl BookmarkLocalStorage {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, StorageError> {
        let app_data_dir = app_handle.path().app_data_dir()?;
        std::fs::create_dir_all(&app_data_dir)?;

        let db_path = app_data_dir.join("bookmarks.db");
        let storage = Self {
            db_path,
            lock: Mutex::new(()),
        };

        storage.init_db()?;
        Ok(storage)
    }

    fn get_connection(&self) -> Result<Connection, StorageError> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        Ok(conn)
    }

    fn init_db(&self) -> Result<(), StorageError> {
        let conn = self.get_connection()?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS bookmark_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS bookmark_accounts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT,
                description TEXT,
                tag TEXT,
                tag_color TEXT,
                created_at INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL DEFAULT 0,
                version INTEGER NOT NULL DEFAULT 0,
                deleted INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS bookmark_deletions (
                id TEXT PRIMARY KEY,
                version INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_bookmark_accounts_version ON bookmark_accounts(version);
            CREATE INDEX IF NOT EXISTS idx_bookmark_accounts_deleted ON bookmark_accounts(deleted);
            CREATE INDEX IF NOT EXISTS idx_bookmark_accounts_name ON bookmark_accounts(name);
            CREATE INDEX IF NOT EXISTS idx_bookmark_accounts_tag ON bookmark_accounts(tag);
            "#,
        )?;

        // 初始化元数据
        conn.execute(
            "INSERT OR IGNORE INTO bookmark_meta (key, value) VALUES ('schema_version', '1')",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO bookmark_meta (key, value) VALUES ('version', '0')",
            [],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO bookmark_meta (key, value) VALUES ('current_account_id', '')",
            [],
        )?;

        Ok(())
    }

    fn get_meta(&self, conn: &Connection, key: &str) -> Result<Option<String>, StorageError> {
        let mut stmt = conn.prepare("SELECT value FROM bookmark_meta WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get::<_, String>(0));
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn set_meta(&self, conn: &Connection, key: &str, value: &str) -> Result<(), StorageError> {
        conn.execute(
            "INSERT OR REPLACE INTO bookmark_meta (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    fn get_store_version(&self, conn: &Connection) -> Result<i64, StorageError> {
        let value = self.get_meta(conn, "version")?;
        Ok(value.and_then(|v| v.parse().ok()).unwrap_or(0))
    }

    fn next_version(&self, conn: &Connection) -> Result<i64, StorageError> {
        let current = self.get_store_version(conn)?;
        let next = current + 1;
        self.set_meta(conn, "version", &next.to_string())?;
        Ok(next)
    }

    const INSERT_SQL: &'static str =
        "INSERT OR REPLACE INTO bookmark_accounts
            (id, name, url, description, tag, tag_color, created_at, updated_at, version, deleted)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0)";

    fn row_to_bookmark(row: &rusqlite::Row) -> rusqlite::Result<Bookmark> {
        Ok(Bookmark {
            id: row.get("id")?,
            name: row.get("name")?,
            url: row.get("url")?,
            description: row.get("description")?,
            tag: row.get("tag")?,
            tag_color: row.get("tag_color")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
            version: row.get("version")?,
            deleted: row.get("deleted")?,
        })
    }
}

#[async_trait::async_trait]
impl AccountStorage<Bookmark> for BookmarkLocalStorage {
    async fn save_account(&self, account: &Bookmark) -> Result<(), StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        let mut account = account.clone();
        account.deleted = false;

        if account.version <= 0 {
            let version = self.next_version(&conn)?;
            account.version = version;
        } else {
            let current_version = self.get_store_version(&conn)?;
            if account.version > current_version {
                self.set_meta(&conn, "version", &account.version.to_string())?;
            }
        }

        conn.execute(
            Self::INSERT_SQL,
            params![
                account.id,
                account.name,
                account.url,
                account.description,
                account.tag,
                account.tag_color,
                account.created_at,
                account.updated_at,
                account.version,
            ],
        )?;

        // 从 deletions 中移除
        conn.execute(
            "DELETE FROM bookmark_deletions WHERE id = ?1",
            params![account.id],
        )?;

        Ok(())
    }

    async fn load_accounts(&self) -> Result<Vec<Bookmark>, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, url, description, tag, tag_color, created_at, updated_at, version, deleted
             FROM bookmark_accounts WHERE deleted = 0",
        )?;

        let accounts: Vec<Bookmark> = stmt
            .query_map([], Self::row_to_bookmark)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(accounts)
    }

    async fn get_account(&self, id: &str) -> Result<Option<Bookmark>, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, url, description, tag, tag_color, created_at, updated_at, version, deleted
             FROM bookmark_accounts WHERE id = ?1 AND deleted = 0",
        )?;

        let result = stmt.query_row(params![id], Self::row_to_bookmark);
        match result {
            Ok(bookmark) => Ok(Some(bookmark)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    async fn update_account(&self, account: &Bookmark) -> Result<(), StorageError> {
        self.save_account(account).await
    }

    async fn delete_account(&self, id: &str) -> Result<bool, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        let exists = conn
            .prepare("SELECT COUNT(*) FROM bookmark_accounts WHERE id = ?1")?
            .query_row(params![id], |row| row.get::<_, i64>(0))?
            > 0;

        conn.execute(
            "DELETE FROM bookmark_accounts WHERE id = ?1",
            params![id],
        )?;

        let version = self.next_version(&conn)?;
        conn.execute(
            "INSERT OR REPLACE INTO bookmark_deletions (id, version) VALUES (?1, ?2)",
            params![id, version],
        )?;

        // 更新 current_account_id
        let current = self.get_meta(&conn, "current_account_id")?;
        if current.as_deref() == Some(id) {
            let mut stmt = conn.prepare(
                "SELECT id FROM bookmark_accounts WHERE deleted = 0 LIMIT 1",
            )?;
            let first_id: Option<String> = stmt.query_row([], |row| row.get(0)).ok();
            self.set_meta(&conn, "current_account_id", &first_id.unwrap_or_default())?;
        }

        Ok(exists)
    }

    async fn clear_all_accounts(&self) -> Result<(), StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        conn.execute("DELETE FROM bookmark_accounts", [])?;
        conn.execute("DELETE FROM bookmark_deletions", [])?;
        self.set_meta(&conn, "version", "0")?;
        self.set_meta(&conn, "current_account_id", "")?;

        Ok(())
    }

    fn storage_type(&self) -> &'static str {
        "bookmark_native_sqlite"
    }

    async fn is_available(&self) -> bool {
        self.db_path
            .parent()
            .map(|p| p.exists() || std::fs::create_dir_all(p).is_ok())
            .unwrap_or(false)
    }
}

#[async_trait::async_trait]
impl SyncableLocalStorage<Bookmark> for BookmarkLocalStorage {
    async fn get_current_account_id(&self) -> Result<Option<String>, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;
        let value = self.get_meta(&conn, "current_account_id")?;
        Ok(value.filter(|v| !v.is_empty()))
    }

    async fn set_current_account_id(&self, id: Option<String>) -> Result<(), StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;
        self.set_meta(&conn, "current_account_id", &id.unwrap_or_default())
    }

    async fn replace_all(
        &self,
        mut accounts: Vec<Bookmark>,
        deletions: Vec<String>,
        version: i64,
        current_account_id: Option<String>,
    ) -> Result<(), StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        conn.execute("DELETE FROM bookmark_accounts", [])?;
        conn.execute("DELETE FROM bookmark_deletions", [])?;

        for account in &mut accounts {
            account.deleted = false;
            conn.execute(
                Self::INSERT_SQL,
                params![
                    account.id,
                    account.name,
                    account.url,
                    account.description,
                    account.tag,
                    account.tag_color,
                    account.created_at,
                    account.updated_at,
                    account.version,
                ],
            )?;
        }

        for id in &deletions {
            conn.execute(
                "INSERT INTO bookmark_deletions (id, version) VALUES (?1, 0)",
                params![id],
            )?;
        }

        self.set_meta(&conn, "version", &version.to_string())?;
        self.set_meta(
            &conn,
            "current_account_id",
            &current_account_id.unwrap_or_default(),
        )?;

        Ok(())
    }

    fn get_local_version(&self) -> Result<i64, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;
        self.get_store_version(&conn)
    }

    fn get_deletions(&self) -> Result<Vec<String>, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare("SELECT id FROM bookmark_deletions")?;
        let ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(ids)
    }
}
