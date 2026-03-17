use crate::data::bookmark::{Bookmark, BookmarkLocalStorage};
use crate::data::storage::common::AccountStorage;
use super::client::RaindropClient;
use super::models::*;
use serde_json;
use std::collections::HashMap;
use tauri::Manager;

/// 获取 Raindrop 配置文件路径
fn config_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(dir.join("raindrop_config.json"))
}

/// 读取配置
fn load_config(app: &tauri::AppHandle) -> Result<Option<RaindropConfig>, String> {
    let path = config_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read raindrop config: {}", e))?;
    let config: RaindropConfig =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse raindrop config: {}", e))?;
    Ok(Some(config))
}

/// 保存配置
fn save_config(app: &tauri::AppHandle, config: &RaindropConfig) -> Result<(), String> {
    let path = config_path(app)?;
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize raindrop config: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write raindrop config: {}", e))?;
    Ok(())
}

/// 解析 ISO 8601 日期为 Unix 时间戳
fn parse_iso_date(date_str: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(date_str)
        .map(|dt| dt.timestamp())
        .unwrap_or_else(|_| chrono::Utc::now().timestamp())
}

/// 将 Raindrop 项转换为本地 Bookmark
fn raindrop_to_bookmark(
    item: &RaindropItem,
    collection_map: &HashMap<i64, RaindropCollectionInfo>,
) -> Bookmark {
    let id = format!("rd_{}", item.id);
    let name = item.title.clone().unwrap_or_else(|| item.link.clone());

    let now = chrono::Utc::now().timestamp();
    let created_at = item
        .created
        .as_deref()
        .map(parse_iso_date)
        .unwrap_or(now);
    let updated_at = item
        .last_update
        .as_deref()
        .map(parse_iso_date)
        .unwrap_or(now);

    // 标签：优先使用 Raindrop tags，否则使用集合名称
    let tag = if !item.tags.is_empty() {
        Some(item.tags.join(", "))
    } else {
        item.collection
            .as_ref()
            .and_then(|c| collection_map.get(&c.id))
            .map(|info| info.title.clone())
    };

    // 标签颜色：使用集合颜色
    let tag_color = item
        .collection
        .as_ref()
        .and_then(|c| collection_map.get(&c.id))
        .and_then(|info| info.color.clone());

    let mut bookmark = Bookmark::new(id, name);
    bookmark.url = Some(item.link.clone());
    bookmark.description = item.excerpt.clone();
    bookmark.tag = tag;
    bookmark.tag_color = tag_color;
    bookmark.created_at = created_at;
    bookmark.updated_at = updated_at;
    bookmark
}

// ===================== Tauri Commands =====================

/// 保存 Raindrop Token 配置
#[tauri::command]
pub async fn raindrop_save_config(
    token: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // 先验证 token
    let client = RaindropClient::new(token.clone())?;
    let valid = client.validate_token().await?;
    if !valid {
        return Err("Invalid Raindrop.io token".to_string());
    }

    // 读取已有配置保留 last_sync_at
    let existing = load_config(&app)?;
    let config = RaindropConfig {
        token,
        last_sync_at: existing.and_then(|c| c.last_sync_at),
    };
    save_config(&app, &config)?;
    Ok(())
}

/// 读取 Raindrop 配置（不暴露完整 token）
#[tauri::command]
pub async fn raindrop_load_config(
    app: tauri::AppHandle,
) -> Result<Option<RaindropConfigView>, String> {
    let config = load_config(&app)?;
    Ok(config.map(|c| RaindropConfigView {
        has_token: !c.token.is_empty(),
        token_preview: if c.token.len() > 8 {
            format!("{}...{}", &c.token[..4], &c.token[c.token.len() - 4..])
        } else {
            "****".to_string()
        },
        last_sync_at: c.last_sync_at,
    }))
}

/// 前端可见的配置视图（隐藏完整 token）
#[derive(Debug, Clone, serde::Serialize)]
pub struct RaindropConfigView {
    pub has_token: bool,
    pub token_preview: String,
    pub last_sync_at: Option<String>,
}

/// 删除 Raindrop 配置
#[tauri::command]
pub async fn raindrop_delete_config(
    app: tauri::AppHandle,
) -> Result<(), String> {
    let path = config_path(&app)?;
    if path.exists() {
        std::fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete raindrop config: {}", e))?;
    }
    Ok(())
}

/// 同步 Raindrop 书签到本地
#[tauri::command]
pub async fn raindrop_sync(
    app: tauri::AppHandle,
) -> Result<RaindropSyncResult, String> {
    let config = load_config(&app)?
        .ok_or("Raindrop config not found. Please configure your token first.")?;

    let client = RaindropClient::new(config.token.clone())?;

    // 获取集合信息（用于映射 tag 和 color）
    let collections = client.fetch_collections().await?;
    let collection_map: HashMap<i64, RaindropCollectionInfo> = collections
        .into_iter()
        .map(|c| (c.id, c))
        .collect();

    // 增量同步：使用 last_sync_at 作为起点
    let raindrops = client
        .fetch_all_raindrops(config.last_sync_at.as_deref())
        .await?;

    let total_fetched = raindrops.len();
    let mut created = 0usize;
    let mut updated = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;
    let mut affected_ids: Vec<String> = Vec::new();

    // 初始化本地存储
    let local_storage = BookmarkLocalStorage::new(&app)
        .map_err(|e| format!("Failed to create bookmark local storage: {}", e))?;

    // 加载现有书签用于判断是新增还是更新
    let existing_bookmarks = local_storage
        .load_accounts()
        .await
        .map_err(|e| format!("Failed to load existing bookmarks: {}", e))?;
    let existing_map: HashMap<String, &Bookmark> = existing_bookmarks
        .iter()
        .map(|b| (b.id.clone(), b))
        .collect();

    for item in &raindrops {
        // 跳过已删除的
        if item.removed {
            skipped += 1;
            continue;
        }

        let bookmark = raindrop_to_bookmark(item, &collection_map);
        let bookmark_id = bookmark.id.clone();

        if let Some(local) = existing_map.get(&bookmark_id) {
            // 已存在：比较实际内容是否有变化
            let content_changed = bookmark.name != local.name
                || bookmark.url != local.url
                || bookmark.description != local.description
                || bookmark.tag != local.tag
                || bookmark.tag_color != local.tag_color;

            if content_changed {
                match local_storage.save_account(&bookmark).await {
                    Ok(_) => {
                        updated += 1;
                        affected_ids.push(bookmark_id);
                    }
                    Err(e) => {
                        eprintln!("Failed to update bookmark {}: {}", bookmark_id, e);
                        failed += 1;
                    }
                }
            } else {
                skipped += 1;
            }
        } else {
            // 新书签
            match local_storage.save_account(&bookmark).await {
                Ok(_) => {
                    created += 1;
                    affected_ids.push(bookmark_id);
                }
                Err(e) => {
                    eprintln!("Failed to create bookmark {}: {}", bookmark_id, e);
                    failed += 1;
                }
            }
        }
    }

    // 更新 last_sync_at
    let now = chrono::Utc::now().to_rfc3339();
    let updated_config = RaindropConfig {
        token: config.token,
        last_sync_at: Some(now),
    };
    save_config(&app, &updated_config)?;

    Ok(RaindropSyncResult {
        total_fetched,
        created,
        updated,
        skipped,
        failed,
        affected_ids,
    })
}

/// 全量重新同步（忽略 last_sync_at）
#[tauri::command]
pub async fn raindrop_full_sync(
    app: tauri::AppHandle,
) -> Result<RaindropSyncResult, String> {
    // 先清除 last_sync_at
    let mut config = load_config(&app)?
        .ok_or("Raindrop config not found. Please configure your token first.")?;
    config.last_sync_at = None;
    save_config(&app, &config)?;

    // 然后执行常规同步
    raindrop_sync(app).await
}

/// 验证 Token 是否有效
#[tauri::command]
pub async fn raindrop_validate_token(
    token: String,
) -> Result<bool, String> {
    let client = RaindropClient::new(token)?;
    client.validate_token().await
}
