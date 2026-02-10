use crate::AppState;
use crate::data::storage::common::{
    GenericDualStorage, GenericLocalStorage, GenericPostgreSQLStorage,
};
use crate::data::subscription::{Subscription, SubscriptionMapper};
use std::sync::Arc;
use tauri::State;

/// 订阅本地存储类型别名
pub type SubscriptionLocalStorage = GenericLocalStorage<Subscription>;

/// 订阅 PostgreSQL 存储类型别名
pub type SubscriptionPostgreSQLStorage = GenericPostgreSQLStorage<Subscription, SubscriptionMapper>;

/// 订阅双层存储类型别名
pub type SubscriptionDualStorage = GenericDualStorage<Subscription, SubscriptionMapper>;

pub async fn initialize_subscription_storage_manager(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let local_storage = Arc::new(SubscriptionLocalStorage::new(app)?);

    let postgres_storage = {
        let db_manager_guard = state.database_manager.lock().unwrap();
        if let Some(db_manager) = db_manager_guard.as_ref() {
            Some(Arc::new(SubscriptionPostgreSQLStorage::new(
                db_manager.clone(),
            )))
        } else {
            None
        }
    };

    let dual_storage = Arc::new(SubscriptionDualStorage::new(
        local_storage,
        postgres_storage,
        false,
    ));

    *state.subscription_storage_manager.lock().unwrap() = Some(dual_storage);

    Ok(())
}
