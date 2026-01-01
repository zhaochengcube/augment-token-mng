pub mod traits;
pub mod local_storage;
pub mod postgres_storage;
pub mod dual_storage;

pub use traits::*;
pub use local_storage::*;
pub use postgres_storage::*;
pub use dual_storage::*;

use crate::AppState;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn antigravity_sync_accounts_to_database(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.antigravity_storage_manager.lock().unwrap();
        guard.clone().ok_or("Antigravity storage manager not initialized")?
    };

    storage_manager.sync_local_to_remote().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn antigravity_sync_accounts_from_database(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.antigravity_storage_manager.lock().unwrap();
        guard.clone().ok_or("Antigravity storage manager not initialized")?
    };

    storage_manager.sync_remote_to_local().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn antigravity_bidirectional_sync_accounts(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.antigravity_storage_manager.lock().unwrap();
        guard.clone().ok_or("Antigravity storage manager not initialized")?
    };

    storage_manager.bidirectional_sync().await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn antigravity_sync_accounts(
    req_json: String,
    state: State<'_, AppState>,
) -> Result<ServerAccountSyncResponse, String> {
    let storage_manager = {
        let guard = state.antigravity_storage_manager.lock().unwrap();
        guard.clone().ok_or("Antigravity storage manager not initialized")?
    };

    let req: ClientAccountSyncRequest = serde_json::from_str(&req_json)
        .map_err(|e| format!("Failed to parse sync request: {}", e))?;

    storage_manager.sync_accounts(req).await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn get_antigravity_storage_status(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let storage_manager = {
        let guard = state.antigravity_storage_manager.lock().unwrap();
        guard.clone()
    };

    if storage_manager.is_none() {
        return Ok(serde_json::json!({
            "is_available": false,
            "storage_type": "initializing",
            "is_database_available": false,
            "is_initializing": true
        }));
    }

    let storage_manager = storage_manager.unwrap();
    let is_available = storage_manager.is_available().await;
    let storage_type = storage_manager.storage_type();
    let is_database_available = storage_manager.is_database_available();

    Ok(serde_json::json!({
        "is_available": is_available,
        "storage_type": storage_type,
        "is_database_available": is_database_available,
        "is_initializing": false
    }))
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
                return Err(format!("Failed to initialize Antigravity storage manager: {}", e));
            }
            let guard = state.antigravity_storage_manager.lock().unwrap();
            guard.clone().ok_or("Antigravity storage manager still not initialized after initialization attempt")?
        }
    };

    storage_manager.get_sync_status().await
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
            Some(Arc::new(AntigravityPostgreSQLStorage::new(db_manager.clone())))
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
