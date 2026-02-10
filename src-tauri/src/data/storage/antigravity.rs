pub mod mapper;
pub mod traits;

pub use mapper::*;
pub use traits::*;

use crate::AppState;
use crate::data::storage::common::{
    AccountSyncManager as CommonAccountSyncManager, GenericDualStorage, GenericLocalStorage,
    GenericPostgreSQLStorage,
};
use crate::platforms::antigravity::models::Account;
use std::sync::Arc;
use tauri::State;

/// Antigravity 本地存储类型别名
pub type AntigravityLocalStorage = GenericLocalStorage<Account>;

/// Antigravity PostgreSQL 存储类型别名
pub type AntigravityPostgreSQLStorage = GenericPostgreSQLStorage<Account, AntigravityAccountMapper>;

/// Antigravity 双层存储类型别名
pub type AntigravityDualStorage = GenericDualStorage<Account, AntigravityAccountMapper>;

#[tauri::command]
pub async fn antigravity_sync_accounts_to_database(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.antigravity_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Antigravity storage manager not initialized")?
    };

    storage_manager
        .sync_local_to_remote()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn antigravity_sync_accounts_from_database(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.antigravity_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Antigravity storage manager not initialized")?
    };

    storage_manager
        .sync_remote_to_local()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn antigravity_bidirectional_sync_accounts(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.antigravity_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Antigravity storage manager not initialized")?
    };

    storage_manager
        .bidirectional_sync()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn antigravity_sync_accounts(
    req_json: String,
    state: State<'_, AppState>,
) -> Result<ServerAccountSyncResponse<Account>, String> {
    let storage_manager = {
        let guard = state.antigravity_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Antigravity storage manager not initialized")?
    };

    let req: ClientAccountSyncRequest<Account> = serde_json::from_str(&req_json)
        .map_err(|e| format!("Failed to parse sync request: {}", e))?;

    storage_manager
        .sync_accounts(req)
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn antigravity_get_sync_status(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<AccountSyncStatus>, String> {
    let storage_manager = {
        let manager_option = {
            let guard = state.antigravity_storage_manager.lock().unwrap();
            guard.clone()
        };

        if let Some(manager) = manager_option {
            manager
        } else {
            if let Err(e) = initialize_antigravity_storage_manager(&app, &state).await {
                return Err(format!(
                    "Failed to initialize Antigravity storage manager: {}",
                    e
                ));
            }
            let guard = state.antigravity_storage_manager.lock().unwrap();
            guard.clone().ok_or(
                "Antigravity storage manager still not initialized after initialization attempt",
            )?
        }
    };

    storage_manager
        .get_sync_status()
        .await
        .map_err(|e| format!("Failed to get sync status: {}", e))
}

pub async fn initialize_antigravity_storage_manager(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let local_storage = Arc::new(AntigravityLocalStorage::new(app)?);

    let postgres_storage = {
        let db_manager_guard = state.database_manager.lock().unwrap();
        if let Some(db_manager) = db_manager_guard.as_ref() {
            Some(Arc::new(AntigravityPostgreSQLStorage::new(
                db_manager.clone(),
            )))
        } else {
            None
        }
    };

    let dual_storage = Arc::new(AntigravityDualStorage::new(
        local_storage,
        postgres_storage,
        false,
    ));

    *state.antigravity_storage_manager.lock().unwrap() = Some(dual_storage);

    Ok(())
}
