use super::traits::{TokenStorage, TokenData, convert_legacy_token, convert_to_legacy_format};
use std::path::PathBuf;
use std::fs;
use std::sync::Mutex;
use tauri::Manager;

pub struct LocalFileStorage {
    storage_path: PathBuf,
    // 使用Mutex来确保文件操作的线程安全
    _lock: Mutex<()>,
}

impl LocalFileStorage {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let app_data_dir = app_handle.path().app_data_dir()?;
        fs::create_dir_all(&app_data_dir)?;
        
        let storage_path = app_data_dir.join("tokens.json");
        
        Ok(Self {
            storage_path,
            _lock: Mutex::new(()),
        })
    }

    pub fn new_with_path(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            _lock: Mutex::new(()),
        }
    }

    async fn read_file_content(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self._lock.lock().unwrap();
        
        if !self.storage_path.exists() {
            return Ok("[]".to_string());
        }

        let content = fs::read_to_string(&self.storage_path)?;
        if content.trim().is_empty() {
            return Ok("[]".to_string());
        }

        Ok(content)
    }

    async fn write_file_content(&self, content: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _guard = self._lock.lock().unwrap();

        // 确保父目录存在
        if let Some(parent) = self.storage_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = self.storage_path.with_extension("tmp");

        // 验证JSON格式
        serde_json::from_str::<serde_json::Value>(content)?;

        // 原子性写入
        fs::write(&temp_path, content)?;

        // 尝试重命名,如果失败则清理临时文件并重试
        match fs::rename(&temp_path, &self.storage_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                // 清理临时文件
                let _ = fs::remove_file(&temp_path);

                // 检查临时文件是否仍然存在
                if !temp_path.exists() {
                    return Err(format!("Failed to rename temp file (temp file was removed): {}", e).into());
                }

                // 检查父目录是否仍然存在
                if let Some(parent) = self.storage_path.parent() {
                    if !parent.exists() {
                        return Err(format!("Failed to rename temp file (parent directory disappeared): {}", e).into());
                    }
                }

                // 在Windows上,如果目标文件被占用,尝试先删除再重命名
                #[cfg(target_os = "windows")]
                {
                    if self.storage_path.exists() {
                        if let Err(remove_err) = fs::remove_file(&self.storage_path) {
                            let _ = fs::remove_file(&temp_path);
                            return Err(format!("Failed to remove existing file before rename: {}", remove_err).into());
                        }
                    }

                    // 重新尝试重命名
                    match fs::rename(&temp_path, &self.storage_path) {
                        Ok(_) => return Ok(()),
                        Err(retry_err) => {
                            let _ = fs::remove_file(&temp_path);
                            return Err(format!("Failed to rename temp file after retry: {}", retry_err).into());
                        }
                    }
                }

                #[cfg(not(target_os = "windows"))]
                {
                    Err(format!("Failed to rename temp file: {}", e).into())
                }
            }
        }
    }

    async fn parse_tokens_from_content(&self, content: &str) -> Result<Vec<TokenData>, Box<dyn std::error::Error + Send + Sync>> {
        let json_value: serde_json::Value = serde_json::from_str(content)?;
        let mut tokens = Vec::new();

        match json_value {
            serde_json::Value::Array(array) => {
                for item in array {
                    match convert_legacy_token(&item) {
                        Ok(token) => tokens.push(token),
                        Err(e) => {
                            eprintln!("Failed to convert token: {}", e);
                            continue;
                        }
                    }
                }
            }
            serde_json::Value::Object(ref obj) => {
                // 检查是否是旧格式 {tokens: [...]}
                if let Some(tokens_array) = obj.get("tokens") {
                    if let serde_json::Value::Array(array) = tokens_array {
                        for item in array {
                            match convert_legacy_token(item) {
                                Ok(token) => tokens.push(token),
                                Err(e) => {
                                    eprintln!("Failed to convert token: {}", e);
                                    continue;
                                }
                            }
                        }
                    }
                } else {
                    // 单个对象格式
                    match convert_legacy_token(&json_value) {
                        Ok(token) => tokens.push(token),
                        Err(e) => eprintln!("Failed to convert single token: {}", e),
                    }
                }
            }
            _ => {
                // 其他格式，返回空数组
            }
        }

        Ok(tokens)
    }
}

#[async_trait::async_trait]
impl TokenStorage for LocalFileStorage {
    async fn save_token(&self, token: &TokenData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tokens = self.load_tokens().await?;
        
        // 检查是否已存在相同ID的token
        if let Some(existing_index) = tokens.iter().position(|t| t.id == token.id) {
            tokens[existing_index] = token.clone();
        } else {
            tokens.push(token.clone());
        }

        // 转换为旧格式并保存
        let legacy_tokens: Vec<serde_json::Value> = tokens.iter()
            .map(convert_to_legacy_format)
            .collect();

        let json_content = serde_json::to_string_pretty(&legacy_tokens)?;
        self.write_file_content(&json_content).await?;

        Ok(())
    }

    async fn load_tokens(&self) -> Result<Vec<TokenData>, Box<dyn std::error::Error + Send + Sync>> {
        let content = self.read_file_content().await?;
        self.parse_tokens_from_content(&content).await
    }

    async fn update_token(&self, token: &TokenData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut updated_token = token.clone();
        updated_token.update_timestamp();
        self.save_token(&updated_token).await
    }

    async fn delete_token(&self, token_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let mut tokens = self.load_tokens().await?;
        let initial_len = tokens.len();
        
        tokens.retain(|t| t.id != token_id);
        
        if tokens.len() < initial_len {
            // 转换为旧格式并保存
            let legacy_tokens: Vec<serde_json::Value> = tokens.iter()
                .map(convert_to_legacy_format)
                .collect();

            let json_content = serde_json::to_string_pretty(&legacy_tokens)?;
            self.write_file_content(&json_content).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn get_token(&self, token_id: &str) -> Result<Option<TokenData>, Box<dyn std::error::Error + Send + Sync>> {
        let tokens = self.load_tokens().await?;
        Ok(tokens.into_iter().find(|t| t.id == token_id))
    }

    async fn clear_all_tokens(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.write_file_content("[]").await
    }

    fn storage_type(&self) -> &'static str {
        "local_file"
    }

    async fn is_available(&self) -> bool {
        // 检查文件是否可写
        if let Some(parent) = self.storage_path.parent() {
            parent.exists() || fs::create_dir_all(parent).is_ok()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_local_storage_operations() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("test_tokens.json");
        let storage = LocalFileStorage::new_with_path(storage_path);

        // 测试保存token
        let token = TokenData::new(
            "test_id".to_string(),
            "https://example.com".to_string(),
            "test_token".to_string(),
            Some("https://portal.example.com".to_string()),
            Some("test note".to_string()),
        );

        assert!(storage.save_token(&token).await.is_ok());

        // 测试加载tokens
        let loaded_tokens = storage.load_tokens().await.unwrap();
        assert_eq!(loaded_tokens.len(), 1);
        assert_eq!(loaded_tokens[0].id, "test_id");

        // 测试获取单个token
        let retrieved_token = storage.get_token("test_id").await.unwrap();
        assert!(retrieved_token.is_some());
        assert_eq!(retrieved_token.unwrap().id, "test_id");

        // 测试删除token
        let deleted = storage.delete_token("test_id").await.unwrap();
        assert!(deleted);

        let tokens_after_delete = storage.load_tokens().await.unwrap();
        assert_eq!(tokens_after_delete.len(), 0);
    }

    #[tokio::test]
    async fn test_storage_availability() {
        let temp_dir = tempdir().unwrap();
        let storage_path = temp_dir.path().join("test_tokens.json");
        let storage = LocalFileStorage::new_with_path(storage_path);

        assert!(storage.is_available().await);
        assert_eq!(storage.storage_type(), "local_file");
    }
}
