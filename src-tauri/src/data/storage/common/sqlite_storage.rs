use super::traits::{AccountStorage, StorageError, SyncableAccount, SyncableLocalStorage};
use rusqlite::{Connection, params};
use serde_json;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

const SCHEMA_VERSION: i64 = 1;

/// 通用 SQLite 本地存储
/// 使用 JSON 序列化存储每条记录，配合 SQLite 索引实现高效查询
pub struct GenericSQLiteStorage<T: SyncableAccount> {
    db_path: PathBuf,
    lock: Mutex<()>,
    _phantom: PhantomData<T>,
}

impl<T: SyncableAccount> GenericSQLiteStorage<T> {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, StorageError> {
        let app_data_dir = app_handle.path().app_data_dir()?;
        std::fs::create_dir_all(&app_data_dir)?;

        let db_path = app_data_dir.join(format!("{}_storage.db", T::platform_name()));

        let storage = Self {
            db_path,
            lock: Mutex::new(()),
            _phantom: PhantomData,
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

        conn.execute_batch(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {table}_meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS {table}_accounts (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 0,
                deleted INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS {table}_deletions (
                id TEXT PRIMARY KEY,
                version INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_{table}_accounts_version ON {table}_accounts(version);
            CREATE INDEX IF NOT EXISTS idx_{table}_accounts_deleted ON {table}_accounts(deleted);
            "#,
            table = T::platform_name()
        ))?;

        // 初始化 schema_version
        conn.execute(
            &format!(
                "INSERT OR IGNORE INTO {}_meta (key, value) VALUES ('schema_version', ?1)",
                T::platform_name()
            ),
            params![SCHEMA_VERSION.to_string()],
        )?;

        // 初始化 version
        conn.execute(
            &format!(
                "INSERT OR IGNORE INTO {}_meta (key, value) VALUES ('version', '0')",
                T::platform_name()
            ),
            [],
        )?;

        // 初始化 current_account_id
        conn.execute(
            &format!(
                "INSERT OR IGNORE INTO {}_meta (key, value) VALUES ('current_account_id', '')",
                T::platform_name()
            ),
            [],
        )?;

        Ok(())
    }

    fn get_meta(&self, conn: &Connection, key: &str) -> Result<Option<String>, StorageError> {
        let mut stmt = conn.prepare(&format!(
            "SELECT value FROM {}_meta WHERE key = ?1",
            T::platform_name()
        ))?;

        let result = stmt.query_row(params![key], |row| row.get::<_, String>(0));

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn set_meta(&self, conn: &Connection, key: &str, value: &str) -> Result<(), StorageError> {
        conn.execute(
            &format!(
                "INSERT OR REPLACE INTO {}_meta (key, value) VALUES (?1, ?2)",
                T::platform_name()
            ),
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

    pub async fn get_current_account_id(&self) -> Result<Option<String>, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;
        let value = self.get_meta(&conn, "current_account_id")?;
        Ok(value.filter(|v| !v.is_empty()))
    }

    pub async fn set_current_account_id(&self, id: Option<String>) -> Result<(), StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;
        self.set_meta(&conn, "current_account_id", &id.unwrap_or_default())
    }

    pub async fn replace_all(
        &self,
        mut accounts: Vec<T>,
        deletions: Vec<String>,
        version: i64,
        current_account_id: Option<String>,
    ) -> Result<(), StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        conn.execute(
            &format!("DELETE FROM {}_accounts", T::platform_name()),
            [],
        )?;
        conn.execute(
            &format!("DELETE FROM {}_deletions", T::platform_name()),
            [],
        )?;

        for account in &mut accounts {
            account.set_deleted(false);
            let data = serde_json::to_string(account)?;
            conn.execute(
                &format!(
                    "INSERT INTO {}_accounts (id, data, version, deleted, updated_at) VALUES (?1, ?2, ?3, 0, ?4)",
                    T::platform_name()
                ),
                params![account.id(), data, account.version(), account.updated_at()],
            )?;
        }

        for id in &deletions {
            conn.execute(
                &format!(
                    "INSERT INTO {}_deletions (id, version) VALUES (?1, 0)",
                    T::platform_name()
                ),
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

    pub fn get_local_version(&self) -> Result<i64, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;
        self.get_store_version(&conn)
    }

    pub fn get_deletions(&self) -> Result<Vec<String>, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(&format!(
            "SELECT id FROM {}_deletions",
            T::platform_name()
        ))?;

        let ids: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(ids)
    }
}

#[async_trait::async_trait]
impl<T: SyncableAccount> AccountStorage<T> for GenericSQLiteStorage<T> {
    async fn save_account(&self, account: &T) -> Result<(), StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        let mut account = account.clone();
        account.set_deleted(false);

        if account.version() <= 0 {
            let version = self.next_version(&conn)?;
            account.set_version(version);
        } else {
            let current_version = self.get_store_version(&conn)?;
            if account.version() > current_version {
                self.set_meta(&conn, "version", &account.version().to_string())?;
            }
        }

        let data = serde_json::to_string(&account)?;
        let id = account.id().to_string();

        conn.execute(
            &format!(
                "INSERT OR REPLACE INTO {}_accounts (id, data, version, deleted, updated_at) VALUES (?1, ?2, ?3, 0, ?4)",
                T::platform_name()
            ),
            params![id, data, account.version(), account.updated_at()],
        )?;

        // 从 deletions 中移除
        conn.execute(
            &format!(
                "DELETE FROM {}_deletions WHERE id = ?1",
                T::platform_name()
            ),
            params![id],
        )?;

        Ok(())
    }

    async fn load_accounts(&self) -> Result<Vec<T>, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(&format!(
            "SELECT data FROM {}_accounts WHERE deleted = 0",
            T::platform_name()
        ))?;

        let accounts: Vec<T> = stmt
            .query_map([], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })?
            .filter_map(|r| r.ok())
            .filter_map(|data| serde_json::from_str::<T>(&data).ok())
            .filter(|a| !a.is_deleted())
            .collect();

        Ok(accounts)
    }

    async fn get_account(&self, id: &str) -> Result<Option<T>, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(&format!(
            "SELECT data FROM {}_accounts WHERE id = ?1 AND deleted = 0",
            T::platform_name()
        ))?;

        let result = stmt.query_row(params![id], |row| {
            let data: String = row.get(0)?;
            Ok(data)
        });

        match result {
            Ok(data) => {
                let account: T = serde_json::from_str(&data)?;
                if account.is_deleted() {
                    Ok(None)
                } else {
                    Ok(Some(account))
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(Box::new(e)),
        }
    }

    async fn update_account(&self, account: &T) -> Result<(), StorageError> {
        self.save_account(account).await
    }

    async fn delete_account(&self, id: &str) -> Result<bool, StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        let exists = conn
            .prepare(&format!(
                "SELECT COUNT(*) FROM {}_accounts WHERE id = ?1",
                T::platform_name()
            ))?
            .query_row(params![id], |row| row.get::<_, i64>(0))?
            > 0;

        conn.execute(
            &format!(
                "DELETE FROM {}_accounts WHERE id = ?1",
                T::platform_name()
            ),
            params![id],
        )?;

        let version = self.next_version(&conn)?;
        conn.execute(
            &format!(
                "INSERT OR REPLACE INTO {}_deletions (id, version) VALUES (?1, ?2)",
                T::platform_name()
            ),
            params![id, version],
        )?;

        // 更新 current_account_id
        let current = self.get_meta(&conn, "current_account_id")?;
        if current.as_deref() == Some(id) {
            let mut stmt = conn.prepare(&format!(
                "SELECT id FROM {}_accounts WHERE deleted = 0 LIMIT 1",
                T::platform_name()
            ))?;
            let first_id: Option<String> = stmt
                .query_row([], |row| row.get(0))
                .ok();
            self.set_meta(&conn, "current_account_id", &first_id.unwrap_or_default())?;
        }

        Ok(exists)
    }

    async fn clear_all_accounts(&self) -> Result<(), StorageError> {
        let _guard = self.lock.lock().unwrap();
        let conn = self.get_connection()?;

        conn.execute(
            &format!("DELETE FROM {}_accounts", T::platform_name()),
            [],
        )?;
        conn.execute(
            &format!("DELETE FROM {}_deletions", T::platform_name()),
            [],
        )?;
        self.set_meta(&conn, "version", "0")?;
        self.set_meta(&conn, "current_account_id", "")?;

        Ok(())
    }

    fn storage_type(&self) -> &'static str {
        "generic_sqlite"
    }

    async fn is_available(&self) -> bool {
        self.db_path
            .parent()
            .map(|p| p.exists() || std::fs::create_dir_all(p).is_ok())
            .unwrap_or(false)
    }
}

#[async_trait::async_trait]
impl<T: SyncableAccount> SyncableLocalStorage<T> for GenericSQLiteStorage<T> {
    async fn get_current_account_id(&self) -> Result<Option<String>, StorageError> {
        GenericSQLiteStorage::get_current_account_id(self).await
    }

    async fn set_current_account_id(&self, id: Option<String>) -> Result<(), StorageError> {
        GenericSQLiteStorage::set_current_account_id(self, id).await
    }

    async fn replace_all(
        &self,
        accounts: Vec<T>,
        deletions: Vec<String>,
        version: i64,
        current_account_id: Option<String>,
    ) -> Result<(), StorageError> {
        GenericSQLiteStorage::replace_all(self, accounts, deletions, version, current_account_id).await
    }

    fn get_local_version(&self) -> Result<i64, StorageError> {
        GenericSQLiteStorage::get_local_version(self)
    }

    fn get_deletions(&self) -> Result<Vec<String>, StorageError> {
        GenericSQLiteStorage::get_deletions(self)
    }
}
