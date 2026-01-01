use crate::storage::initialize_antigravity_storage_manager;
use crate::storage::antigravity::traits::AccountStorage;
use crate::{storage::AntigravityDualStorage, AppState};
use crate::platforms::antigravity::models::Account;
use std::sync::Arc;
use tauri::Manager;

async fn get_storage_manager(app_handle: &tauri::AppHandle) -> Result<Arc<AntigravityDualStorage>, String> {
    let state = app_handle.state::<AppState>();
    let manager_option = {
        let guard = state.antigravity_storage_manager.lock().unwrap();
        guard.clone()
    };

    if let Some(manager) = manager_option {
        Ok(manager)
    } else {
        initialize_antigravity_storage_manager(app_handle, &state).await
            .map_err(|e| format!("Failed to initialize Antigravity storage manager: {}", e))?;
        let guard = state.antigravity_storage_manager.lock().unwrap();
        guard.clone().ok_or("Antigravity storage manager still not initialized".to_string())
    }
}

pub async fn list_accounts(app_handle: &tauri::AppHandle) -> Result<Vec<Account>, String> {
    let manager = get_storage_manager(app_handle).await?;
    manager.load_accounts().await
        .map_err(|e| format!("Failed to load accounts: {}", e))
}

pub async fn load_account(app_handle: &tauri::AppHandle, account_id: &str) -> Result<Account, String> {
    let manager = get_storage_manager(app_handle).await?;
    let account = manager.get_account(account_id).await
        .map_err(|e| format!("Failed to load account: {}", e))?;

    account.ok_or_else(|| format!("Account not found: {}", account_id))
}

pub async fn save_account(app_handle: &tauri::AppHandle, account: &Account) -> Result<(), String> {
    let manager = get_storage_manager(app_handle).await?;
    manager.save_account(account).await
        .map_err(|e| format!("Failed to save account: {}", e))
}

pub async fn delete_account(app_handle: &tauri::AppHandle, account_id: &str) -> Result<bool, String> {
    let manager = get_storage_manager(app_handle).await?;
    manager.delete_account(account_id).await
        .map_err(|e| format!("Failed to delete account: {}", e))
}

pub async fn get_current_account_id(app_handle: &tauri::AppHandle) -> Result<Option<String>, String> {
    let manager = get_storage_manager(app_handle).await?;
    manager.get_current_account_id().await
        .map_err(|e| format!("Failed to get current account: {}", e))
}

pub async fn set_current_account_id(app_handle: &tauri::AppHandle, account_id: Option<String>) -> Result<(), String> {
    let manager = get_storage_manager(app_handle).await?;
    manager.set_current_account_id(account_id).await
        .map_err(|e| format!("Failed to update current account: {}", e))
}
