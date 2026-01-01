pub mod platforms {
    pub mod antigravity;
    pub mod augment;
}

pub mod features {
    pub mod bookmarks;
    pub mod mail;
}

pub mod core {
    pub mod api_server;
    pub mod app_commands;
    pub mod http_client;
    pub mod proxy_config;
    pub mod proxy_helper;
}

pub mod data {
    pub mod database;
    pub mod storage;
}

// Backwards-compatible re-exports for existing module paths.
pub use core::{api_server, http_client, proxy_config, proxy_helper};
pub use data::{database, storage};
pub use features::{bookmarks, mail};
pub use platforms::{antigravity, augment};

use crate::features::mail::{gptmail, outlook};
use crate::platforms::augment::models::AugmentOAuthState;
use outlook::OutlookManager;
use database::{DatabaseConfigManager, DatabaseManager};
use storage::{DualStorage, AntigravityDualStorage};
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
    api_server: Mutex<Option<api_server::ApiServer>>,
    outlook_manager: Mutex<OutlookManager>,
    pub storage_manager: Arc<Mutex<Option<Arc<DualStorage>>>>,
    pub antigravity_storage_manager: Arc<Mutex<Option<Arc<AntigravityDualStorage>>>>,
    database_manager: Arc<Mutex<Option<Arc<DatabaseManager>>>>,
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
        .setup(|app| {
            let app_state = AppState {
                augment_oauth_state: Mutex::new(None),
                api_server: Mutex::new(None),
                outlook_manager: Mutex::new(OutlookManager::new()),
                storage_manager: Arc::new(Mutex::new(None)),
                antigravity_storage_manager: Arc::new(Mutex::new(None)),
                database_manager: Arc::new(Mutex::new(None)),
                app_session_cache: Arc::new(Mutex::new(HashMap::new())),
                app_handle: app.handle().clone(),
            };

            app.manage(app_state);

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
                        api_server: Mutex::new(None),
                        outlook_manager: Mutex::new(OutlookManager::new()),
                        storage_manager: state.storage_manager.clone(),
                        antigravity_storage_manager: state.antigravity_storage_manager.clone(),
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

            // Antigravity 管理命令
            antigravity::antigravity_list_accounts,
            antigravity::antigravity_add_account,
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
            storage::antigravity::get_antigravity_storage_status,
            storage::antigravity::antigravity_get_sync_status,

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
            proxy_config::proxy_config_exists
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
