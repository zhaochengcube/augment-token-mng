use crate::data::storage::common::{AccountStorage, GenericLocalStorage};
use crate::cursor::models::{Account, AccountIndex, AccountSummary};
use std::sync::Arc;

/// Cursor 本地存储 (使用通用层)
pub type CursorLocalStorage = GenericLocalStorage<Account>;

/// 创建 Cursor 本地存储实例
pub fn create_cursor_storage(
    app_handle: &tauri::AppHandle,
) -> Result<Arc<CursorLocalStorage>, String> {
    CursorLocalStorage::new(app_handle)
        .map(Arc::new)
        .map_err(|e| format!("Failed to create Cursor storage: {}", e))
}

/// 加载账号索引 (兼容旧接口)
pub async fn load_index(app_handle: &tauri::AppHandle) -> Result<AccountIndex, String> {
    let storage = create_cursor_storage(app_handle)?;
    let accounts = storage
        .load_accounts()
        .await
        .map_err(|e| format!("Failed to load accounts: {}", e))?;

    let current_account_id = storage
        .get_current_account_id()
        .await
        .map_err(|e| format!("Failed to get current account id: {}", e))?;

    let summaries: Vec<AccountSummary> = accounts
        .iter()
        .map(|a| AccountSummary {
            id: a.id.clone(),
            email: a.email.clone(),
            name: a.name.clone(),
            created_at: a.created_at,
            last_used: a.last_used,
        })
        .collect();

    Ok(AccountIndex {
        version: "1.0".to_string(),
        accounts: summaries,
        current_account_id,
    })
}

/// 加载单个账号
pub async fn load_account(app_handle: &tauri::AppHandle, account_id: &str) -> Result<Account, String> {
    let storage = create_cursor_storage(app_handle)?;
    storage
        .get_account(account_id)
        .await
        .map_err(|e| format!("Failed to load account: {}", e))?
        .ok_or_else(|| format!("Account not found: {}", account_id))
}

/// 保存单个账号
pub async fn save_account(app_handle: &tauri::AppHandle, account: &Account) -> Result<(), String> {
    let storage = create_cursor_storage(app_handle)?;
    storage
        .save_account(account)
        .await
        .map_err(|e| format!("Failed to save account: {}", e))
}

/// 删除账号
pub async fn delete_account(app_handle: &tauri::AppHandle, account_id: &str) -> Result<bool, String> {
    let storage = create_cursor_storage(app_handle)?;
    storage
        .delete_account(account_id)
        .await
        .map_err(|e| format!("Failed to delete account: {}", e))
}

/// 列出所有账号
pub async fn list_accounts(app_handle: &tauri::AppHandle) -> Result<Vec<Account>, String> {
    let storage = create_cursor_storage(app_handle)?;
    storage
        .load_accounts()
        .await
        .map_err(|e| format!("Failed to list accounts: {}", e))
}

/// 获取当前账号 ID
pub async fn get_current_account_id(app_handle: &tauri::AppHandle) -> Result<Option<String>, String> {
    let storage = create_cursor_storage(app_handle)?;
    storage
        .get_current_account_id()
        .await
        .map_err(|e| format!("Failed to get current account id: {}", e))
}

/// 设置当前账号 ID
pub async fn set_current_account_id(app_handle: &tauri::AppHandle, account_id: Option<String>) -> Result<(), String> {
    let storage = create_cursor_storage(app_handle)?;
    storage
        .set_current_account_id(account_id)
        .await
        .map_err(|e| format!("Failed to set current account id: {}", e))
}

