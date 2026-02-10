use crate::AppState;
use crate::data::storage::common::{
    AccountStorage, AccountSyncManager as CommonAccountSyncManager, AccountSyncStatus,
    ClientAccountSyncRequest, ServerAccountSyncResponse,
};
use crate::data::subscription::Subscription;
use crate::data::subscription::storage::{
    SubscriptionDualStorage, SubscriptionLocalStorage, initialize_subscription_storage_manager,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

/// 订阅列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionListResponse {
    pub subscriptions: Vec<Subscription>,
}

async fn get_subscription_storage_manager(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> Result<Arc<SubscriptionDualStorage>, String> {
    if let Some(manager) = state.subscription_storage_manager.lock().unwrap().clone() {
        return Ok(manager);
    }

    initialize_subscription_storage_manager(app, state)
        .await
        .map_err(|e| format!("Failed to initialize subscription storage manager: {}", e))?;

    state
        .subscription_storage_manager
        .lock()
        .unwrap()
        .clone()
        .ok_or("Subscription storage manager not initialized".to_string())
}

/// 列出所有订阅
#[tauri::command]
pub async fn subscription_list(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<SubscriptionListResponse, String> {
    let storage_manager = get_subscription_storage_manager(&app, &state).await?;

    let subscriptions = storage_manager
        .load_accounts()
        .await
        .map_err(|e| format!("Failed to load subscriptions: {}", e))?;

    // 过滤掉已删除的
    let active_subscriptions: Vec<Subscription> =
        subscriptions.into_iter().filter(|s| !s.deleted).collect();

    Ok(SubscriptionListResponse {
        subscriptions: active_subscriptions,
    })
}

/// 直接从本地文件加载订阅（不依赖 storage manager 初始化）
#[tauri::command]
pub async fn subscription_load_local(
    app: tauri::AppHandle,
) -> Result<SubscriptionListResponse, String> {
    let local_storage = SubscriptionLocalStorage::new(&app)
        .map_err(|e| format!("Failed to create Subscription local storage: {}", e))?;

    let subscriptions = local_storage
        .load_accounts()
        .await
        .map_err(|e| format!("Failed to load Subscription local accounts: {}", e))?;

    let active_subscriptions: Vec<Subscription> =
        subscriptions.into_iter().filter(|s| !s.deleted).collect();

    Ok(SubscriptionListResponse {
        subscriptions: active_subscriptions,
    })
}

/// 添加订阅
#[tauri::command]
pub async fn subscription_add(
    subscription: Subscription,
    state: State<'_, AppState>,
) -> Result<Subscription, String> {
    let storage_manager = {
        let guard = state.subscription_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Subscription storage manager not initialized")?
    };

    storage_manager
        .save_account(&subscription)
        .await
        .map_err(|e| format!("Failed to save subscription: {}", e))?;

    Ok(subscription)
}

/// 更新订阅
#[tauri::command]
pub async fn subscription_update(
    subscription: Subscription,
    state: State<'_, AppState>,
) -> Result<Subscription, String> {
    let storage_manager = {
        let guard = state.subscription_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Subscription storage manager not initialized")?
    };

    storage_manager
        .update_account(&subscription)
        .await
        .map_err(|e| format!("Failed to update subscription: {}", e))?;

    Ok(subscription)
}

/// 删除订阅
#[tauri::command]
pub async fn subscription_delete(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let storage_manager = {
        let guard = state.subscription_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Subscription storage manager not initialized")?
    };

    storage_manager
        .delete_account(&id)
        .await
        .map_err(|e| format!("Failed to delete subscription: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn subscription_sync_accounts(
    req_json: String,
    state: State<'_, AppState>,
) -> Result<ServerAccountSyncResponse<Subscription>, String> {
    let storage_manager = {
        let guard = state.subscription_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Subscription storage manager not initialized")?
    };

    let req: ClientAccountSyncRequest<Subscription> = serde_json::from_str(&req_json)
        .map_err(|e| format!("Failed to parse sync request: {}", e))?;

    let upserts_len = req.upserts.len();
    let deletions_len = req.deletions.len();
    let last_version = req.last_version;

    match storage_manager.sync_accounts(req).await {
        Ok(res) => Ok(res),
        Err(e) => {
            println!(
                "Subscription sync_accounts failed (last_version={}, upserts={}, deletions={}): {}",
                last_version, upserts_len, deletions_len, e
            );
            Err(format!("Sync failed: {}", e))
        }
    }
}

#[tauri::command]
pub async fn subscription_sync_to_database(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.subscription_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Subscription storage manager not initialized")?
    };

    storage_manager
        .sync_local_to_remote()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn subscription_sync_from_database(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.subscription_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Subscription storage manager not initialized")?
    };

    storage_manager
        .sync_remote_to_local()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}

#[tauri::command]
pub async fn subscription_bidirectional_sync(
    state: State<'_, AppState>,
) -> Result<AccountSyncStatus, String> {
    let storage_manager = {
        let guard = state.subscription_storage_manager.lock().unwrap();
        guard
            .clone()
            .ok_or("Subscription storage manager not initialized")?
    };

    storage_manager
        .bidirectional_sync()
        .await
        .map_err(|e| format!("Sync failed: {}", e))
}
