pub mod traits;
pub mod mapper;

pub use traits::*;
pub use mapper::*;

use crate::data::storage::common::{
    GenericLocalStorage, GenericPostgreSQLStorage, GenericDualStorage,
    AccountSyncManager as CommonAccountSyncManager,
};
use crate::platforms::openai::models::Account;
use crate::AppState;
use std::sync::Arc;
use tauri::State;

/// OpenAI 本地存储类型别名
pub type OpenAILocalStorage = GenericLocalStorage<Account>;

/// OpenAI PostgreSQL 存储类型别名
pub type OpenAIPostgreSQLStorage = GenericPostgreSQLStorage<Account, OpenAIAccountMapper>;

/// OpenAI 双层存储类型别名
pub type OpenAIDualStorage = GenericDualStorage<Account, OpenAIAccountMapper>;

#[tauri::command]
pub async fn openai_sync_accounts_to_database(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.openai_storage_manager.lock().unwrap();
        guard.clone().ok_or("OpenAI storage manager not initialized")?
    };

    storage_manager.sync_local_to_remote().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn openai_sync_accounts_from_database(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.openai_storage_manager.lock().unwrap();
        guard.clone().ok_or("OpenAI storage manager not initialized")?
    };

    storage_manager.sync_remote_to_local().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn openai_bidirectional_sync_accounts(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.openai_storage_manager.lock().unwrap();
        guard.clone().ok_or("OpenAI storage manager not initialized")?
    };

    storage_manager.bidirectional_sync().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn openai_sync_accounts(
    req_json: String,
    state: State<'_, AppState>,
) -> Result<ServerAccountSyncResponse<Account>, String> {
    let storage_manager = {
        let guard = state.openai_storage_manager.lock().unwrap();
        guard.clone().ok_or("OpenAI storage manager not initialized")?
    };

    let req: ClientAccountSyncRequest<Account> = serde_json::from_str(&req_json)
        .map_err(|e| format!("Failed to parse sync request: {}", e))?;

    storage_manager.sync_accounts(req).await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn openai_get_sync_status(
    state: State<'_, AppState>,
) -> Result<Option<AccountSyncStatus>, String> {
    let storage_manager = {
        let guard = state.openai_storage_manager.lock().unwrap();
        guard.clone().ok_or("OpenAI storage manager not initialized")?
    };

    storage_manager.get_sync_status().await
        .map_err(|e| format!("Failed to get sync status: {}", e))
}

pub async fn initialize_openai_storage_manager(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let local_storage = Arc::new(OpenAILocalStorage::new(app)?);

    let postgres_storage = {
        let db_manager_guard = state.database_manager.lock().unwrap();
        if let Some(db_manager) = db_manager_guard.as_ref() {
            Some(Arc::new(OpenAIPostgreSQLStorage::new(db_manager.clone())))
        } else {
            None
        }
    };

    let dual_storage = Arc::new(OpenAIDualStorage::new(
        local_storage,
        postgres_storage,
        false,
    ));

    *state.openai_storage_manager.lock().unwrap() = Some(dual_storage);

    Ok(())
}

