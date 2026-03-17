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

/// 浏览器书签导入结果
#[derive(Debug, Clone, Serialize)]
pub struct BookmarkImportResult {
    pub total_parsed: usize,
    pub success_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub imported_ids: Vec<String>,
}

/// 从浏览器导出的 HTML 书签文件中解析书签
/// 支持 Netscape Bookmark File Format（Chrome、Firefox、Edge、Safari 通用）
fn parse_browser_bookmark_html(content: &str) -> Vec<Bookmark> {
    use regex::Regex;

    let folder_re = Regex::new(r"(?i)<H3[^>]*>(.*?)</H3>").unwrap();
    let bookmark_re = Regex::new(r#"(?i)<A\s[^>]*HREF="([^"]*)"[^>]*>(.*?)</A>"#).unwrap();
    let add_date_re = Regex::new(r#"(?i)ADD_DATE="(\d+)""#).unwrap();

    let mut bookmarks = Vec::new();
    let mut folder_stack: Vec<String> = Vec::new();
    let mut pending_folder: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // 检测文件夹标题
        if let Some(caps) = folder_re.captures(trimmed) {
            pending_folder = Some(html_unescape(&caps[1]));
        }

        // <DL> 表示进入一个文件夹
        if trimmed.contains("<DL>") || trimmed.contains("<dl>") {
            if let Some(folder) = pending_folder.take() {
                folder_stack.push(folder);
            }
        }

        // </DL> 表示离开一个文件夹
        if trimmed.contains("</DL>") || trimmed.contains("</dl>") {
            folder_stack.pop();
        }

        // 检测书签链接
        if let Some(caps) = bookmark_re.captures(trimmed) {
            let url = html_unescape(&caps[1]);
            let name = html_unescape(&caps[2]);

            // 跳过空 URL 或 javascript: 协议
            if url.is_empty() || url.starts_with("javascript:") {
                continue;
            }

            let add_date = add_date_re
                .captures(trimmed)
                .and_then(|c| c[1].parse::<i64>().ok());

            // 跳过第一层文件夹（书签栏、其他书签等浏览器固定根文件夹）
            let tag = if folder_stack.len() <= 1 {
                None
            } else {
                Some(folder_stack[1..].join("/"))
            };

            let now = chrono::Utc::now().timestamp();
            let created_at = add_date.unwrap_or(now);

            let id = format!("bm_import_{}_{}", now, bookmarks.len());
            let mut bookmark = Bookmark::new(id, name);
            bookmark.url = Some(url);
            bookmark.tag = tag;
            bookmark.created_at = created_at;
            bookmark.updated_at = now;

            bookmarks.push(bookmark);
        }
    }

    bookmarks
}

/// 简单的 HTML 实体反转义
fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

/// 从浏览器导出的书签 HTML 内容导入书签
#[tauri::command]
pub async fn bookmark_import_from_browser(
    content: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<BookmarkImportResult, String> {
    // 1. 解析书签
    let parsed = parse_browser_bookmark_html(&content);
    let total_parsed = parsed.len();

    if total_parsed == 0 {
        return Err("No bookmarks found in the file".to_string());
    }

    // 2. 获取现有书签用于去重
    let storage_manager = get_bookmark_storage_manager(&app, &state).await?;
    let existing = storage_manager
        .load_accounts()
        .await
        .map_err(|e| format!("Failed to load existing bookmarks: {}", e))?;

    let existing_urls: std::collections::HashSet<String> = existing
        .iter()
        .filter_map(|b| b.url.clone())
        .map(|u| u.to_lowercase())
        .collect();

    // 3. 保存书签（跳过已存在的 URL）
    let mut success_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;
    let mut imported_ids: Vec<String> = Vec::new();

    for bookmark in parsed {
        // URL 去重
        if let Some(ref url) = bookmark.url {
            if existing_urls.contains(&url.to_lowercase()) {
                skipped_count += 1;
                continue;
            }
        }

        let bookmark_id = bookmark.id.clone();
        match storage_manager.save_account(&bookmark).await {
            Ok(_) => {
                success_count += 1;
                imported_ids.push(bookmark_id);
            }
            Err(_) => failed_count += 1,
        }
    }

    Ok(BookmarkImportResult {
        total_parsed,
        success_count,
        skipped_count,
        failed_count,
        imported_ids,
    })
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
