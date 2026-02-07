pub mod platforms {
    pub mod antigravity;
    pub mod augment;
    pub mod windsurf;
    pub mod cursor;
    pub mod openai;
    pub mod claude;
}

pub mod features {
    pub mod bookmarks;
    pub mod mail;
}

pub mod core {
    pub mod api_server;
    pub mod app_commands;
    pub mod http_client;
    pub mod path_manager;
    pub mod proxy_config;
    pub mod proxy_helper;
    pub mod tray;
    pub mod telegram;
    pub mod subscription_monitor;
}

pub mod data {
    pub mod database;
    pub mod storage;
    pub mod subscription;
}

// Backwards-compatible re-exports for existing module paths.
pub use core::{api_server, http_client, proxy_config, proxy_helper, tray, telegram, subscription_monitor};
pub use data::{database, storage};
pub use features::{bookmarks, mail};
pub use platforms::{antigravity, augment, windsurf, cursor, openai, claude};

use crate::features::mail::{gptmail, outlook};
use crate::platforms::augment::models::AugmentOAuthState;
use crate::platforms::openai::models::OpenAIOAuthSession;
use crate::core::tray::TrayState;
use outlook::OutlookManager;
use database::{DatabaseConfigManager, DatabaseManager};
use storage::{DualStorage, AntigravityDualStorage, WindsurfDualStorage, CursorDualStorage, OpenAIDualStorage, ClaudeDualStorage};
use crate::data::subscription::SubscriptionDualStorage;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::SystemTime;
use tauri::{Manager, Emitter};
use tauri_plugin_deep_link::DeepLinkExt;

// App Session 缓存结构 (公开以便其他模块使用)
#[derive(Clone)]
pub struct AppSessionCache {
    pub app_session: String,
    pub created_at: SystemTime,
}

// Global state to store OAuth state and storage managers
pub struct AppState {
    augment_oauth_state: Mutex<Option<AugmentOAuthState>>,
    pub openai_oauth_sessions: Arc<Mutex<HashMap<String, OpenAIOAuthSession>>>,
    api_server: Mutex<Option<api_server::ApiServer>>,
    outlook_manager: Mutex<OutlookManager>,
    pub storage_manager: Arc<Mutex<Option<Arc<DualStorage>>>>,
    pub antigravity_storage_manager: Arc<Mutex<Option<Arc<AntigravityDualStorage>>>>,
    pub windsurf_storage_manager: Arc<Mutex<Option<Arc<WindsurfDualStorage>>>>,
    pub cursor_storage_manager: Arc<Mutex<Option<Arc<CursorDualStorage>>>>,
    pub openai_storage_manager: Arc<Mutex<Option<Arc<OpenAIDualStorage>>>>,
    pub subscription_storage_manager: Arc<Mutex<Option<Arc<SubscriptionDualStorage>>>>,
    pub claude_storage_manager: Arc<Mutex<Option<Arc<ClaudeDualStorage>>>>,
    pub database_manager: Arc<Mutex<Option<Arc<DatabaseManager>>>>,
    // App session 缓存: key 为 auth_session, value 为缓存的 app_session
    pub app_session_cache: Arc<Mutex<HashMap<String, AppSessionCache>>>,
    // App handle for emitting events
    pub app_handle: tauri::AppHandle,
}

pub fn run() {
    let mut builder = tauri::Builder::default();

    // 在桌面平台上，single-instance 插件必须是第一个注册的插件
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            eprintln!("A new app instance was opened with argv: {:?}", argv);
            eprintln!("Deep link event was already triggered by the plugin");

            // 聚焦主窗口
            if let Some(main_window) = app.get_webview_window("main") {
                let _ = main_window.set_focus();
                let _ = main_window.unminimize();
            }
        }));
    }

    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_state = AppState {
                augment_oauth_state: Mutex::new(None),
                openai_oauth_sessions: Arc::new(Mutex::new(HashMap::new())),
                api_server: Mutex::new(None),
                outlook_manager: Mutex::new(OutlookManager::new()),
                storage_manager: Arc::new(Mutex::new(None)),
                antigravity_storage_manager: Arc::new(Mutex::new(None)),
                windsurf_storage_manager: Arc::new(Mutex::new(None)),
                cursor_storage_manager: Arc::new(Mutex::new(None)),
                openai_storage_manager: Arc::new(Mutex::new(None)),
                subscription_storage_manager: Arc::new(Mutex::new(None)),
                claude_storage_manager: Arc::new(Mutex::new(None)),
                database_manager: Arc::new(Mutex::new(None)),
                app_session_cache: Arc::new(Mutex::new(HashMap::new())),
                app_handle: app.handle().clone(),
            };

            app.manage(app_state);

            // 初始化托盘状态管理器
            app.manage(TrayState::new());

            // 异步初始化存储管理器
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<AppState>();

                // 尝试加载数据库配置
                match DatabaseConfigManager::new(&app_handle) {
                    Ok(config_manager) => {
                        match config_manager.load_config() {
                            Ok(config) => {
                                if config.enabled {
                                    let mut db_manager = DatabaseManager::new(config);
                                    if db_manager.initialize().await.is_ok() {
                                        // 检查表是否存在
                                        if let Some(pool) = db_manager.get_pool() {
                                            match pool.get().await {
                                                Ok(client) => {
                                                        match database::augment::check_tables_exist(&client).await {
                                                            Ok(exists) => {
                                                                if !exists {
                                                                    // 创建表
                                                                    if let Err(e) = database::augment::create_tables(&client).await {
                                                                        eprintln!("Failed to create tables on startup: {}", e);
                                                                    }
                                                                } else {
                                                                    // 表已存在，检查并添加新字段
                                                                    if let Err(e) = database::augment::add_new_fields_if_not_exist(&client).await {
                                                                        eprintln!("Failed to add new fields on startup: {}", e);
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Failed to check tables on startup: {}", e);
                                                            }
                                                        }
                                                        match database::antigravity::check_tables_exist(&client).await {
                                                            Ok(exists) => {
                                                                if !exists {
                                                                    if let Err(e) = database::antigravity::create_tables(&client).await {
                                                                        eprintln!("Failed to create Antigravity tables on startup: {}", e);
                                                                    }
                                                                } else if let Err(e) = database::antigravity::add_new_fields_if_not_exist(&client).await {
                                                                    eprintln!("Failed to add new fields to Antigravity tables on startup: {}", e);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Failed to check Antigravity tables on startup: {}", e);
                                                            }
                                                        }
                                                        match database::windsurf::check_tables_exist(&client).await {
                                                            Ok(exists) => {
                                                                if !exists {
                                                                    if let Err(e) = database::windsurf::create_tables(&client).await {
                                                                        eprintln!("Failed to create Windsurf tables on startup: {}", e);
                                                                    }
                                                                } else if let Err(e) = database::windsurf::add_new_fields_if_not_exist(&client).await {
                                                                    eprintln!("Failed to update Windsurf tables on startup: {}", e);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Failed to check Windsurf tables on startup: {}", e);
                                                            }
                                                        }
                                                        match database::cursor::check_tables_exist(&client).await {
                                                            Ok(exists) => {
                                                                if !exists {
                                                                    if let Err(e) = database::cursor::create_tables(&client).await {
                                                                        eprintln!("Failed to create Cursor tables on startup: {}", e);
                                                                    }
                                                                } else if let Err(e) = database::cursor::add_new_fields_if_not_exist(&client).await {
                                                                    eprintln!("Failed to update Cursor tables on startup: {}", e);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Failed to check Cursor tables on startup: {}", e);
                                                            }
                                                        }
                                                        match database::openai::check_tables_exist(&client).await {
                                                            Ok(exists) => {
                                                                if !exists {
                                                                    if let Err(e) = database::openai::create_tables(&client).await {
                                                                        eprintln!("Failed to create OpenAI tables on startup: {}", e);
                                                                    }
                                                                } else if let Err(e) = database::openai::add_new_fields_if_not_exist(&client).await {
                                                                    eprintln!("Failed to update OpenAI tables on startup: {}", e);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Failed to check OpenAI tables on startup: {}", e);
                                                            }
                                                        }
                                                        match database::claude::check_tables_exist(&client).await {
                                                            Ok(exists) => {
                                                                if !exists {
                                                                    if let Err(e) = database::claude::create_tables(&client).await {
                                                                        eprintln!("Failed to create Claude tables on startup: {}", e);
                                                                    }
                                                                } else if let Err(e) = database::claude::add_new_fields_if_not_exist(&client).await {
                                                                    eprintln!("Failed to update Claude tables on startup: {}", e);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Failed to check Claude tables on startup: {}", e);
                                                            }
                                                        }
                                                        match crate::data::subscription::migrations::check_tables_exist(&client).await {
                                                            Ok(exists) => {
                                                                if !exists {
                                                                    if let Err(e) = crate::data::subscription::migrations::create_tables(&client).await {
                                                                        eprintln!("Failed to create Subscription tables on startup: {}", e);
                                                                    }
                                                                } else if let Err(e) = crate::data::subscription::migrations::add_new_fields_if_not_exist(&client).await {
                                                                    eprintln!("Failed to update Subscription tables on startup: {}", e);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Failed to check Subscription tables on startup: {}", e);
                                                            }
                                                        }
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to get database client on startup: {}", e);
                                                }
                                            }
                                        }

                                        *state.database_manager.lock().unwrap() = Some(Arc::new(db_manager));
                                    }
                                }
                            }
                            Err(e) => eprintln!("Failed to load database config: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Failed to create config manager: {}", e),
                }

                // 初始化存储管理器
                if let Err(e) = storage::initialize_storage_manager(&app_handle, &state).await {
                    eprintln!("Failed to initialize storage manager: {}", e);
                }
                if let Err(e) = storage::initialize_antigravity_storage_manager(&app_handle, &state).await {
                    eprintln!("Failed to initialize Antigravity storage manager: {}", e);
                }
                if let Err(e) = storage::initialize_windsurf_storage_manager(&app_handle, &state).await {
                    eprintln!("Failed to initialize Windsurf storage manager: {}", e);
                }
                if let Err(e) = storage::initialize_cursor_storage_manager(&app_handle, &state).await {
                    eprintln!("Failed to initialize Cursor storage manager: {}", e);
                }
                if let Err(e) = storage::initialize_openai_storage_manager(&app_handle, &state).await {
                    eprintln!("Failed to initialize OpenAI storage manager: {}", e);
                }
                if let Err(e) = storage::initialize_claude_storage_manager(&app_handle, &state).await {
                    eprintln!("Failed to initialize Claude storage manager: {}", e);
                }
                if let Err(e) = crate::data::subscription::initialize_subscription_storage_manager(&app_handle, &state).await {
                    eprintln!("Failed to initialize Subscription storage manager: {}", e);
                }
            });

            // 启动 API 服务器
            let app_handle_for_api = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle_for_api.state::<AppState>();

                // 等待存储管理器初始化完成
                let mut attempts = 0;
                while attempts < 50 {
                    let storage_ready = {
                        let storage_guard = state.storage_manager.lock().unwrap();
                        storage_guard.is_some()
                    };

                    if storage_ready {
                        break;
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    attempts += 1;
                }

                // 启动 API 服务器
                match api_server::start_api_server(
                    Arc::new(AppState {
                        augment_oauth_state: Mutex::new(None),
                        openai_oauth_sessions: state.openai_oauth_sessions.clone(),
                        api_server: Mutex::new(None),
                        outlook_manager: Mutex::new(OutlookManager::new()),
                        storage_manager: state.storage_manager.clone(),
                        antigravity_storage_manager: state.antigravity_storage_manager.clone(),
                        windsurf_storage_manager: state.windsurf_storage_manager.clone(),
                        cursor_storage_manager: state.cursor_storage_manager.clone(),
                        openai_storage_manager: state.openai_storage_manager.clone(),
                        claude_storage_manager: state.claude_storage_manager.clone(),
                        subscription_storage_manager: state.subscription_storage_manager.clone(),
                        database_manager: state.database_manager.clone(),
                        app_session_cache: state.app_session_cache.clone(),
                        app_handle: app_handle_for_api.clone(),
                    }),
                    8766,
                ).await {
                    Ok(server) => {
                        println!("✅ API Server initialized successfully");
                        *state.api_server.lock().unwrap() = Some(server);
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to start API server: {}", e);
                    }
                }
            });

            // 启动订阅到期监控定时任务
            let app_handle_for_monitor = app.handle().clone();
            subscription_monitor::start_subscription_monitor(app_handle_for_monitor);

            // 设置 deep-link 处理器
            let app_handle_for_deep_link = app.app_handle().clone();
            app.deep_link().on_open_url(move |event| {
                let urls = event.urls();
                eprintln!("Deep link received: {:?}", urls);

                for parsed_url in urls {
                    let url_str = parsed_url.as_str();

                    // 检查是否是 atm://import 协议
                    if url_str.starts_with("atm://import") {
                        eprintln!("Processing ATM import deep link: {}", url_str);

                        // 查找 session 参数
                        if let Some(session) = parsed_url
                            .query_pairs()
                            .find(|(key, _)| key == "session")
                            .map(|(_, value)| value.to_string())
                        {
                            eprintln!("Found session parameter in deep link");

                            // 克隆需要的变量
                            let app_handle = app_handle_for_deep_link.clone();

                            // 在异步任务中等待窗口就绪并发送 session 到前端
                            tauri::async_runtime::spawn(async move {
                                // 等待主窗口加载完成（最多等待 10 秒）
                                let mut attempts = 0;
                                let max_attempts = 100; // 100 * 100ms = 10 秒
                                while attempts < max_attempts {
                                    if let Some(main_window) = app_handle.get_webview_window("main") {
                                        // 窗口存在，再等待一小段时间确保前端事件监听器已注册
                                        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

                                        // 聚焦主窗口
                                        let _ = main_window.set_focus();
                                        let _ = main_window.unminimize();

                                        // 发送 session 到前端，由前端调用导入方法
                                        let _ = app_handle.emit(
                                            "deep-link-session-received",
                                            serde_json::json!({
                                                "session": session
                                            })
                                        );

                                        break;
                                    }
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                    attempts += 1;
                                }
                            });
                        } else {
                            eprintln!("No session parameter found in deep link URL");
                        }
                    }
                }
            });

            // 在 Linux 和 Windows 上注册协议
            // Linux: 总是注册（AppImage 需要运行时注册）
            // Windows: 总是注册（确保协议在所有模式下都可用）
            // macOS: 不支持运行时注册，必须通过 bundle 配置
            #[cfg(any(target_os = "linux", windows))]
            {
                if let Err(e) = app.deep_link().register_all() {
                    eprintln!("Failed to register deep link protocols: {}", e);
                } else {
                    eprintln!("Successfully registered deep link protocols");
                }
            }

            // 监听主窗口关闭事件，实现最小化到托盘功能
            if let Some(main_window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        // 检查托盘是否启用
                        let tray_state = app_handle.state::<TrayState>();
                        let tray_enabled = tray_state.tray_icon.lock()
                            .map(|guard| guard.is_some())
                            .unwrap_or(false);
                        
                        if tray_enabled {
                            // 托盘已启用，阻止关闭并隐藏窗口
                            api.prevent_close();
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        // 托盘未启用时，允许正常关闭（退出应用）
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Augment 命令
            augment::generate_augment_auth_url,
            augment::get_augment_token,
            augment::batch_check_tokens_status,
            augment::fetch_batch_credit_consumption,
            augment::fetch_payment_method_link_command,
            augment::add_token_from_session,
            augment::batch_refresh_sessions,
            // Augment 团队管理命令
            augment::fetch_team_info,
            augment::update_team_seats,
            augment::invite_team_members,
            augment::delete_team_member,
            augment::delete_team_invitation,

            // OpenAI OAuth 命令
            openai::openai_generate_auth_url,
            openai::openai_exchange_code,
            openai::openai_refresh_token,
            openai::openai_list_accounts,
            openai::openai_load_account,
            openai::openai_save_account,
            openai::openai_delete_account,
            openai::openai_get_current_account_id,
            openai::openai_set_current_account_id,
            openai::openai_fetch_quota,
            openai::openai_refresh_all_quotas,
            openai::openai_refresh_account_token,
            openai::openai_refresh_all_tokens,
            openai::openai_load_accounts_json,
            openai::openai_add_account,
            openai::openai_add_api_account,
            openai::openai_update_api_account,
            openai::openai_save_accounts,
            openai::openai_update_account,
            openai::openai_refresh_account,
            openai::openai_start_oauth_login,
            openai::openai_cancel_oauth_login,
            openai::openai_switch_account,

            // Antigravity 管理命令
            antigravity::antigravity_list_accounts,
            antigravity::antigravity_load_accounts_json,
            antigravity::antigravity_add_account,
            antigravity::antigravity_update_account,
            antigravity::antigravity_delete_account,
            antigravity::antigravity_get_current_account,
            antigravity::antigravity_fetch_quota,
            antigravity::antigravity_refresh_all_quotas,
            antigravity::antigravity_switch_account,
            antigravity::antigravity_check_installation,
            antigravity::antigravity_is_running,
            antigravity::antigravity_launch,
            antigravity::antigravity_get_auth_url,
            antigravity::antigravity_exchange_code,
            antigravity::antigravity_start_oauth_login,
            antigravity::antigravity_cancel_oauth_login,
            antigravity::antigravity_get_custom_path,
            antigravity::antigravity_set_custom_path,
            antigravity::antigravity_validate_path,
            antigravity::antigravity_get_default_path,
            antigravity::antigravity_select_executable_path,

            // Windsurf 管理命令
            windsurf::windsurf_login,
            windsurf::windsurf_add_account,
            windsurf::windsurf_list_accounts,
            windsurf::windsurf_update_account,
            windsurf::windsurf_delete_account,
            windsurf::windsurf_switch_account,
            windsurf::windsurf_fetch_quota,
            windsurf::windsurf_fetch_all_quotas,
            windsurf::windsurf_get_custom_path,
            windsurf::windsurf_set_custom_path,
            windsurf::windsurf_validate_path,
            windsurf::windsurf_get_default_path,
            windsurf::windsurf_select_executable_path,
            // Windsurf 同步命令
            data::storage::windsurf::windsurf_sync_accounts_to_database,
            data::storage::windsurf::windsurf_sync_accounts_from_database,
            data::storage::windsurf::windsurf_bidirectional_sync_accounts,
            data::storage::windsurf::windsurf_sync_accounts,

            // Cursor 管理命令
            cursor::cursor_get_user_info_from_session,
            cursor::cursor_refresh_account_tokens,
            cursor::cursor_add_account_with_session,
            cursor::cursor_import_accounts,
            cursor::cursor_update_account,
            cursor::cursor_list_accounts,
            cursor::cursor_delete_account,
            cursor::cursor_switch_account,
            cursor::cursor_generate_and_bind_machine_id,
            cursor::cursor_export_accounts,
            cursor::cursor_get_custom_path,
            cursor::cursor_set_custom_path,
            cursor::cursor_validate_path,
            cursor::cursor_get_default_path,
            cursor::cursor_select_executable_path,
            cursor::cursor_validate_account,
            cursor::cursor_get_subscription_info,
            cursor::cursor_get_aggregated_usage,
            cursor::cursor_get_filtered_usage_events,

            // 书签管理命令
            bookmarks::add_bookmark,
            bookmarks::update_bookmark,
            bookmarks::delete_bookmark,
            bookmarks::get_all_bookmarks,

            // 核心应用命令
            core::app_commands::open_url,
            core::app_commands::open_data_folder,
            core::app_commands::open_editor_with_protocol,
            core::app_commands::create_jetbrains_token_file,
            core::app_commands::configure_vim_augment,
            core::app_commands::configure_auggie,
            core::app_commands::open_internal_browser,
            core::app_commands::check_for_updates,
            core::app_commands::copy_to_clipboard,

            // 存储命令
            storage::save_tokens_json,
            storage::load_tokens_json,
            storage::bidirectional_sync_tokens,
            storage::sync_tokens,
            storage::get_storage_status,
            storage::get_sync_status,
            storage::antigravity::antigravity_sync_accounts_to_database,
            storage::antigravity::antigravity_sync_accounts_from_database,
            storage::antigravity::antigravity_bidirectional_sync_accounts,
            storage::antigravity::antigravity_sync_accounts,
            storage::antigravity::antigravity_get_sync_status,
            // OpenAI 同步命令
            storage::openai::openai_sync_accounts_to_database,
            storage::openai::openai_sync_accounts_from_database,
            storage::openai::openai_bidirectional_sync_accounts,
            storage::openai::openai_sync_accounts,
            storage::openai::openai_get_sync_status,
            // Cursor 同步命令
            storage::cursor::cursor_sync_accounts_to_database,
            storage::cursor::cursor_sync_accounts_from_database,
            storage::cursor::cursor_bidirectional_sync_accounts,
            storage::cursor::cursor_sync_accounts,
            storage::cursor::cursor_get_sync_status,

            // 订阅同步命令
            data::subscription::subscription_sync_accounts,
            data::subscription::subscription_sync_to_database,
            data::subscription::subscription_sync_from_database,
            data::subscription::subscription_bidirectional_sync,
            // 订阅 CRUD 命令
            data::subscription::subscription_list,
            data::subscription::subscription_add,
            data::subscription::subscription_update,
            data::subscription::subscription_delete,

            // Claude 账户管理命令
            storage::claude::claude_list,
            storage::claude::claude_add,
            storage::claude::claude_update,
            storage::claude::claude_delete,
            storage::claude::claude_sync_accounts,
            storage::claude::claude_sync_accounts_to_database,
            storage::claude::claude_sync_accounts_from_database,
            storage::claude::claude_bidirectional_sync_accounts,
            storage::claude::claude_switch_account,
            storage::claude::claude_get_current_account_id,

            // API 服务器管理命令
            api_server::get_api_server_status,
            api_server::start_api_server_cmd,
            api_server::stop_api_server,

            // Outlook 邮箱管理命令
            outlook::outlook_save_credentials,
            outlook::outlook_get_all_accounts_info,
            outlook::outlook_delete_account,
            outlook::outlook_check_account_status,
            outlook::outlook_get_emails,
            outlook::outlook_get_email_details,

            // GPTMail 管理命令
            gptmail::generate_random_email,
            gptmail::get_emails,

            // 数据库配置命令
            database::save_database_config,
            database::load_database_config,
            database::test_database_connection_cmd,
            database::delete_database_config,

            // 代理配置命令
            proxy_config::save_proxy_config,
            proxy_config::load_proxy_config,
            proxy_config::test_proxy_config,
            proxy_config::delete_proxy_config,
            proxy_config::proxy_config_exists,

            // 系统托盘命令
            tray::create_tray,
            tray::destroy_tray,
            tray::toggle_tray,
            tray::get_tray_status,

            // Telegram 通知命令
            telegram::save_telegram_config,
            telegram::load_telegram_config,
            telegram::delete_telegram_config,
            telegram::test_telegram_connection_cmd,
            telegram::send_telegram_message_cmd,

            // 订阅监控命令
            subscription_monitor::check_subscriptions_expiry,
            subscription_monitor::get_expiring_subscriptions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
