pub mod traits;

pub use traits::*;
// 使用 platforms 中已定义的 mapper
pub use crate::platforms::cursor::modules::mapper::CursorAccountMapper;

use crate::AppState;
use crate::data::storage::common::{
    AccountSyncManager as CommonAccountSyncManager, GenericDualStorage, GenericLocalStorage,
    GenericPostgreSQLStorage,
};
use crate::platforms::cursor::models::Account;
use std::sync::Arc;
use tauri::State;

/// Cursor 本地存储类型别名
pub type CursorLocalStorage = GenericLocalStorage<Account>;

/// Cursor PostgreSQL 存储类型别名
pub type CursorPostgreSQLStorage = GenericPostgreSQLStorage<Account, CursorAccountMapper>;

/// Cursor 双层存储类型别名
pub type CursorDualStorage = GenericDualStorage<Account, CursorAccountMapper>;

#[tauri::command]
pub async fn cursor_sync_accounts_to_database(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.cursor_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Cursor storage manager not initialized")?
    };

    storage_manager
        .sync_local_to_remote()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn cursor_sync_accounts_from_database(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.cursor_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Cursor storage manager not initialized")?
    };

    storage_manager
        .sync_remote_to_local()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn cursor_bidirectional_sync_accounts(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.cursor_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Cursor storage manager not initialized")?
    };

    storage_manager
        .bidirectional_sync()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn cursor_sync_accounts(
    req_json: String,
    state: State<'_, AppState>,
) -> Result<ServerAccountSyncResponse<Account>, String> {
    let storage_manager = {
        let guard = state.cursor_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Cursor storage manager not initialized")?
    };

    let req: ClientAccountSyncRequest<Account> = serde_json::from_str(&req_json)
        .map_err(|e| format!("Failed to parse sync request: {}", e))?;

    let upserts_len = req.upserts.len();
    let deletions_len = req.deletions.len();
    let last_version = req.last_version;

    match storage_manager.sync_accounts(req).await {
        Ok(res) => Ok(res),
        Err(e) => {
            println!(
                "Cursor sync_accounts failed (last_version={}, upserts={}, deletions={}): {}",
                last_version, upserts_len, deletions_len, e
            );
            Err(format!("Sync failed: {}", e))
        }
    }
}

#[tauri::command]
pub async fn cursor_get_sync_status(
    state: State<'_, AppState>,
) -> Result<Option<AccountSyncStatus>, String> {
    let storage_manager = {
        let guard = state.cursor_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Cursor storage manager not initialized")?
    };

    storage_manager
        .get_sync_status()
        .await
        .map_err(|e| format!("Failed to get sync status: {}", e))
}

pub async fn initialize_cursor_storage_manager(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let local_storage = Arc::new(CursorLocalStorage::new(app)?);

    let postgres_storage = {
        let db_manager_guard = state.database_manager.lock().unwrap();
        if let Some(db_manager) = db_manager_guard.as_ref() {
            Some(Arc::new(CursorPostgreSQLStorage::new(db_manager.clone())))
        } else {
            None
        }
    };

    let dual_storage = Arc::new(CursorDualStorage::new(
        local_storage,
        postgres_storage,
        false,
    ));

    *state.cursor_storage_manager.lock().unwrap() = Some(dual_storage);

    Ok(())
}
