use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::AppHandle;
use tauri::Manager;
use tokio::sync::Mutex;
use tokio::time::Duration;

use crate::core::telegram::{TelegramConfigManager, send_telegram_message};
use crate::data::storage::common::traits::AccountStorage;
use crate::data::subscription::models::Subscription;
use crate::data::subscription::storage::SubscriptionLocalStorage;

/// 通知记录 - 记录已发送的通知，避免重复
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    /// 订阅 ID
    pub subscription_id: String,
    /// 提前天数 (15, 7, 3)
    pub days_before: i32,
    /// 通知发送时间戳
    pub notified_at: i64,
}

/// 通知记录存储
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationRecords {
    /// 记录列表，key 为 "{subscription_id}_{days_before}"
    pub records: HashMap<String, NotificationRecord>,
}

impl NotificationRecords {
    /// 生成记录键
    fn make_key(subscription_id: &str, days_before: i32) -> String {
        format!("{}_{}", subscription_id, days_before)
    }

    /// 检查是否已通知
    pub fn has_notified(&self, subscription_id: &str, days_before: i32) -> bool {
        let key = Self::make_key(subscription_id, days_before);
        self.records.contains_key(&key)
    }

    /// 添加通知记录
    pub fn add_record(&mut self, subscription_id: &str, days_before: i32) {
        let key = Self::make_key(subscription_id, days_before);
        self.records.insert(
            key,
            NotificationRecord {
                subscription_id: subscription_id.to_string(),
                days_before,
                notified_at: chrono::Utc::now().timestamp(),
            },
        );
    }

    /// 清理过期记录 (30 天前的记录)
    pub fn cleanup_old_records(&mut self) {
        let threshold = chrono::Utc::now().timestamp() - 30 * 24 * 60 * 60;
        self.records
            .retain(|_, record| record.notified_at > threshold);
    }
}

/// 通知记录管理器
pub struct NotificationRecordManager {
    record_path: PathBuf,
}

impl NotificationRecordManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {}", e))?;

        fs::create_dir_all(&app_data_dir)
            .map_err(|e| format!("Failed to create app data dir: {}", e))?;

        let record_path = app_data_dir.join("notification_records.json");

        Ok(Self { record_path })
    }

    /// 加载记录
    pub fn load_records(&self) -> Result<NotificationRecords, String> {
        if !self.record_path.exists() {
            return Ok(NotificationRecords::default());
        }

        let content = fs::read_to_string(&self.record_path)
            .map_err(|e| format!("Failed to read records: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse records: {}", e))
    }

    /// 保存记录
    pub fn save_records(&self, records: &NotificationRecords) -> Result<(), String> {
        let content = serde_json::to_string_pretty(records)
            .map_err(|e| format!("Failed to serialize records: {}", e))?;

        fs::write(&self.record_path, content).map_err(|e| format!("Failed to write records: {}", e))
    }
}

/// 计算订阅剩余天数
fn calculate_days_left(expiry_date: &str) -> Option<i32> {
    let expiry = chrono::NaiveDate::parse_from_str(expiry_date, "%Y-%m-%d").ok()?;
    let today = chrono::Utc::now().date_naive();
    let duration = expiry.signed_duration_since(today);
    Some(duration.num_days() as i32)
}

/// 检查订阅到期并发送通知
pub async fn check_and_notify_expiring_subscriptions(app_handle: &AppHandle) -> Result<(), String> {
    // 加载 Telegram 配置
    let telegram_manager = TelegramConfigManager::new(app_handle)?;
    let telegram_config = telegram_manager.load_config()?;

    // 如果未启用或配置不完整，跳过
    if !telegram_config.enabled
        || telegram_config.bot_token.is_empty()
        || telegram_config.chat_id.is_empty()
    {
        return Ok(());
    }

    // 加载订阅列表
    let storage = SubscriptionLocalStorage::new(app_handle)
        .map_err(|e| format!("Failed to create storage: {}", e))?;
    let subscriptions = storage
        .load_accounts()
        .await
        .map_err(|e| format!("Failed to load subscriptions: {}", e))?;

    // 加载通知记录
    let record_manager = NotificationRecordManager::new(app_handle)?;
    let mut records = record_manager.load_records()?;

    // 清理旧记录
    records.cleanup_old_records();

    // 收集需要通知的订阅
    let mut notifications: HashMap<i32, Vec<(String, i32, String)>> = HashMap::new(); // days_before -> [(name, days_left, expiry_date)]

    for sub in &subscriptions {
        if sub.deleted {
            continue;
        }

        if let Some(expiry_date) = &sub.expiry_date {
            if let Some(days_left) = calculate_days_left(expiry_date) {
                // 检查是否在提醒天数范围内
                for &notify_day in &telegram_config.notify_days {
                    // 检查是否在该提醒阶段（允许 1 天的误差范围）
                    if days_left <= notify_day && days_left > notify_day - 1 {
                        // 检查是否已通知过
                        if !records.has_notified(&sub.id, notify_day) {
                            notifications.entry(notify_day).or_default().push((
                                sub.website.clone(),
                                days_left,
                                expiry_date.to_string(),
                            ));

                            // 标记为已通知
                            records.add_record(&sub.id, notify_day);
                        }
                    }
                }
            }
        }
    }

    // 发送通知
    if !notifications.is_empty() {
        let mut message = String::from("📅 <b>订阅到期提醒</b>\n\n🔔 以下订阅即将到期：\n\n");

        // 按剩余天数排序
        let mut all_items: Vec<(String, i32, String)> =
            notifications.into_values().flatten().collect();
        all_items.sort_by_key(|(_, days, _)| *days);

        for (name, days_left, expiry_date) in all_items {
            let days_text = if days_left <= 0 {
                "已到期".to_string()
            } else if days_left == 1 {
                "明天到期".to_string()
            } else {
                format!("{} 天后到期", days_left)
            };
            message.push_str(&format!(
                "• <b>{}</b> - {} ({})\n",
                name, days_text, expiry_date
            ));
        }

        message.push_str("\n请及时处理续费事宜。");

        // 发送消息
        if let Err(e) = send_telegram_message(
            &telegram_config.bot_token,
            &telegram_config.chat_id,
            &message,
        )
        .await
        {
            eprintln!("Failed to send Telegram notification: {}", e);
        }
    }

    // 保存通知记录
    record_manager.save_records(&records)?;

    Ok(())
}

/// 监控状态
pub struct MonitorState {
    pub is_running: Arc<Mutex<bool>>,
}

impl MonitorState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
        }
    }
}

impl Default for MonitorState {
    fn default() -> Self {
        Self::new()
    }
}

/// 启动订阅监控定时任务
pub fn start_subscription_monitor(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // 启动后延迟 10 秒再进行首次检查
        tokio::time::sleep(Duration::from_secs(10)).await;

        // 首次检查
        if let Err(e) = check_and_notify_expiring_subscriptions(&app_handle).await {
            eprintln!("Subscription monitor check failed: {}", e);
        }

        loop {
            // 每次循环读取配置，支持动态修改检查间隔
            let interval_hours = match TelegramConfigManager::new(&app_handle) {
                Ok(manager) => match manager.load_config() {
                    Ok(config) => config.check_interval_hours.max(1), // 最少 1 小时
                    Err(_) => 6,                                      // 默认 6 小时
                },
                Err(_) => 6,
            };

            // 等待指定的间隔时间
            tokio::time::sleep(Duration::from_secs(interval_hours as u64 * 60 * 60)).await;

            if let Err(e) = check_and_notify_expiring_subscriptions(&app_handle).await {
                eprintln!("Subscription monitor check failed: {}", e);
            }
        }
    });
}

// ============ Tauri Commands ============

/// 手动触发订阅到期检查
#[tauri::command]
pub async fn check_subscriptions_expiry(app: AppHandle) -> Result<(), String> {
    check_and_notify_expiring_subscriptions(&app).await
}

/// 获取即将到期的订阅列表（供前端展示）
#[tauri::command]
pub async fn get_expiring_subscriptions(
    app: AppHandle,
    days: i32,
) -> Result<Vec<Subscription>, String> {
    let storage = SubscriptionLocalStorage::new(&app)
        .map_err(|e| format!("Failed to create storage: {}", e))?;
    let subscriptions = storage
        .load_accounts()
        .await
        .map_err(|e| format!("Failed to load subscriptions: {}", e))?;

    let expiring: Vec<Subscription> = subscriptions
        .into_iter()
        .filter(|sub| {
            if sub.deleted {
                return false;
            }
            if let Some(expiry_date) = &sub.expiry_date {
                if let Some(days_left) = calculate_days_left(expiry_date) {
                    return days_left >= 0 && days_left <= days;
                }
            }
            false
        })
        .collect();

    Ok(expiring)
}
