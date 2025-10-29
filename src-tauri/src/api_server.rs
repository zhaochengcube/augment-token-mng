use std::sync::Arc;
use tokio::sync::{oneshot, Semaphore};
use warp::{Filter, Reply, Rejection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tauri::Emitter;
use crate::storage::{TokenData, TokenStorage};

// ==================== 数据结构定义 ====================

/// 单个 session 导入请求
#[derive(Debug, Deserialize)]
pub struct ImportSessionRequest {
    pub session: String,
    #[serde(default = "default_detailed_response")]
    pub detailed_response: bool,
}

/// 批量 session 导入请求
#[derive(Debug, Deserialize)]
pub struct ImportSessionsRequest {
    pub sessions: Vec<String>,
    #[serde(default = "default_detailed_response")]
    pub detailed_response: bool,
}

/// 默认返回详细响应
fn default_detailed_response() -> bool {
    true
}

/// 单个导入结果
#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_data: Option<TokenData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_preview: Option<String>,
}

/// 批量导入结果
#[derive(Debug, Serialize)]
pub struct BatchImportResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<ImportResult>,
}

/// 健康检查响应
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub port: u16,
}

/// API 错误响应
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub code: String,
}

/// 简化导入响应
#[derive(Debug, Serialize)]
pub struct SimpleImportResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

// ==================== API 服务器结构 ====================

pub struct ApiServer {
    shutdown_tx: Option<oneshot::Sender<()>>,
    port: u16,
}

impl ApiServer {
    pub fn new(port: u16) -> Self {
        Self {
            shutdown_tx: None,
            port,
        }
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
            println!("🛑 API Server shutdown signal sent");
        }
    }
}

impl Drop for ApiServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// ==================== 辅助函数 ====================

/// 脱敏 session 字符串（只显示前4位和后1位）
fn mask_session(session: &str) -> String {
    if session.len() <= 5 {
        return "***".to_string();
    }
    format!("{}***{}", &session[..4], &session[session.len()-1..])
}

/// 验证 session 格式
fn validate_session(session: &str) -> Result<(), String> {
    if session.trim().is_empty() {
        return Err("Session cannot be empty".to_string());
    }
    if session.len() < 10 {
        return Err("Session is too short".to_string());
    }
    Ok(())
}

// ==================== 路由处理器 ====================

/// 健康检查处理器
async fn health_handler(port: u16) -> Result<impl Reply, Rejection> {
    let response = HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        port,
    };
    Ok(warp::reply::json(&response))
}

/// 单个 session 导入处理器
async fn import_session_handler(
    request: ImportSessionRequest,
    state: Arc<crate::AppState>,
) -> Result<impl Reply, Rejection> {
    println!("📥 API: Importing single session: {}", mask_session(&request.session));

    // 验证 session
    if let Err(e) = validate_session(&request.session) {
        let error_response = ApiErrorResponse {
            error: e,
            code: "INVALID_SESSION".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&error_response),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // 调用内部函数导入
    match crate::add_token_from_session_internal_with_cache(&request.session, &state).await {
        Ok(response) => {
            // 检查重复 email（与前端逻辑保持一致）
            if let Some(ref email_note) = response.user_info.email_note {
                let email_to_check = email_note.trim().to_lowercase();

                // 从 storage_manager 加载现有 tokens
                let storage_manager = {
                    let guard = state.storage_manager.lock().unwrap();
                    guard.clone()
                };

                if let Some(storage) = storage_manager {
                    match storage.load_tokens().await {
                        Ok(existing_tokens) => {
                            // 检查是否存在相同的 email
                            if existing_tokens.iter().any(|token| {
                                if let Some(ref existing_email) = token.email_note {
                                    existing_email.trim().to_lowercase() == email_to_check
                                } else {
                                    false
                                }
                            }) {
                                println!("⚠️  API: Duplicate email detected: {}", email_note);
                                let error_response = ApiErrorResponse {
                                    error: format!("Token with email '{}' already exists", email_note),
                                    code: "DUPLICATE_EMAIL".to_string(),
                                };
                                return Ok(warp::reply::with_status(
                                    warp::reply::json(&error_response),
                                    warp::http::StatusCode::CONFLICT,
                                ));
                            }
                        }
                        Err(e) => {
                            eprintln!("⚠️  API: Failed to load existing tokens for duplicate check: {}", e);
                            // 继续导入，不因为加载失败而阻止导入
                        }
                    }
                }
            }

            // 使用 UUID 生成唯一 ID（与前端逻辑保持一致）
            let id = Uuid::new_v4().to_string();

            // 构造 TokenData（与前端逻辑保持一致）
            let token_data = TokenData {
                id,
                tenant_url: response.tenant_url.clone(),
                access_token: response.access_token.clone(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                portal_url: response.user_info.portal_url.clone(),
                email_note: response.user_info.email_note.clone(),
                tag_name: None,
                tag_color: None,
                ban_status: None,  // 与前端保持一致，设置为 null
                portal_info: None,
                auth_session: Some(request.session.clone()),
                suspensions: response.user_info.suspensions.clone(),
                balance_color_mode: None,
                skip_check: Some(false),  // 与前端保持一致，默认不跳过检测
            };

            // 保存到存储
            let storage = {
                let storage_guard = state.storage_manager.lock().unwrap();
                storage_guard.as_ref().cloned()
            };

            let storage_result = if let Some(storage) = storage {
                storage.save_token(&token_data).await.map_err(|e| e.to_string())
            } else {
                Err("Storage manager not initialized".to_string())
            };

            match storage_result {
                Ok(_) => {
                    println!("✅ API: Session imported successfully");

                    // 发送前端刷新事件
                    if let Err(e) = state.app_handle.emit("tokens-updated", ()) {
                        eprintln!("⚠️  Failed to emit tokens-updated event: {}", e);
                    }

                    // 根据 detailed_response 参数返回不同格式
                    if request.detailed_response {
                        // 返回完整响应
                        let result = ImportResult {
                            success: true,
                            token_data: Some(token_data),
                            error: None,
                            session_preview: Some(mask_session(&request.session)),
                        };
                        Ok(warp::reply::with_status(
                            warp::reply::json(&result),
                            warp::http::StatusCode::OK,
                        ))
                    } else {
                        // 返回简化响应
                        let result = SimpleImportResult {
                            success: true,
                            message: Some("Session imported successfully".to_string()),
                            error: None,
                            code: None,
                        };
                        Ok(warp::reply::with_status(
                            warp::reply::json(&result),
                            warp::http::StatusCode::OK,
                        ))
                    }
                }
                Err(e) => {
                    eprintln!("❌ API: Failed to save token: {}", e);
                    let error_response = ApiErrorResponse {
                        error: format!("Failed to save token: {}", e),
                        code: "STORAGE_ERROR".to_string(),
                    };
                    Ok(warp::reply::with_status(
                        warp::reply::json(&error_response),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Err(e) => {
            eprintln!("❌ API: Failed to import session: {}", e);
            let error_response = ApiErrorResponse {
                error: e,
                code: "IMPORT_ERROR".to_string(),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::UNPROCESSABLE_ENTITY,
            ))
        }
    }
}

/// 批量 session 导入处理器
async fn import_sessions_handler(
    request: ImportSessionsRequest,
    state: Arc<crate::AppState>,
) -> Result<impl Reply, Rejection> {
    println!("📥 API: Importing {} sessions", request.sessions.len());

    // 验证请求
    if request.sessions.is_empty() {
        let error_response = ApiErrorResponse {
            error: "Sessions array cannot be empty".to_string(),
            code: "EMPTY_ARRAY".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&error_response),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if request.sessions.len() > 100 {
        let error_response = ApiErrorResponse {
            error: "Too many sessions (max 100)".to_string(),
            code: "TOO_MANY_SESSIONS".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&error_response),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // 使用 Semaphore 限制并发
    let semaphore = Arc::new(Semaphore::new(5));
    let mut tasks = Vec::new();

    for session in request.sessions.iter() {
        let session = session.clone();
        let state = state.clone();
        let semaphore = semaphore.clone();

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            // 验证 session
            if let Err(e) = validate_session(&session) {
                return ImportResult {
                    success: false,
                    token_data: None,
                    error: Some(e),
                    session_preview: Some(mask_session(&session)),
                };
            }

            // 导入 session
            match crate::add_token_from_session_internal_with_cache(&session, &state).await {
                Ok(response) => {
                    // 检查重复 email（与前端逻辑保持一致）
                    if let Some(ref email_note) = response.user_info.email_note {
                        let email_to_check = email_note.trim().to_lowercase();

                        // 从 storage_manager 加载现有 tokens
                        let storage_manager = {
                            let guard = state.storage_manager.lock().unwrap();
                            guard.clone()
                        };

                        if let Some(storage) = storage_manager {
                            match storage.load_tokens().await {
                                Ok(existing_tokens) => {
                                    // 检查是否存在相同的 email
                                    if existing_tokens.iter().any(|token| {
                                        if let Some(ref existing_email) = token.email_note {
                                            existing_email.trim().to_lowercase() == email_to_check
                                        } else {
                                            false
                                        }
                                    }) {
                                        println!("⚠️  API: Duplicate email detected in batch: {}", email_note);
                                        return ImportResult {
                                            success: false,
                                            token_data: None,
                                            error: Some(format!("Token with email '{}' already exists", email_note)),
                                            session_preview: Some(mask_session(&session)),
                                        };
                                    }
                                }
                                Err(e) => {
                                    eprintln!("⚠️  API: Failed to load existing tokens for duplicate check: {}", e);
                                    // 继续导入，不因为加载失败而阻止导入
                                }
                            }
                        }
                    }

                    // 使用 UUID 生成唯一 ID（与前端逻辑保持一致）
                    let id = Uuid::new_v4().to_string();

                    let token_data = TokenData {
                        id,
                        tenant_url: response.tenant_url.clone(),
                        access_token: response.access_token.clone(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        portal_url: response.user_info.portal_url.clone(),
                        email_note: response.user_info.email_note.clone(),
                        tag_name: None,
                        tag_color: None,
                        ban_status: None,  // 与前端保持一致，设置为 null
                        portal_info: None,
                        auth_session: Some(session.clone()),
                        suspensions: response.user_info.suspensions.clone(),
                        balance_color_mode: None,
                        skip_check: Some(false),  // 与前端保持一致，默认不跳过检测
                    };

                    // 保存到存储
                    let storage = {
                        let storage_guard = state.storage_manager.lock().unwrap();
                        storage_guard.as_ref().cloned()
                    };

                    let storage_result = if let Some(storage) = storage {
                        storage.save_token(&token_data).await.map_err(|e| e.to_string())
                    } else {
                        Err("Storage manager not initialized".to_string())
                    };

                    match storage_result {
                        Ok(_) => ImportResult {
                            success: true,
                            token_data: Some(token_data),
                            error: None,
                            session_preview: Some(mask_session(&session)),
                        },
                        Err(e) => ImportResult {
                            success: false,
                            token_data: None,
                            error: Some(format!("Storage error: {}", e)),
                            session_preview: Some(mask_session(&session)),
                        },
                    }
                }
                Err(e) => ImportResult {
                    success: false,
                    token_data: None,
                    error: Some(e),
                    session_preview: Some(mask_session(&session)),
                },
            }
        });

        tasks.push(task);
    }

    // 等待所有任务完成
    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(result) => results.push(result),
            Err(e) => {
                results.push(ImportResult {
                    success: false,
                    token_data: None,
                    error: Some(format!("Task error: {}", e)),
                    session_preview: None,
                });
            }
        }
    }

    // 统计结果
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    println!("✅ API: Batch import completed - {}/{} successful", successful, results.len());

    // 如果有成功导入的 token，发送前端刷新事件
    if successful > 0 {
        if let Err(e) = state.app_handle.emit("tokens-updated", ()) {
            eprintln!("⚠️  Failed to emit tokens-updated event: {}", e);
        }
    }

    // 根据 detailed_response 参数返回不同格式
    if request.detailed_response {
        // 返回完整响应
        let batch_result = BatchImportResult {
            total: results.len(),
            successful,
            failed,
            results,
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&batch_result),
            warp::http::StatusCode::OK,
        ))
    } else {
        // 返回简化响应
        let result = SimpleImportResult {
            success: true,
            message: Some(format!("{} of {} sessions imported successfully", successful, results.len())),
            error: None,
            code: None,
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&result),
            warp::http::StatusCode::OK,
        ))
    }
}

// ==================== 服务器启动 ====================

/// 启动 API 服务器
pub async fn start_api_server(
    state: Arc<crate::AppState>,
    start_port: u16,
) -> Result<ApiServer, String> {
    println!("🚀 Starting API Server...");

    // 尝试绑定端口（从 start_port 到 start_port + 9）
    let mut last_error = String::new();

    for port in start_port..start_port + 10 {
        match try_bind_server(state.clone(), port).await {
            Ok(server) => {
                println!("✅ API Server started successfully on http://127.0.0.1:{}", port);
                println!("📡 Available endpoints:");
                println!("   - GET  http://127.0.0.1:{}/api/health", port);
                println!("   - POST http://127.0.0.1:{}/api/import/session", port);
                println!("   - POST http://127.0.0.1:{}/api/import/sessions", port);
                return Ok(server);
            }
            Err(e) => {
                last_error = e;
                eprintln!("⚠️  Port {} is in use, trying next port...", port);
            }
        }
    }

    Err(format!("Failed to start API server: {}", last_error))
}

/// 尝试在指定端口绑定服务器
async fn try_bind_server(
    state: Arc<crate::AppState>,
    port: u16,
) -> Result<ApiServer, String> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // 克隆 state 用于各个路由
    let state_filter = warp::any().map(move || state.clone());
    let port_filter = warp::any().map(move || port);

    // 健康检查路由
    let health_route = warp::path!("api" / "health")
        .and(warp::get())
        .and(port_filter.clone())
        .and_then(health_handler);

    // 单个 session 导入路由
    let import_session_route = warp::path!("api" / "import" / "session")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 1024)) // 1MB 限制
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(import_session_handler);

    // 批量 session 导入路由
    let import_sessions_route = warp::path!("api" / "import" / "sessions")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 1024)) // 1MB 限制
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(import_sessions_handler);

    // 组合所有路由
    let routes = health_route
        .or(import_session_route)
        .or(import_sessions_route)
        .with(
            warp::cors()
                .allow_any_origin() // 允许任何来源（因为只监听 localhost）
                .allow_methods(vec!["GET", "POST"])
                .allow_headers(vec!["Content-Type"])
        )
        .recover(handle_rejection);

    // 尝试绑定端口
    let (_addr, server) = warp::serve(routes)
        .try_bind_with_graceful_shutdown(([127, 0, 0, 1], port), async {
            shutdown_rx.await.ok();
        })
        .map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

    // 在后台启动服务器
    tokio::spawn(server);

    Ok(ApiServer {
        shutdown_tx: Some(shutdown_tx),
        port,
    })
}

/// 处理 warp 拒绝错误
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.is_not_found() {
        let error_response = ApiErrorResponse {
            error: "Endpoint not found".to_string(),
            code: "NOT_FOUND".to_string(),
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&error_response),
            warp::http::StatusCode::NOT_FOUND,
        ))
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        let error_response = ApiErrorResponse {
            error: "Invalid JSON body".to_string(),
            code: "INVALID_JSON".to_string(),
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&error_response),
            warp::http::StatusCode::BAD_REQUEST,
        ))
    } else if let Some(_) = err.find::<warp::reject::PayloadTooLarge>() {
        let error_response = ApiErrorResponse {
            error: "Request payload too large (max 1MB)".to_string(),
            code: "PAYLOAD_TOO_LARGE".to_string(),
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&error_response),
            warp::http::StatusCode::PAYLOAD_TOO_LARGE,
        ))
    } else {
        let error_response = ApiErrorResponse {
            error: "Internal server error".to_string(),
            code: "INTERNAL_ERROR".to_string(),
        };
        Ok(warp::reply::with_status(
            warp::reply::json(&error_response),
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

