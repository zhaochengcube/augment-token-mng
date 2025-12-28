use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub category: String, // "temp" or "service"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Bookmark {
    pub fn new(name: String, url: String, description: Option<String>, category: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            url,
            description,
            category,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, name: String, url: String, description: Option<String>) {
        self.name = name;
        self.url = url;
        self.description = description;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BookmarkStorage {
    pub bookmarks: Vec<Bookmark>,
}

impl BookmarkStorage {
    pub fn new() -> Self {
        Self {
            bookmarks: Vec::new(),
        }
    }

    pub fn add_bookmark(&mut self, name: String, url: String, description: Option<String>, category: String) -> String {
        let bookmark = Bookmark::new(name, url, description, category);
        let id = bookmark.id.clone();
        self.bookmarks.push(bookmark);
        id
    }

    pub fn update_bookmark(&mut self, id: &str, name: String, url: String, description: Option<String>) -> bool {
        if let Some(bookmark) = self.bookmarks.iter_mut().find(|b| b.id == id) {
            bookmark.update(name, url, description);
            true
        } else {
            false
        }
    }

    pub fn remove_bookmark(&mut self, id: &str) -> bool {
        let initial_len = self.bookmarks.len();
        self.bookmarks.retain(|bookmark| bookmark.id != id);
        self.bookmarks.len() < initial_len
    }

    pub fn get_bookmarks_by_category(&self, category: &str) -> Vec<Bookmark> {
        self.bookmarks
            .iter()
            .filter(|bookmark| bookmark.category == category)
            .cloned()
            .collect()
    }


}

#[derive(Clone)]
pub struct BookmarkManager {
    storage_path: PathBuf,
}

impl BookmarkManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;
        
        // Create app data directory if it doesn't exist
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data directory: {}", e))?;
        
        let storage_path = app_data_dir.join("bookmarks.json");
        
        Ok(Self { storage_path })
    }

    pub fn load_bookmarks(&self) -> Result<BookmarkStorage, Box<dyn std::error::Error>> {
        if !self.storage_path.exists() {
            return Ok(BookmarkStorage::new());
        }

        let content = fs::read_to_string(&self.storage_path)
            .map_err(|e| format!("Failed to read bookmarks file: {}", e))?;
        
        let storage: BookmarkStorage = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse bookmarks file: {}", e))?;
        
        Ok(storage)
    }

    pub fn save_bookmarks(&self, storage: &BookmarkStorage) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(storage)
            .map_err(|e| format!("Failed to serialize bookmarks: {}", e))?;
        
        fs::write(&self.storage_path, content)
            .map_err(|e| format!("Failed to write bookmarks file: {}", e))?;
        
        Ok(())
    }

    pub fn add_bookmark(&self, name: String, url: String, description: Option<String>, category: String) -> Result<String, Box<dyn std::error::Error>> {
        let mut storage = self.load_bookmarks()?;
        let id = storage.add_bookmark(name, url, description, category);
        self.save_bookmarks(&storage)?;
        Ok(id)
    }

    pub fn update_bookmark(&self, id: &str, name: String, url: String, description: Option<String>) -> Result<bool, Box<dyn std::error::Error>> {
        let mut storage = self.load_bookmarks()?;
        let updated = storage.update_bookmark(id, name, url, description);
        if updated {
            self.save_bookmarks(&storage)?;
        }
        Ok(updated)
    }

    pub fn remove_bookmark(&self, id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut storage = self.load_bookmarks()?;
        let removed = storage.remove_bookmark(id);
        if removed {
            self.save_bookmarks(&storage)?;
        }
        Ok(removed)
    }

    pub fn get_bookmarks_by_category(&self, category: &str) -> Result<Vec<Bookmark>, Box<dyn std::error::Error>> {
        let storage = self.load_bookmarks()?;
        Ok(storage.get_bookmarks_by_category(category))
    }

    pub fn get_all_bookmarks(&self) -> Result<Vec<Bookmark>, Box<dyn std::error::Error>> {
        let storage = self.load_bookmarks()?;
        Ok(storage.bookmarks)
    }
}

// Bookmark management commands
#[tauri::command]
pub async fn add_bookmark(
    name: String,
    url: String,
    description: Option<String>,
    category: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    let bookmark_manager = BookmarkManager::new(&app)
        .map_err(|e| format!("Failed to initialize bookmark manager: {}", e))?;

    bookmark_manager.add_bookmark(name, url, description, category)
        .map_err(|e| format!("Failed to add bookmark: {}", e))
}

#[tauri::command]
pub async fn update_bookmark(
    id: String,
    name: String,
    url: String,
    description: Option<String>,
    app: tauri::AppHandle,
) -> Result<bool, String> {
    let bookmark_manager = BookmarkManager::new(&app)
        .map_err(|e| format!("Failed to initialize bookmark manager: {}", e))?;

    bookmark_manager.update_bookmark(&id, name, url, description)
        .map_err(|e| format!("Failed to update bookmark: {}", e))
}

#[tauri::command]
pub async fn delete_bookmark(
    id: String,
    app: tauri::AppHandle,
) -> Result<bool, String> {
    let bookmark_manager = BookmarkManager::new(&app)
        .map_err(|e| format!("Failed to initialize bookmark manager: {}", e))?;

    bookmark_manager.remove_bookmark(&id)
        .map_err(|e| format!("Failed to delete bookmark: {}", e))
}

#[tauri::command]
pub async fn get_bookmarks(
    category: String,
    app: tauri::AppHandle,
) -> Result<Vec<Bookmark>, String> {
    let bookmark_manager = BookmarkManager::new(&app)
        .map_err(|e| format!("Failed to initialize bookmark manager: {}", e))?;

    bookmark_manager.get_bookmarks_by_category(&category)
        .map_err(|e| format!("Failed to get bookmarks: {}", e))
}

#[tauri::command]
pub async fn get_all_bookmarks(
    app: tauri::AppHandle,
) -> Result<Vec<Bookmark>, String> {
    let bookmark_manager = BookmarkManager::new(&app)
        .map_err(|e| format!("Failed to initialize bookmark manager: {}", e))?;

    bookmark_manager.get_all_bookmarks()
        .map_err(|e| format!("Failed to get all bookmarks: {}", e))
}
