use crate::AppState;
use crate::data::storage::common::{
    GenericPostgreSQLStorage, SQLiteDualStorage,
};
use crate::data::bookmark::{Bookmark, BookmarkLocalStorage, BookmarkMapper};
use std::sync::Arc;
use tauri::State;

/// 书签 PostgreSQL 存储类型别名
pub type BookmarkPostgreSQLStorage = GenericPostgreSQLStorage<Bookmark, BookmarkMapper>;

/// 书签双层存储类型别名（原生 SQLite 本地 + PostgreSQL 远端）
pub type BookmarkDualStorage = SQLiteDualStorage<Bookmark, BookmarkMapper>;

pub async fn initialize_bookmark_storage_manager(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let local_storage = Arc::new(BookmarkLocalStorage::new(app)?);

    let postgres_storage = {
        let db_manager_guard = state.database_manager.lock().unwrap();
        if let Some(db_manager) = db_manager_guard.as_ref() {
            Some(Arc::new(BookmarkPostgreSQLStorage::new(
                db_manager.clone(),
            )))
        } else {
            None
        }
    };

    let dual_storage = Arc::new(BookmarkDualStorage::new(
        local_storage,
        postgres_storage,
    ));

    *state.bookmark_storage_manager.lock().unwrap() = Some(dual_storage);

    Ok(())
}
