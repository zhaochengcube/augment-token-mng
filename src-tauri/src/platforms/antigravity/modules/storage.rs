use std::path::PathBuf;
use std::fs;
use tauri::Manager;
use crate::antigravity::models::{Account, AccountIndex, AccountSummary};

const ACCOUNTS_INDEX_FILE: &str = "antigravity_accounts.json";
const ACCOUNTS_DIR: &str = "antigravity_accounts";

/// 获取 Antigravity 数据目录（使用 app_data_dir）
pub fn get_antigravity_data_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    // 确保目录存在
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    
    Ok(app_data_dir)
}

/// 获取账号索引文件路径
pub fn get_accounts_index_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = get_antigravity_data_dir(app_handle)?;
    Ok(data_dir.join(ACCOUNTS_INDEX_FILE))
}

/// 获取账号详情目录
pub fn get_accounts_dir(app_handle: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = get_antigravity_data_dir(app_handle)?;
    let accounts_dir = data_dir.join(ACCOUNTS_DIR);
    
    if !accounts_dir.exists() {
        fs::create_dir_all(&accounts_dir)
            .map_err(|e| format!("Failed to create accounts directory: {}", e))?;
    }
    
    Ok(accounts_dir)
}

/// 加载账号索引
pub fn load_account_index(app_handle: &tauri::AppHandle) -> Result<AccountIndex, String> {
    let index_path = get_accounts_index_path(app_handle)?;
    
    if !index_path.exists() {
        return Ok(AccountIndex::new());
    }
    
    let content = fs::read_to_string(&index_path)
        .map_err(|e| format!("Failed to read account index: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse account index: {}", e))
}

/// 保存账号索引
pub fn save_account_index(app_handle: &tauri::AppHandle, index: &AccountIndex) -> Result<(), String> {
    let index_path = get_accounts_index_path(app_handle)?;
    
    let content = serde_json::to_string_pretty(index)
        .map_err(|e| format!("Failed to serialize account index: {}", e))?;
    
    fs::write(&index_path, content)
        .map_err(|e| format!("Failed to write account index: {}", e))
}

/// 加载单个账号
pub fn load_account(app_handle: &tauri::AppHandle, account_id: &str) -> Result<Account, String> {
    let accounts_dir = get_accounts_dir(app_handle)?;
    let account_path = accounts_dir.join(format!("{}.json", account_id));
    
    if !account_path.exists() {
        return Err(format!("Account not found: {}", account_id));
    }
    
    let content = fs::read_to_string(&account_path)
        .map_err(|e| format!("Failed to read account file: {}", e))?;
    
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse account: {}", e))
}

/// 保存单个账号
pub fn save_account(app_handle: &tauri::AppHandle, account: &Account) -> Result<(), String> {
    let accounts_dir = get_accounts_dir(app_handle)?;
    let account_path = accounts_dir.join(format!("{}.json", account.id));
    
    let content = serde_json::to_string_pretty(account)
        .map_err(|e| format!("Failed to serialize account: {}", e))?;
    
    fs::write(&account_path, content)
        .map_err(|e| format!("Failed to write account file: {}", e))
}

/// 删除账号文件
pub fn delete_account_file(app_handle: &tauri::AppHandle, account_id: &str) -> Result<(), String> {
    let accounts_dir = get_accounts_dir(app_handle)?;
    let account_path = accounts_dir.join(format!("{}.json", account_id));
    
    if account_path.exists() {
        fs::remove_file(&account_path)
            .map_err(|e| format!("Failed to delete account file: {}", e))?;
    }
    
    Ok(())
}

/// 列出所有账号
pub fn list_accounts(app_handle: &tauri::AppHandle) -> Result<Vec<Account>, String> {
    let index = load_account_index(app_handle)?;
    let mut accounts = Vec::new();
    
    for summary in &index.accounts {
        match load_account(app_handle, &summary.id) {
            Ok(account) => accounts.push(account),
            Err(e) => eprintln!("Failed to load account {}: {}", summary.id, e),
        }
    }
    
    Ok(accounts)
}

/// 添加或更新账号摘要到索引
pub fn upsert_account_summary(
    app_handle: &tauri::AppHandle,
    account: &Account,
) -> Result<(), String> {
    let mut index = load_account_index(app_handle)?;
    
    // 移除旧的摘要（如果存在）
    index.accounts.retain(|s| s.id != account.id);
    
    // 添加新的摘要
    index.accounts.push(AccountSummary {
        id: account.id.clone(),
        email: account.email.clone(),
        name: account.name.clone(),
        created_at: account.created_at,
        last_used: account.last_used,
    });
    
    save_account_index(app_handle, &index)
}

