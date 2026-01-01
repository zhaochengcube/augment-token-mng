use super::traits::{AccountStorage};
use crate::platforms::antigravity::models::Account;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

const STORAGE_FILE: &str = "antigravity_accounts.json";
const SCHEMA_VERSION: i32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeletedAccount {
    id: String,
    version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountStore {
    schema_version: i32,
    version: i64,
    current_account_id: Option<String>,
    accounts: Vec<Account>,
    deletions: Vec<DeletedAccount>,
}

impl AccountStore {
    fn new() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            version: 0,
            current_account_id: None,
            accounts: Vec::new(),
            deletions: Vec::new(),
        }
    }

    fn next_version(&mut self) -> i64 {
        self.version += 1;
        self.version
    }
}

pub struct AntigravityLocalStorage {
    storage_path: PathBuf,
    _lock: Mutex<()>,
}

impl AntigravityLocalStorage {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let app_data_dir = app_handle.path().app_data_dir()?;
        fs::create_dir_all(&app_data_dir)?;

        Ok(Self {
            storage_path: app_data_dir.join(STORAGE_FILE),
            _lock: Mutex::new(()),
        })
    }

    pub fn new_with_path(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            _lock: Mutex::new(()),
        }
    }

    async fn read_store(&self) -> Result<AccountStore, Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self._lock.lock().unwrap();

        if !self.storage_path.exists() {
            return Ok(AccountStore::new());
        }

        let content = fs::read_to_string(&self.storage_path)?;
        if content.trim().is_empty() {
            return Ok(AccountStore::new());
        }

        let store: AccountStore = serde_json::from_str(&content)?;
        if store.schema_version != SCHEMA_VERSION {
            return Err(format!("Unsupported Antigravity account schema version: {}", store.schema_version).into());
        }

        Ok(store)
    }

    async fn write_store(&self, store: &AccountStore) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self._lock.lock().unwrap();

        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = self.storage_path.with_extension("tmp");
        let content = serde_json::to_string_pretty(store)?;

        fs::write(&temp_path, content)?;

        match fs::rename(&temp_path, &self.storage_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                let _ = fs::remove_file(&temp_path);

                if !temp_path.exists() {
                    return Err(format!("Failed to rename temp file (temp file was removed): {}", e).into());
                }

                if let Some(parent) = self.storage_path.parent() {
                    if !parent.exists() {
                        return Err(format!("Failed to rename temp file (parent directory disappeared): {}", e).into());
                    }
                }

                #[cfg(target_os = "windows")]
                {
                    if self.storage_path.exists() {
                        if let Err(remove_err) = fs::remove_file(&self.storage_path) {
                            let _ = fs::remove_file(&temp_path);
                            return Err(format!("Failed to remove existing file before rename: {}", remove_err).into());
                        }
                    }

                    match fs::rename(&temp_path, &self.storage_path) {
                        Ok(_) => return Ok(()),
                        Err(retry_err) => {
                            let _ = fs::remove_file(&temp_path);
                            return Err(format!("Failed to rename temp file after retry: {}", retry_err).into());
                        }
                    }
                }

                #[cfg(not(target_os = "windows"))]
                {
                    Err(format!("Failed to rename temp file: {}", e).into())
                }
            }
        }
    }

    pub async fn replace_all(
        &self,
        mut accounts: Vec<Account>,
        deletions: Vec<String>,
        version: i64,
        current_account_id: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for account in &mut accounts {
            account.deleted = false;
        }

        let store = AccountStore {
            schema_version: SCHEMA_VERSION,
            version,
            current_account_id,
            accounts,
            deletions: deletions.into_iter().map(|id| DeletedAccount { id, version: 0 }).collect(),
        };

        self.write_store(&store).await
    }

    pub async fn get_current_account_id(&self) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let store = self.read_store().await?;
        Ok(store.current_account_id)
    }

    pub async fn set_current_account_id(&self, account_id: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut store = self.read_store().await?;
        store.current_account_id = account_id;
        self.write_store(&store).await
    }
}

#[async_trait::async_trait]
impl AccountStorage for AntigravityLocalStorage {
    async fn save_account(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut store = self.read_store().await?;
        let mut account = account.clone();

        account.deleted = false;

        if account.version <= 0 {
            account.version = store.next_version();
        } else if account.version > store.version {
            store.version = account.version;
        }

        let account_id = account.id.clone();
        store.accounts.retain(|a| a.id != account_id);
        store.accounts.push(account);
        store.deletions.retain(|d| d.id != account_id);

        self.write_store(&store).await
    }

    async fn load_accounts(&self) -> Result<Vec<Account>, Box<dyn std::error::Error + Send + Sync>> {
        let store = self.read_store().await?;
        Ok(store.accounts.into_iter().filter(|a| !a.deleted).collect())
    }

    async fn update_account(&self, account: &Account) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.save_account(account).await
    }

    async fn delete_account(&self, account_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut store = self.read_store().await?;
        let initial_len = store.accounts.len();

        store.accounts.retain(|a| a.id != account_id);
        store.deletions.retain(|d| d.id != account_id);

        let version = store.next_version();
        store.deletions.push(DeletedAccount {
            id: account_id.to_string(),
            version,
        });

        if store.current_account_id.as_deref() == Some(account_id) {
            store.current_account_id = store.accounts.first().map(|account| account.id.clone());
        }

        self.write_store(&store).await?;

        Ok(store.accounts.len() < initial_len)
    }

    async fn get_account(&self, account_id: &str) -> Result<Option<Account>, Box<dyn std::error::Error + Send + Sync>> {
        let store = self.read_store().await?;
        Ok(store.accounts.into_iter().find(|a| a.id == account_id && !a.deleted))
    }

    async fn clear_all_accounts(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.write_store(&AccountStore::new()).await
    }

    fn storage_type(&self) -> &'static str {
        "antigravity_local_file"
    }

    async fn is_available(&self) -> bool {
        if let Some(parent) = self.storage_path.parent() {
            parent.exists() || fs::create_dir_all(parent).is_ok()
        } else {
            false
        }
    }
}
