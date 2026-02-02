use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

/// Telegram 配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    /// Bot Token
    pub bot_token: String,
    /// Chat ID
    pub chat_id: String,
    /// 是否启用
    pub enabled: bool,
    /// 提醒天数列表 (如 [15, 7, 3])
    #[serde(default = "default_notify_days")]
    pub notify_days: Vec<i32>,
    /// 检查间隔（小时）
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u32,
}

fn default_notify_days() -> Vec<i32> {
    vec![15, 7, 3]
}

fn default_check_interval() -> u32 {
    6
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            chat_id: String::new(),
            enabled: false,
            notify_days: default_notify_days(),
            check_interval_hours: default_check_interval(),
        }
    }
}

/// Telegram 配置管理器
pub struct TelegramConfigManager {
    config_path: PathBuf,
}

impl TelegramConfigManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;

        // 确保目录存在
        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;

        let config_path = app_data_dir.join("telegram_config.json");

        Ok(Self { config_path })
    }

    /// 加载配置
    pub fn load_config(&self) -> Result<TelegramConfig, String> {
        if !self.config_path.exists() {
            return Ok(TelegramConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))
    }

    /// 保存配置
    pub fn save_config(&self, config: &TelegramConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&self.config_path, content)
            .map_err(|e| format!("Failed to write config: {}", e))
    }

    /// 删除配置
    pub fn delete_config(&self) -> Result<(), String> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path)
                .map_err(|e| format!("Failed to delete config: {}", e))?;
        }
        Ok(())
    }
}

/// 发送 Telegram 消息
pub async fn send_telegram_message(
    bot_token: &str,
    chat_id: &str,
    message: &str,
) -> Result<(), String> {
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage",
        bot_token
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&serde_json::json!({
            "chat_id": chat_id,
            "text": message,
            "parse_mode": "HTML"
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Telegram API error: {}", error_text))
    }
}

/// 测试 Telegram 连接
pub async fn test_telegram_connection(bot_token: &str, chat_id: &str) -> Result<bool, String> {
    let test_message = "🔔 ATM 连接测试成功！\n\nTelegram 通知已正确配置。";
    send_telegram_message(bot_token, chat_id, test_message).await?;
    Ok(true)
}

// ============ Tauri Commands ============

/// 保存 Telegram 配置
#[tauri::command]
pub fn save_telegram_config(app: AppHandle, config: TelegramConfig) -> Result<(), String> {
    let manager = TelegramConfigManager::new(&app)?;
    manager.save_config(&config)
}

/// 加载 Telegram 配置
#[tauri::command]
pub fn load_telegram_config(app: AppHandle) -> Result<TelegramConfig, String> {
    let manager = TelegramConfigManager::new(&app)?;
    manager.load_config()
}

/// 删除 Telegram 配置
#[tauri::command]
pub fn delete_telegram_config(app: AppHandle) -> Result<(), String> {
    let manager = TelegramConfigManager::new(&app)?;
    manager.delete_config()
}

/// 测试 Telegram 连接
#[tauri::command]
pub async fn test_telegram_connection_cmd(bot_token: String, chat_id: String) -> Result<bool, String> {
    test_telegram_connection(&bot_token, &chat_id).await
}

/// 发送 Telegram 消息
#[tauri::command]
pub async fn send_telegram_message_cmd(
    app: AppHandle,
    message: String,
) -> Result<(), String> {
    let manager = TelegramConfigManager::new(&app)?;
    let config = manager.load_config()?;

    if !config.enabled {
        return Err("Telegram notifications are disabled".to_string());
    }

    if config.bot_token.is_empty() || config.chat_id.is_empty() {
        return Err("Telegram configuration is incomplete".to_string());
    }

    send_telegram_message(&config.bot_token, &config.chat_id, &message).await
}
