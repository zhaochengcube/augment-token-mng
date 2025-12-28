use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use tauri::Manager;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::{rngs::OsRng, RngCore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

impl Default for SslMode {
    fn default() -> Self {
        SslMode::Prefer
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub password: String,
    pub password_encrypted: String,
    pub ssl_mode: SslMode,
    pub enabled: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "augment_tokens".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            password_encrypted: String::new(),
            ssl_mode: SslMode::default(),
            enabled: false,
        }
    }
}

impl DatabaseConfig {
    pub fn new(host: String, port: u16, database: String, username: String, password: String) -> Self {
        let mut config = Self {
            host,
            port,
            database,
            username,
            password: password.clone(),
            password_encrypted: String::new(),
            ssl_mode: SslMode::default(),
            enabled: true,
        };

        if let Ok(encrypted) = encrypt_password(&password) {
            config.password_encrypted = encrypted;
        }

        config
    }

    pub fn new_with_ssl(host: String, port: u16, database: String, username: String, password: String, ssl_mode: SslMode) -> Self {
        let mut config = Self {
            host,
            port,
            database,
            username,
            password: password.clone(),
            password_encrypted: String::new(),
            ssl_mode,
            enabled: true,
        };

        if let Ok(encrypted) = encrypt_password(&password) {
            config.password_encrypted = encrypted;
        }

        config
    }

    pub fn connection_string(&self) -> String {
        let ssl_mode_str = match self.ssl_mode {
            SslMode::Disable => "disable",
            SslMode::Prefer => "prefer",
            SslMode::Require => "require",
        };

        format!(
            "host={} port={} dbname={} user={} password={} sslmode={}",
            self.host, self.port, self.database, self.username, self.password, ssl_mode_str
        )
    }

    pub fn decrypt_password(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.password_encrypted.is_empty() {
            self.password = decrypt_password(&self.password_encrypted)?;
        }
        Ok(())
    }

    pub fn test_connection(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 这里实现连接测试逻辑
        // 暂时返回Ok，后续在connection.rs中实现
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfigManager {
    config_path: PathBuf,
}

impl DatabaseConfigManager {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let app_data_dir = app_handle.path().app_data_dir()?;
        fs::create_dir_all(&app_data_dir)?;
        
        let config_path = app_data_dir.join("database_config.json");
        
        Ok(Self { config_path })
    }

    pub fn load_config(&self) -> Result<DatabaseConfig, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config_path.exists() {
            return Ok(DatabaseConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let mut config: DatabaseConfig = serde_json::from_str(&content)?;
        config.decrypt_password()?;
        
        Ok(config)
    }

    pub fn save_config(&self, config: &DatabaseConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    pub fn delete_config(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path)?;
        }
        Ok(())
    }
}

// 密码加密/解密功能
fn get_encryption_key() -> [u8; 32] {
    // 在实际应用中，这应该是一个更安全的密钥派生方法
    // 这里使用固定密钥仅用于演示
    let mut key = [0u8; 32];
    key[..16].copy_from_slice(b"augment_token_mg");
    key[16..].copy_from_slice(b"r_encryption_key");
    key
}

fn encrypt_password(password: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let key = Aes256Gcm::new_from_slice(&get_encryption_key())
        .map_err(|e| format!("Failed to create encryption key: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = key.encrypt(nonce, password.as_bytes())
        .map_err(|e| format!("Failed to encrypt password: {}", e))?;

    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);

    Ok(hex::encode(result))
}

fn decrypt_password(encrypted_hex: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let encrypted_data = hex::decode(encrypted_hex)?;

    if encrypted_data.len() < 12 {
        return Err("Invalid encrypted data".into());
    }

    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let key = Aes256Gcm::new_from_slice(&get_encryption_key())
        .map_err(|e| format!("Failed to create decryption key: {}", e))?;
    let plaintext = key.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Failed to decrypt password: {}", e))?;

    Ok(String::from_utf8(plaintext)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_encryption() {
        let password = "test_password_123";
        let encrypted = encrypt_password(password).unwrap();
        let decrypted = decrypt_password(&encrypted).unwrap();
        assert_eq!(password, decrypted);
    }

    #[test]
    fn test_database_config() {
        let config = DatabaseConfig::new(
            "localhost".to_string(),
            5432,
            "test_db".to_string(),
            "user".to_string(),
            "password".to_string(),
        );
        
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "test_db");
        assert_eq!(config.username, "user");
        assert_eq!(config.password, "password");
        assert!(!config.password_encrypted.is_empty());
        assert!(config.enabled);
    }
}
