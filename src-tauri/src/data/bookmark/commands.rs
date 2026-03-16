use crate::AppState;
use crate::data::storage::common::{
    AccountStorage, AccountSyncManager as CommonAccountSyncManager, AccountSyncStatus,
    ClientAccountSyncRequest, ServerAccountSyncResponse,
};
use crate::data::bookmark::Bookmark;
use crate::data::bookmark::storage::{
    BookmarkDualStorage, initialize_bookmark_storage_manager,
};
use crate::data::bookmark::BookmarkLocalStorage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

/// 书签列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkListResponse {
    pub bookmarks: Vec<Bookmark>,
}

async fn get_bookmark_storage_manager(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> Result<Arc<BookmarkDualStorage>, String> {
    if let Some(manager) = state.bookmark_storage_manager.lock().unwrap().clone() {
        return Ok(manager);
    }

    initialize_bookmark_storage_manager(app, state)
        .await
        .map_err(|e| format!("Failed to initialize bookmark storage manager: {}", e))?;

    state
        .bookmark_storage_manager
        .lock()
        .unwrap()
        .clone()
        .ok_or("Bookmark storage manager not initialized".to_string())
}

/// 列出所有书签
#[tauri::command]
pub async fn bookmark_list(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<BookmarkListResponse, String> {
    let storage_manager = get_bookmark_storage_manager(&app, &state).await?;

    let bookmarks = storage_manager
        .load_accounts()
        .await
        .map_err(|e| format!("Failed to load bookmarks: {}", e))?;

    // 过滤掉已删除的
    let active_bookmarks: Vec<Bookmark> =
        bookmarks.into_iter().filter(|b| !b.deleted).collect();

    Ok(BookmarkListResponse {
        bookmarks: active_bookmarks,
    })
}

/// 直接从本地 SQLite 加载书签（不依赖 storage manager 初始化）
#[tauri::command]
pub async fn bookmark_load_local(
    app: tauri::AppHandle,
) -> Result<BookmarkListResponse, String> {
    let local_storage = BookmarkLocalStorage::new(&app)
        .map_err(|e| format!("Failed to create Bookmark local storage: {}", e))?;

    let bookmarks = local_storage
        .load_accounts()
        .await
        .map_err(|e| format!("Failed to load Bookmark local accounts: {}", e))?;

    let active_bookmarks: Vec<Bookmark> =
        bookmarks.into_iter().filter(|b| !b.deleted).collect();

    Ok(BookmarkListResponse {
        bookmarks: active_bookmarks,
    })
}

/// 添加书签
#[tauri::command]
pub async fn bookmark_add(
    bookmark: Bookmark,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Bookmark, String> {
    let storage_manager = get_bookmark_storage_manager(&app, &state).await?;

    storage_manager
        .save_account(&bookmark)
        .await
        .map_err(|e| format!("Failed to save bookmark: {}", e))?;

    Ok(bookmark)
}

/// 更新书签
#[tauri::command]
pub async fn bookmark_update(
    bookmark: Bookmark,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Bookmark, String> {
    let storage_manager = get_bookmark_storage_manager(&app, &state).await?;

    storage_manager
        .update_account(&bookmark)
        .await
        .map_err(|e| format!("Failed to update bookmark: {}", e))?;

    Ok(bookmark)
}

/// 删除书签
#[tauri::command]
pub async fn bookmark_delete(id: String, app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let storage_manager = get_bookmark_storage_manager(&app, &state).await?;

    storage_manager
        .delete_account(&id)
        .await
        .map_err(|e| format!("Failed to delete bookmark: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn bookmark_sync_accounts(
    req_json: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<ServerAccountSyncResponse<Bookmark>, String> {
    let storage_manager = get_bookmark_storage_manager(&app, &state).await?;

    let req: ClientAccountSyncRequest<Bookmark> = serde_json::from_str(&req_json)
        .map_err(|e| format!("Failed to parse sync request: {}", e))?;

    let upserts_len = req.upserts.len();
    let deletions_len = req.deletions.len();
    let last_version = req.last_version;

    match storage_manager.sync_accounts(req).await {
        Ok(res) => Ok(res),
        Err(e) => {
            println!(
                "Bookmark sync_accounts failed (last_version={}, upserts={}, deletions={}): {}",
                last_version, upserts_len, deletions_len, e
            );
            Err(format!("Sync failed: {}", e))
        }
    }
}

#[tauri::command]
pub async fn bookmark_sync_to_database(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = get_bookmark_storage_manager(&app, &state).await?;

    storage_manager
        .sync_local_to_remote()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn bookmark_sync_from_database(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = get_bookmark_storage_manager(&app, &state).await?;

    storage_manager
        .sync_remote_to_local()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn bookmark_bidirectional_sync(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = get_bookmark_storage_manager(&app, &state).await?;

    storage_manager
        .bidirectional_sync()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}
