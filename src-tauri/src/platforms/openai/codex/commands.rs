//! Codex 模块 Tauri Commands
//!
//! 提供给前端调用的 Codex API 管理命令
use std::fs;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use tokio::sync::Mutex as TokioMutex;

use super::logger::RequestLogger;
use super::models::{
    DailyStatsResponse, LogPage, LogQuery, ModelTokenStats, PeriodTokenStats, PoolStrategy,
    RequestLog, TokenStats,
};
use super::pool::{CodexServerConfig, CodexServerStatus};
use crate::AppState;
use crate::platforms::openai::codex::server::CodexServer;
static QUOTA_REFRESH_TASK: std::sync::LazyLock<TokioMutex<Option<tokio::task::JoinHandle<()>>>> =
    std::sync::LazyLock::new(|| TokioMutex::new(None));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAccessConfig {
    pub server_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexRuntimeSettings {
    pub quota_refresh_enabled: bool,
    pub quota_refresh_interval_seconds: u64,
    pub fast_mode_enabled: bool,
}

const CODEX_CONFIG_FILE: &str = "openai_codex_config.json";
const SHARED_API_SERVER_PORT: u16 = 8766;
const MIN_QUOTA_REFRESH_INTERVAL_SECONDS: u64 = 60;
const MAX_QUOTA_REFRESH_INTERVAL_SECONDS: u64 = 24 * 60 * 60;

fn normalize_access_fields(config: &mut CodexServerConfig) {
    config.api_key = config.api_key.as_ref().and_then(|v| {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });
}

fn normalize_server_port(config: &mut CodexServerConfig) {
    config.port = SHARED_API_SERVER_PORT;
}

fn normalize_runtime_fields(config: &mut CodexServerConfig) {
    let defaults = CodexServerConfig::default();

    if config.pool_strategy.trim().is_empty() {
        config.pool_strategy = defaults.pool_strategy;
    }

    if config.quota_refresh_interval_seconds < MIN_QUOTA_REFRESH_INTERVAL_SECONDS {
        config.quota_refresh_interval_seconds = defaults
            .quota_refresh_interval_seconds
            .max(MIN_QUOTA_REFRESH_INTERVAL_SECONDS);
    }
    if config.quota_refresh_interval_seconds > MAX_QUOTA_REFRESH_INTERVAL_SECONDS {
        config.quota_refresh_interval_seconds = MAX_QUOTA_REFRESH_INTERVAL_SECONDS;
    }
}

fn runtime_settings_from_config(config: &CodexServerConfig) -> CodexRuntimeSettings {
    CodexRuntimeSettings {
        quota_refresh_enabled: config.quota_refresh_enabled,
        quota_refresh_interval_seconds: config.quota_refresh_interval_seconds,
        fast_mode_enabled: config.fast_mode_enabled,
    }
}

/// 根据快速模式开关同步编辑用户目录下的 ~/.codex/config.toml：
/// 开启时写入 service_tier = "fast" 和 [features] fast_mode = true；
/// 关闭时移除这两处。路径与 Codex 切换账号一致。
fn apply_fast_mode_to_codex_config_toml(
    app: &tauri::AppHandle,
    fast_mode_enabled: bool,
) -> Result<(), String> {
    let home_dir = app
        .path()
        .home_dir()
        .map_err(|e| format!("Failed to get home directory: {}", e))?;
    let codex_dir = home_dir.join(".codex");
    let config_path = codex_dir.join("config.toml");

    if !fast_mode_enabled && !config_path.exists() {
        return Ok(());
    }

    let mut config: toml::Table = if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config.toml: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse config.toml: {}", e))?
    } else {
        fs::create_dir_all(&codex_dir)
            .map_err(|e| format!("Failed to create .codex directory: {}", e))?;
        toml::Table::new()
    };

    if fast_mode_enabled {
        config.insert(
            "service_tier".to_string(),
            toml::Value::String("fast".to_string()),
        );
        let mut features = toml::Table::new();
        features.insert(
            "fast_mode".to_string(),
            toml::Value::Boolean(true),
        );
        config.insert("features".to_string(), toml::Value::Table(features));
    } else {
        config.remove("service_tier");
        config.remove("features");
    }

    let content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config.toml: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config.toml: {}", e))?;
    Ok(())
}

fn read_persisted_config(app: &tauri::AppHandle) -> Result<Option<CodexServerConfig>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    let config_path = app_data_dir.join(CODEX_CONFIG_FILE);
    if !config_path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read {}: {}", CODEX_CONFIG_FILE, e))?;
    if content.trim().is_empty() {
        return Ok(None);
    }
    let mut config: CodexServerConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", CODEX_CONFIG_FILE, e))?;
    normalize_access_fields(&mut config);
    normalize_server_port(&mut config);
    normalize_runtime_fields(&mut config);
    Ok(Some(config))
}

fn write_persisted_config(
    app: &tauri::AppHandle,
    config: &CodexServerConfig,
) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    let config_path = app_data_dir.join(CODEX_CONFIG_FILE);
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize codex config: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write {}: {}", CODEX_CONFIG_FILE, e))?;
    Ok(())
}

fn get_or_load_codex_config(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<CodexServerConfig, String> {
    if let Some(config) = state.codex_server_config.lock().unwrap().clone() {
        return Ok(config);
    }
    if let Some(config) = read_persisted_config(app)? {
        let _ = write_persisted_config(app, &config);
        *state.codex_server_config.lock().unwrap() = Some(config.clone());
        return Ok(config);
    }
    let mut config = CodexServerConfig::default();
    normalize_server_port(&mut config);
    normalize_runtime_fields(&mut config);
    let _ = write_persisted_config(app, &config);
    *state.codex_server_config.lock().unwrap() = Some(config.clone());
    Ok(config)
}

fn current_api_server_port(state: &AppState) -> u16 {
    state
        .api_server
        .lock()
        .unwrap()
        .as_ref()
        .map(|server| server.get_port())
        .unwrap_or(SHARED_API_SERVER_PORT)
}

async fn apply_periodic_tasks(app: tauri::AppHandle, state: &AppState, config: &CodexServerConfig) {
    if config.enabled && config.quota_refresh_enabled {
        start_periodic_quota_refresh(
            app,
            state,
            config.quota_refresh_interval_seconds,
        )
        .await;
    } else {
        stop_periodic_quota_refresh().await;
    }
}

async fn init_codex_runtime(
    app: &tauri::AppHandle,
    state: &AppState,
    config: &CodexServerConfig,
) -> Result<(), String> {
    let pool = if let Some(existing) = state.codex_pool.lock().unwrap().clone() {
        existing
    } else {
        let created = Arc::new(crate::platforms::openai::codex::pool::CodexPool::new());
        *state.codex_pool.lock().unwrap() = Some(created.clone());
        created
    };

    let accounts = crate::platforms::openai::modules::storage::list_accounts(app).await?;
    pool.refresh_from_accounts(&accounts).await;
    let strategy = match config.pool_strategy.as_str() {
        "single" => PoolStrategy::Single,
        "smart" => PoolStrategy::Smart,
        _ => PoolStrategy::RoundRobin,
    };
    pool.set_strategy(strategy).await;
    if let Some(ref account_id) = config.selected_account_id {
        pool.set_selected_account_id(account_id.clone()).await;
    }

    let executor =
        Arc::new(crate::platforms::openai::codex::executor::CodexExecutor::new(pool.clone())?);
    *state.codex_executor.lock().unwrap() = Some(executor);

    // 初始化 logger
    if state.codex_logger.lock().unwrap().is_none() {
        let logger = Arc::new(tokio::sync::RwLock::new(RequestLogger::new(3000)));
        *state.codex_logger.lock().unwrap() = Some(logger);
    }

    Ok(())
}

pub async fn init_codex_enabled_state_on_startup(
    app: &tauri::AppHandle,
    state: &AppState,
) -> Result<(), String> {
    let mut config = get_or_load_codex_config(app, state)?;
    normalize_access_fields(&mut config);
    normalize_server_port(&mut config);
    normalize_runtime_fields(&mut config);

    if config.enabled {
        init_codex_runtime(app, state, &config).await?;
        *state.codex_server.lock().unwrap() = Some(CodexServer::new(config.port));
    } else {
        *state.codex_server.lock().unwrap() = None;
    }

    apply_periodic_tasks(app.clone(), state, &config).await;

    *state.codex_server_config.lock().unwrap() = Some(config.clone());
    write_persisted_config(app, &config)?;
    Ok(())
}

/// 获取 Codex 服务器状态
#[tauri::command]
pub async fn get_codex_server_status(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<CodexServerStatus, String> {
    let running = state.codex_server.lock().unwrap().is_some();
    let pool = state.codex_pool.lock().unwrap().clone();
    let _cfg = get_or_load_codex_config(&app, state.inner())?;

    let pool_status = if let Some(p) = pool {
        Some(p.status().await)
    } else {
        None
    };

    let status = CodexServerStatus {
        running,
        port: current_api_server_port(state.inner()),
        address: format!(
            "http://127.0.0.1:{}",
            current_api_server_port(state.inner())
        ),
        pool_status,
    };

    Ok(status)
}

/// 启动 Codex 服务器
#[tauri::command]
pub async fn start_codex_server(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    mut config: CodexServerConfig,
) -> Result<(), String> {
    {
        let server = state.codex_server.lock().unwrap();
        if server.is_some() {
            return Err("Codex server is already running".to_string());
        }
    }

    let need_start_api_server = state.api_server.lock().unwrap().is_none();
    if need_start_api_server {
        // 先更新持久化配置，确保 enabled = true
        let mut persisted_config = get_or_load_codex_config(&app, state.inner())?;
        persisted_config.enabled = true;
        write_persisted_config(&app, &persisted_config)?;
        *state.codex_server_config.lock().unwrap() = Some(persisted_config);

        crate::core::api_server::start_api_server_cmd(app.clone(), state.clone())
            .await
            .map_err(|e| format!("Failed to start API server: {}", e))?;
        return Ok(());
    }

    // 合并已有配置，避免前端只传部分字段时覆盖现有配置
    if let Ok(existing) = get_or_load_codex_config(&app, state.inner()) {
        if config.api_key.is_none() {
            config.api_key = existing.api_key;
        }
        if config.pool_strategy.trim().is_empty() {
            config.pool_strategy = existing.pool_strategy;
        }
        if config.selected_account_id.is_none() {
            config.selected_account_id = existing.selected_account_id;
        }
        // Runtime settings are managed separately from server start/stop dialog.
        config.quota_refresh_enabled = existing.quota_refresh_enabled;
        config.quota_refresh_interval_seconds = existing.quota_refresh_interval_seconds;
        config.fast_mode_enabled = existing.fast_mode_enabled;
    }
    normalize_access_fields(&mut config);
    normalize_server_port(&mut config);
    normalize_runtime_fields(&mut config);
    config.enabled = true;

    init_codex_runtime(&app, state.inner(), &config).await?;
    *state.codex_server.lock().unwrap() = Some(CodexServer::new(config.port));
    *state.codex_server_config.lock().unwrap() = Some(config.clone());
    write_persisted_config(&app, &config)?;
    apply_periodic_tasks(app.clone(), state.inner(), &config).await;
    Ok(())
}

/// 停止 Codex 服务器
#[tauri::command]
pub async fn stop_codex_server(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 停用 Codex 路由并更新持久化状态
    let was_running = {
        let mut server = state.codex_server.lock().unwrap();
        server.take().is_some()
    };
    if was_running {
        let mut config = get_or_load_codex_config(&app, state.inner())?;
        config.enabled = false;
        *state.codex_server_config.lock().unwrap() = Some(config.clone());
        write_persisted_config(&app, &config)?;
        apply_periodic_tasks(app.clone(), state.inner(), &config).await;
        println!("Codex routes disabled");
        Ok(())
    } else {
        Err("Codex server is not running".to_string())
    }
}

/// 获取 Codex 号池状态
#[tauri::command]
pub async fn get_codex_pool_status(
    state: State<'_, AppState>,
) -> Result<crate::platforms::openai::codex::models::PoolStatus, String> {
    let pool = state.codex_pool.lock().unwrap().clone();
    if let Some(pool_ref) = pool {
        Ok(pool_ref.status().await)
    } else {
        Err("Codex pool not initialized".to_string())
    }
}

/// 刷新 Codex 号池
#[tauri::command]
pub async fn refresh_codex_pool(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<crate::platforms::openai::codex::models::PoolStatus, String> {
    let pool_ref = if let Some(existing) = state.codex_pool.lock().unwrap().clone() {
        existing
    } else {
        let created = Arc::new(crate::platforms::openai::codex::pool::CodexPool::new());
        *state.codex_pool.lock().unwrap() = Some(created.clone());
        created
    };

    let accounts = crate::platforms::openai::modules::storage::list_accounts(&app).await?;
    pool_ref.refresh_from_accounts(&accounts).await;
    Ok(pool_ref.status().await)
}

/// 获取内存中的请求日志
#[tauri::command]
pub async fn get_codex_logs(
    state: State<'_, AppState>,
    limit: usize,
) -> Result<Vec<RequestLog>, String> {
    let logger = state.codex_logger.lock().unwrap().clone();
    if let Some(log) = logger {
        let guard = log.read().await;
        Ok(guard.get_recent_logs(limit))
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn query_codex_logs(
    state: State<'_, AppState>,
    query: LogQuery,
) -> Result<LogPage, String> {
    let logger = state.codex_logger.lock().unwrap().clone();
    if let Some(log) = logger {
        let guard = log.read().await;
        Ok(guard.query_logs(&query))
    } else {
        Ok(LogPage {
            total: 0,
            items: vec![],
        })
    }
}

/// 获取 Token 统计
#[tauri::command]
pub async fn get_codex_stats(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<TokenStats, String> {
    let logger = state.codex_logger.lock().unwrap().clone();
    if let Some(log) = logger {
        let guard = log.read().await;
        guard.get_stats(&start_date, &end_date)
    } else {
        Ok(TokenStats {
            start_date,
            end_date,
            total_requests: 0,
            total_tokens: 0,
            per_account: vec![],
        })
    }
}

#[tauri::command]
pub async fn get_codex_period_stats(
    state: State<'_, AppState>,
) -> Result<PeriodTokenStats, String> {
    let logger = state.codex_logger.lock().unwrap().clone();
    if let Some(log) = logger {
        let guard = log.read().await;
        Ok(guard.get_period_stats(chrono::Utc::now().timestamp()))
    } else {
        Ok(PeriodTokenStats {
            today_requests: 0,
            today_tokens: 0,
            week_requests: 0,
            week_tokens: 0,
            month_requests: 0,
            month_tokens: 0,
        })
    }
}

#[tauri::command]
pub async fn get_codex_model_stats(
    state: State<'_, AppState>,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<ModelTokenStats>, String> {
    let logger = state.codex_logger.lock().unwrap().clone();
    if let Some(log) = logger {
        let guard = log.read().await;
        Ok(guard.get_model_stats(start_ts, end_ts))
    } else {
        Ok(vec![])
    }
}



#[tauri::command]
pub async fn clear_codex_logs(state: State<'_, AppState>) -> Result<(), String> {
    let logger = state.codex_logger.lock().unwrap().clone();
    if let Some(log) = logger {
        let mut guard = log.write().await;
        guard.clear();
        Ok(())
    } else {
        Ok(())
    }
}

/// 设置号池策略并持久化
#[tauri::command]
pub async fn set_codex_pool_strategy(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    strategy: String,
) -> Result<(), String> {
    let pool = state.codex_pool.lock().unwrap().clone();
    if let Some(pool_ref) = pool {
        let strategy_enum = match strategy.as_str() {
            "round-robin" => PoolStrategy::RoundRobin,
            "single" => PoolStrategy::Single,
            "smart" => PoolStrategy::Smart,
            _ => return Err(format!("Invalid strategy: {}", strategy)),
        };
        pool_ref.set_strategy(strategy_enum).await;
        let mut config = get_or_load_codex_config(&app, state.inner())?;
        config.pool_strategy = strategy;
        *state.codex_server_config.lock().unwrap() = Some(config.clone());
        write_persisted_config(&app, &config)?;

        Ok(())
    } else {
        Err("Codex pool not initialized".to_string())
    }
}



#[tauri::command]


pub async fn set_codex_selected_account(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    let pool = state.codex_pool.lock().unwrap().clone();
    if let Some(pool_ref) = pool {
        pool_ref.set_selected_account_id(account_id.clone()).await;
        let mut config = get_or_load_codex_config(&app, state.inner())?;
        config.selected_account_id = Some(account_id);
        *state.codex_server_config.lock().unwrap() = Some(config.clone());
        write_persisted_config(&app, &config)?;

        Ok(())
    } else {
        Err("Codex pool not initialized".to_string())
    }
}

#[tauri::command]
pub async fn get_codex_access_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<CodexAccessConfig, String> {
    let config = get_or_load_codex_config(&app, state.inner())?;
    Ok(CodexAccessConfig {
        server_url: format!(
            "http://127.0.0.1:{}",
            current_api_server_port(state.inner())
        ),
        api_key: config.api_key,
    })
}

#[tauri::command]
pub async fn set_codex_access_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    api_key: Option<String>,
) -> Result<CodexAccessConfig, String> {
    let mut config = get_or_load_codex_config(&app, state.inner())?;
    config.api_key = api_key;
    normalize_access_fields(&mut config);
    normalize_server_port(&mut config);
    normalize_runtime_fields(&mut config);
    *state.codex_server_config.lock().unwrap() = Some(config.clone());
    write_persisted_config(&app, &config)?;

    Ok(CodexAccessConfig {
        server_url: format!(
            "http://127.0.0.1:{}",
            current_api_server_port(state.inner())
        ),
        api_key: config.api_key,
    })
}

/// 获取 Codex 运行时设置
#[tauri::command]
pub async fn get_codex_runtime_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<CodexRuntimeSettings, String> {
    let mut config = get_or_load_codex_config(&app, state.inner())?;
    normalize_runtime_fields(&mut config);
    Ok(runtime_settings_from_config(&config))
}

#[tauri::command]
pub async fn set_codex_runtime_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    settings: CodexRuntimeSettings,
) -> Result<CodexRuntimeSettings, String> {
    let mut config = get_or_load_codex_config(&app, state.inner())?;
    config.quota_refresh_enabled = settings.quota_refresh_enabled;
    config.quota_refresh_interval_seconds = settings.quota_refresh_interval_seconds;
    config.fast_mode_enabled = settings.fast_mode_enabled;
    normalize_access_fields(&mut config);
    normalize_server_port(&mut config);
    normalize_runtime_fields(&mut config);
    *state.codex_server_config.lock().unwrap() = Some(config.clone());
    write_persisted_config(&app, &config)?;
    apply_periodic_tasks(app.clone(), state.inner(), &config).await;
    apply_fast_mode_to_codex_config_toml(&app, settings.fast_mode_enabled)?;
    Ok(runtime_settings_from_config(&config))
}

#[tauri::command]
pub async fn query_codex_logs_from_storage(
    state: State<'_, AppState>,
    query: LogQuery,
) -> Result<LogPage, String> {
    let storage = state.codex_log_storage.lock().unwrap().clone();
    if let Some(s) = storage {
        s.query_logs(&query).map_err(|e| e.to_string())
    } else {
        Ok(LogPage {
            total: 0,
            items: vec![],
        })
    }
}
#[tauri::command]
pub async fn get_codex_model_stats_from_storage(
    state: State<'_, AppState>,
    start_ts: i64,
    end_ts: i64,
) -> Result<Vec<ModelTokenStats>, String> {
    let storage = state.codex_log_storage.lock().unwrap().clone();
    if let Some(s) = storage {
        s.get_model_stats(start_ts, end_ts)
            .map_err(|e| e.to_string())
    } else {
        Ok(vec![])
    }
}
#[tauri::command]
pub async fn get_codex_period_stats_from_storage(
    state: State<'_, AppState>,
) -> Result<PeriodTokenStats, String> {
    let storage = state.codex_log_storage.lock().unwrap().clone();
    if let Some(s) = storage {
        let now_ts = chrono::Utc::now().timestamp();
        s.get_period_stats(now_ts).map_err(|e| e.to_string())
    } else {
        Ok(PeriodTokenStats {
            today_requests: 0,
            today_tokens: 0,
            week_requests: 0,
            week_tokens: 0,
            month_requests: 0,
            month_tokens: 0,
        })
    }
}

/// 从 SQLite 存储获取每日统计
#[tauri::command]
pub async fn get_codex_daily_stats_from_storage(
    state: State<'_, AppState>,
    days: Option<u32>,
) -> Result<DailyStatsResponse, String> {
    let storage = state.codex_log_storage.lock().unwrap().clone();
    if let Some(s) = storage {
        let days = days.unwrap_or(30).min(365);
        s.get_daily_stats(days).map_err(|e| e.to_string())
    } else {
        Ok(DailyStatsResponse { stats: vec![] })
    }
}

/// 清空 SQLite 存储中的日志
#[tauri::command]
pub async fn clear_codex_logs_in_storage(state: State<'_, AppState>) -> Result<usize, String> {
    let storage = state.codex_log_storage.lock().unwrap().clone();
    if let Some(s) = storage {
        s.clear_all().map_err(|e| e.to_string())
    } else {
        Ok(0)
    }
}

/// 删除指定日期之前的 SQLite 日志
#[tauri::command]
pub async fn delete_codex_logs_before(
    state: State<'_, AppState>,
    date_key: i64,
) -> Result<usize, String> {
    let storage = state.codex_log_storage.lock().unwrap().clone();
    if let Some(s) = storage {
        s.delete_before(date_key).map_err(|e| e.to_string())
    } else {
        Ok(0)
    }
}

/// 存储状态信息

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexLogStorageStatus {
    pub total_logs: i64,
    pub db_size_bytes: u64,
    pub db_path: String,
}

/// 获取日志存储状态
#[tauri::command]
pub async fn get_codex_log_storage_status(
    state: State<'_, AppState>,
) -> Result<CodexLogStorageStatus, String> {
    let storage = state.codex_log_storage.lock().unwrap().clone();
    if let Some(s) = storage {
        let total_logs = s.total_logs().map_err(|e| e.to_string())?;
        let db_size = s.db_size().map_err(|e| e.to_string())?;
        Ok(CodexLogStorageStatus {
            total_logs,
            db_size_bytes: db_size,
            db_path: format!("{:?}", s),
        })
    } else {
        Err("Codex log storage not initialized".to_string())
    }
}

/// 手动刷新日志缓冲区到数据库
#[tauri::command]
pub async fn flush_codex_logs(state: State<'_, AppState>) -> Result<(), String> {
    let storage = state.codex_log_storage.lock().unwrap().clone();
    if let Some(s) = storage {
        s.flush().await;
        Ok(())
    } else {
        Err("Codex log storage not initialized".to_string())
    }
}

/// 全时间统计

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexAllTimeStats {
    pub requests: u64,
    pub tokens: u64,
}

/// 获取全时间累计统计
#[tauri::command]
pub async fn get_codex_all_time_stats(
    state: State<'_, AppState>,
) -> Result<CodexAllTimeStats, String> {
    let storage = state.codex_log_storage.lock().unwrap().clone();
    if let Some(s) = storage {
        let (requests, tokens) = s.get_all_time_stats()?;
        Ok(CodexAllTimeStats { requests, tokens })
    } else {
        Ok(CodexAllTimeStats {
            requests: 0,
            tokens: 0,
        })
    }
}

// ==================== 定时任务 ====================

async fn start_periodic_quota_refresh(
    app: tauri::AppHandle,
    state: &AppState,
    interval_seconds: u64,
) {
    stop_periodic_quota_refresh().await;

    let pool = state.codex_pool.lock().unwrap().clone();
    let Some(pool_ref) = pool else {
        return;
    };

    let tick_seconds = interval_seconds.max(1);
    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(tick_seconds));
        interval.tick().await;
        loop {
            interval.tick().await;
            println!("[Codex] Starting periodic quota refresh...");

            let accounts = match crate::platforms::openai::modules::storage::list_accounts(&app).await
            {
                Ok(accs) => accs,
                Err(e) => {
                    eprintln!("[Codex] Failed to list accounts for quota refresh: {}", e);
                    continue;
                }
            };

            let mut refreshed = 0;
            for mut account in accounts {
                if account.account_type == crate::platforms::openai::models::account::AccountType::API
                {
                    continue;
                }
                if account
                    .quota
                    .as_ref()
                    .map(|q| q.is_forbidden)
                    .unwrap_or(false)
                {
                    continue;
                }

                match crate::platforms::openai::modules::account::refresh_quota_and_backfill(
                    &mut account,
                )
                .await
                {
                    Ok(_) => {
                        if let Err(e) =
                            crate::platforms::openai::modules::storage::save_account(&app, &account)
                                .await
                        {
                            eprintln!("[Codex] Failed to save account {}: {}", account.email, e);
                        } else {
                            refreshed += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("[Codex] Failed to refresh quota for {}: {}", account.email, e);
                    }
                }
            }

            if let Ok(accounts) = crate::platforms::openai::modules::storage::list_accounts(&app).await
            {
                pool_ref.refresh_from_accounts(&accounts).await;
            }

            println!(
                "[Codex] Periodic quota refresh completed: {} accounts refreshed",
                refreshed
            );
        }
    });

    *QUOTA_REFRESH_TASK.lock().await = Some(handle);
}

async fn stop_periodic_quota_refresh() {
    let mut task = QUOTA_REFRESH_TASK.lock().await;
    if let Some(handle) = task.take() {
        handle.abort();
        println!("[Codex] Periodic quota refresh task stopped");
    }
}
