//! Codex API 服务器（透传模式）
//!
//! 提供本地网关入口，做最小处理后将请求直接透传到 ChatGPT Codex 上游。

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use hyper::{Body, Response};
use serde_json::{Value, json};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::http::{HeaderMap, Method, StatusCode};
use warp::path::FullPath;
use warp::{Filter, Rejection, Reply};

use super::{
    executor::{CodexExecutor, ForwardMeta, ForwardRequest},
    logger::RequestLogger,
    models::{CodexError, RequestLog},
    pool::CodexPool,
    storage::CodexLogStorage,
};
use crate::data::storage::common::traits::AccountStorage;
use crate::AppState;

/// Codex API 服务器
pub struct CodexServer {
    port: u16,
}

impl CodexServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct UsageStats {
    input_tokens: i64,
    output_tokens: i64,
    total_tokens: i64,
}

/// Codex API 路由
pub fn codex_routes_from_state(
    state: Arc<AppState>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    let state_filter = warp::any().map(move || state.clone());

    // 健康检查
    let health = warp::path!("health")
        .and(warp::get())
        .and(state_filter.clone())
        .map(|state: Arc<AppState>| {
            warp::reply::json(&serde_json::json!({
                "status": "ok",
                "service": "codex-api",
                "enabled": state.codex_server.lock().unwrap().is_some(),
            }))
        });

    // GET /v1/models
    let models = warp::path!("v1" / "models")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(|state: Arc<AppState>| async move {
            ensure_codex_enabled(&state)?;
            Result::<_, Rejection>::Ok(warp::reply::json(&get_models()))
        });

    // GET /pool/status
    let pool_status = warp::path!("pool" / "status")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(|state: Arc<AppState>| async move {
            ensure_codex_enabled(&state)?;
            let (pool, _, _, _) = get_runtime_or_reject(&state)?;
            Result::<_, Rejection>::Ok(warp::reply::json(&pool.status().await))
        });

    // 统一透传入口：仅处理 /v1/* 与 /backend-api/codex/*
    let passthrough = warp::any()
        .and(warp::path::full())
        .and(warp::method())
        .and(optional_raw_query())
        .and(warp::header::headers_cloned())
        .and(warp::body::content_length_limit(20 * 1024 * 1024))
        .and(warp::body::bytes())
        .and(state_filter)
        .and_then(handle_passthrough);

    health.or(models).or(pool_status).or(passthrough)
}

fn optional_raw_query() -> impl Filter<Extract = (Option<String>,), Error = Infallible> + Clone {
    warp::query::raw()
        .map(Some)
        .or(warp::any().map(|| None))
        .unify()
}

async fn handle_passthrough(
    full_path: FullPath,
    method: Method,
    query: Option<String>,
    headers: HeaderMap,
    body: Bytes,
    state: Arc<AppState>,
) -> Result<Box<dyn Reply>, Rejection> {
    let path = full_path.as_str().to_string();
    println!("[Codex Server] Incoming request: {} {}", method, path);
    if !is_supported_proxy_path(&path) {
        return Err(warp::reject::not_found());
    }

    ensure_codex_enabled(&state)?;
    validate_api_key(&state, &headers)?;

    let (pool, executor, logger, storage) = get_runtime_or_reject(&state)?;
    let request_format = infer_request_format(&path).to_string();
    let request_model = extract_model_from_json_bytes(&body).unwrap_or_else(|| "unknown".to_string());

    let (body, stream_forced) = if request_format == "openai-responses" {
        let result = normalize_responses_body(&body);
        (result.body.unwrap_or(body), result.stream_forced)
    } else {
        (body, false)
    };

    let forward_request = ForwardRequest {
        method,
        path: path.clone(),
        query,
        headers,
        body,
        format: request_format.clone(),
        model: request_model.clone(),
    };

    let (upstream_response, meta) = match executor.forward(forward_request).await {
        Ok(ok) => ok,
        Err(err) => {
            let is_no_account = matches!(err, CodexError::NoAvailableAccount);
            let err_text = err.to_string();
            add_failed_log(
                logger.clone(),
                storage.clone(),
                &request_model,
                &request_format,
                err_text.clone(),
            )
            .await;

            let rejection = if is_no_account {
                CodexRejection::NoAvailableAccount
            } else {
                CodexRejection::ExecutionError(err_text)
            };
            return Err(warp::reject::custom(rejection));
        }
    };

    let upstream_status =
        StatusCode::from_u16(upstream_response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let upstream_headers = upstream_response.headers().clone();

    // 如果是 402/403，异步更新数据库中的 forbidden 状态
    if upstream_status == StatusCode::PAYMENT_REQUIRED || upstream_status == StatusCode::FORBIDDEN {
        let state_clone = state.clone();
        let account_id = meta.account_id.clone();
        tokio::spawn(async move {
            mark_account_forbidden(&state_clone, &account_id).await;
        });
    }

    if is_event_stream(&upstream_headers) {
        // 客户端原本不要流式 → 收集 SSE 流，提取最终响应对象以 JSON 返回
        if stream_forced {
            let response = destream_responses_sse(
                upstream_status,
                upstream_response,
                pool,
                logger,
                storage,
                meta,
                request_model,
            )
            .await
            .map_err(|e| warp::reject::custom(CodexRejection::InternalError(e)))?;
            return Ok(Box::new(response) as Box<dyn Reply>);
        }

        let response = build_streaming_response_with_metrics(
            upstream_status,
            &upstream_headers,
            upstream_response,
            pool,
            logger,
            storage,
            meta,
            request_model,
        )
        .map_err(|e| warp::reject::custom(CodexRejection::InternalError(e.to_string())))?;
        return Ok(Box::new(response) as Box<dyn Reply>);
    }

    let upstream_bytes = match upstream_response.bytes().await {
        Ok(bytes) => bytes,
        Err(err) => {
            pool.record_failure(&meta.account_id, None).await;
            let err_text = format!("Failed to read upstream response body: {}", err);
            add_failed_log(
                logger,
                storage,
                &request_model,
                &request_format,
                err_text.clone(),
            )
            .await;
            return Err(warp::reject::custom(CodexRejection::ExecutionError(err_text)));
        }
    };

    let usage = extract_usage_from_json_bytes(&upstream_bytes);
    if upstream_status.is_success() && usage.total_tokens > 0 {
        pool.record_usage(&meta.account_id, usage.total_tokens).await;
    }

    let log_model = if request_model == "unknown" {
        extract_model_from_json_bytes(&upstream_bytes).unwrap_or(request_model)
    } else {
        request_model
    };
    let error_message = if upstream_status.is_success() {
        None
    } else {
        extract_error_message(&upstream_bytes)
    };
    let log = build_request_log(
        &meta,
        log_model,
        if upstream_status.is_success() {
            "success"
        } else {
            "error"
        },
        usage,
        error_message,
    );
    record_log(logger, storage, log).await;

    let response = build_buffered_response(upstream_status, &upstream_headers, upstream_bytes)
        .map_err(|e| warp::reject::custom(CodexRejection::InternalError(e.to_string())))?;

    Ok(Box::new(response) as Box<dyn Reply>)
}

/// 收集上游 SSE 流，提取 response.completed 事件中的完整响应对象，
/// 以普通 JSON 返回给不需要流式的客户端。
async fn destream_responses_sse(
    status: StatusCode,
    response: reqwest::Response,
    pool: Arc<CodexPool>,
    logger: Arc<RwLock<RequestLogger>>,
    storage: Option<Arc<CodexLogStorage>>,
    meta: ForwardMeta,
    request_model: String,
) -> Result<Response<Body>, String> {
    let mut stream = response.bytes_stream();
    let mut extractor = SseMetricsExtractor::default();
    let mut all_data = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                extractor.ingest_chunk(&bytes);
                all_data.push_str(&String::from_utf8_lossy(&bytes));
            }
            Err(err) => {
                let err_text = format!("Failed to read upstream stream: {}", err);
                pool.record_failure(&meta.account_id, None).await;
                return Err(err_text);
            }
        }
    }
    extractor.finish();

    // 记录 usage
    let usage = extractor.usage;
    if status.is_success() && usage.total_tokens > 0 {
        pool.record_usage(&meta.account_id, usage.total_tokens).await;
    }

    let log_model = if request_model == "unknown" {
        extractor.model.clone().unwrap_or(request_model.clone())
    } else {
        request_model.clone()
    };
    let error_message = if status.is_success() {
        None
    } else {
        extractor.error_message.clone()
    };
    let log = build_request_log(
        &meta,
        log_model,
        if status.is_success() { "success" } else { "error" },
        usage,
        error_message,
    );
    record_log(logger, storage, log).await;

    // 从 SSE 事件中提取 response.completed 的 response 对象
    let response_json = extract_completed_response(&all_data)
        .unwrap_or_else(|| json!({"error": "Failed to extract response from SSE stream"}));

    let body_bytes = serde_json::to_vec(&response_json)
        .map_err(|e| format!("Failed to serialize response: {}", e))?;

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(body_bytes))
        .map_err(|e| format!("Failed to build destreamed response: {}", e))
}

/// 从 SSE 文本中提取 response.completed 事件的 response 对象
fn extract_completed_response(sse_text: &str) -> Option<Value> {
    // 按双换行分割事件块
    for block in sse_text.split("\n\n") {
        let mut event_type = None;
        let mut data_lines = Vec::new();

        for line in block.lines() {
            let line = line.trim_start();
            if let Some(rest) = line.strip_prefix("event:") {
                event_type = Some(rest.trim());
            } else if let Some(rest) = line.strip_prefix("data:") {
                data_lines.push(rest.trim_start());
            }
        }

        // 查找 response.completed 事件
        if event_type == Some("response.completed") || event_type == Some("response.done") {
            if data_lines.is_empty() {
                continue;
            }
            let data = data_lines.join("\n");
            if let Ok(value) = serde_json::from_str::<Value>(&data) {
                // response.completed 事件的 data 中有 response 字段
                if let Some(resp) = value.get("response").cloned() {
                    return Some(resp);
                }
                // 如果没有 response 字段，整个 data 可能就是响应
                return Some(value);
            }
        }
    }

    // 也尝试 \r\n\r\n 分割
    for block in sse_text.split("\r\n\r\n") {
        let mut event_type = None;
        let mut data_lines = Vec::new();

        for line in block.lines() {
            let line = line.trim_start();
            if let Some(rest) = line.strip_prefix("event:") {
                event_type = Some(rest.trim());
            } else if let Some(rest) = line.strip_prefix("data:") {
                data_lines.push(rest.trim_start());
            }
        }

        if event_type == Some("response.completed") || event_type == Some("response.done") {
            if data_lines.is_empty() {
                continue;
            }
            let data = data_lines.join("\n");
            if let Ok(value) = serde_json::from_str::<Value>(&data) {
                if let Some(resp) = value.get("response").cloned() {
                    return Some(resp);
                }
                return Some(value);
            }
        }
    }

    None
}

fn build_streaming_response_with_metrics(
    status: StatusCode,
    headers: &HeaderMap,
    response: reqwest::Response,
    pool: Arc<CodexPool>,
    logger: Arc<RwLock<RequestLogger>>,
    storage: Option<Arc<CodexLogStorage>>,
    meta: ForwardMeta,
    request_model: String,
) -> Result<Response<Body>, String> {
    let mut builder = Response::builder().status(status);
    for (name, value) in headers.iter() {
        if should_strip_response_header(name.as_str()) {
            continue;
        }
        builder = builder.header(name, value);
    }

    let mut upstream_stream = response.bytes_stream();
    let (mut tx, rx) = futures::channel::mpsc::channel::<Result<Bytes, std::io::Error>>(16);

    tokio::spawn(async move {
        let mut extractor = SseMetricsExtractor::default();

        while let Some(chunk) = upstream_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    extractor.ingest_chunk(&bytes);
                    if tx.send(Ok(bytes)).await.is_err() {
                        break;
                    }
                }
                Err(err) => {
                    let err_text = format!("Failed to read upstream stream chunk: {}", err);
                    extractor.error_message.get_or_insert(err_text.clone());
                    let _ = tx
                        .send(Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            err_text,
                        )))
                        .await;
                    break;
                }
            }
        }

        extractor.finish();
        let usage = extractor.usage;
        if status.is_success() && usage.total_tokens > 0 {
            pool.record_usage(&meta.account_id, usage.total_tokens).await;
        }

        let log_model = if request_model == "unknown" {
            extractor.model.unwrap_or(request_model)
        } else {
            request_model
        };
        let error_message = if status.is_success() {
            None
        } else {
            extractor.error_message
        };
        let log = build_request_log(
            &meta,
            log_model,
            if status.is_success() {
                "success"
            } else {
                "error"
            },
            usage,
            error_message,
        );
        record_log(logger, storage, log).await;
    });

    builder
        .body(Body::wrap_stream(rx))
        .map_err(|e| format!("Failed to build streaming response: {}", e))
}

#[derive(Default)]
struct SseMetricsExtractor {
    pending: String,
    usage: UsageStats,
    model: Option<String>,
    error_message: Option<String>,
}

impl SseMetricsExtractor {
    fn ingest_chunk(&mut self, chunk: &Bytes) {
        let text = String::from_utf8_lossy(chunk);
        self.pending.push_str(&text);

        while let Some(idx) = self.pending.find("\n\n") {
            let event = self.pending[..idx].to_string();
            self.pending.drain(..idx + 2);
            self.parse_event_block(&event);
        }

        while let Some(idx) = self.pending.find("\r\n\r\n") {
            let event = self.pending[..idx].to_string();
            self.pending.drain(..idx + 4);
            self.parse_event_block(&event);
        }
    }

    fn finish(&mut self) {
        if !self.pending.trim().is_empty() {
            let event = std::mem::take(&mut self.pending);
            self.parse_event_block(&event);
        }
    }

    fn parse_event_block(&mut self, block: &str) {
        let mut data_lines = Vec::new();

        for line in block.lines() {
            let line = line.trim_start();
            if let Some(rest) = line.strip_prefix("data:") {
                data_lines.push(rest.trim_start());
            }
        }

        if data_lines.is_empty() {
            return;
        }

        let data = data_lines.join("\n");
        if data.trim() == "[DONE]" {
            return;
        }

        let Ok(value) = serde_json::from_str::<Value>(&data) else {
            return;
        };

        self.extract_fields(&value);
    }

    fn extract_fields(&mut self, value: &Value) {
        // 提取 model
        if self.model.is_none() {
            self.model = value
                .pointer("/response/model")
                .or_else(|| value.get("model"))
                .and_then(|v| v.as_str())
                .map(|v| v.to_string());
        }

        // 提取 usage - ChatGPT Codex 流式响应中 usage 在 response.completed 事件的 response.usage 下
        if let Some(usage) = value
            .pointer("/response/usage")
            .or_else(|| value.get("usage"))
        {
            let input_tokens = to_i64(
                usage
                    .get("input_tokens")
                    .or_else(|| usage.get("prompt_tokens")),
            );
            let output_tokens = to_i64(
                usage
                    .get("output_tokens")
                    .or_else(|| usage.get("completion_tokens")),
            );
            let total_tokens = {
                let explicit_total = to_i64(usage.get("total_tokens"));
                if explicit_total > 0 {
                    explicit_total
                } else {
                    input_tokens + output_tokens
                }
            };

            self.usage = UsageStats {
                input_tokens,
                output_tokens,
                total_tokens,
            };
        }

        // 提取错误信息
        if self.error_message.is_none() {
            self.error_message = value
                .pointer("/response/error/message")
                .or_else(|| value.pointer("/error/message"))
                .or_else(|| value.get("message"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|v| v.to_string());
        }
    }
}

fn build_buffered_response(
    status: StatusCode,
    headers: &HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, String> {
    let mut builder = Response::builder().status(status);
    for (name, value) in headers.iter() {
        if should_strip_response_header(name.as_str()) {
            continue;
        }
        builder = builder.header(name, value);
    }

    builder
        .body(Body::from(body))
        .map_err(|e| format!("Failed to build response: {}", e))
}

fn should_strip_response_header(header_name: &str) -> bool {
    matches!(
        header_name.to_ascii_lowercase().as_str(),
        "content-length"
            | "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}

fn is_event_stream(headers: &HeaderMap) -> bool {
    // ChatGPT Codex API 默认返回 SSE 格式，即使没有设置 content-type
    headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_ascii_lowercase().contains("text/event-stream"))
        .unwrap_or(true) // 当 content-type 缺失时，默认假设为 SSE
}

fn is_supported_proxy_path(path: &str) -> bool {
    path == "/v1"
        || path.starts_with("/v1/")
        || path == "/backend-api/codex"
        || path.starts_with("/backend-api/codex/")
}

fn infer_request_format(path: &str) -> &'static str {
    if path.ends_with("/chat/completions") {
        "openai-chat"
    } else if path.ends_with("/messages") {
        "claude"
    } else if path.contains("/responses") {
        "openai-responses"
    } else {
        "passthrough"
    }
}

/// 规范化 Responses API 请求体的结果
struct NormalizeResult {
    /// 规范化后的 body（None 表示无需修改）
    body: Option<Bytes>,
    /// 是否强制将 stream 从 false/缺失 改为 true
    stream_forced: bool,
}

/// 规范化 Responses API 请求体，使其兼容 ChatGPT Codex 后端：
/// - 将字符串 `input` 转换为消息对象数组
/// - 补充缺失的 `instructions` 字段
/// - 移除不支持的参数
/// - 强制 `stream: true`
fn normalize_responses_body(body: &Bytes) -> NormalizeResult {
    let Some(mut root) = serde_json::from_slice::<Value>(body).ok() else {
        return NormalizeResult { body: None, stream_forced: false };
    };
    let Some(obj) = root.as_object_mut() else {
        return NormalizeResult { body: None, stream_forced: false };
    };
    let mut modified = false;
    let mut stream_forced = false;

    // input 字符串 → 数组
    if let Some(input) = obj.get("input") {
        if let Some(text) = input.as_str() {
            let text = text.to_string();
            obj.insert(
                "input".to_string(),
                json!([{
                    "role": "user",
                    "content": [{"type": "input_text", "text": text}]
                }]),
            );
            modified = true;
        }
    }

    // 补充缺失的 instructions
    if !obj.contains_key("instructions") {
        obj.insert(
            "instructions".to_string(),
            json!("You are a helpful assistant."),
        );
        modified = true;
    }

    // 移除 ChatGPT Codex 后端不支持的参数
    const UNSUPPORTED_PARAMS: &[&str] = &["max_output_tokens"];
    for key in UNSUPPORTED_PARAMS {
        if obj.remove(*key).is_some() {
            modified = true;
        }
    }

    // ChatGPT Codex 后端要求 stream 必须为 true
    match obj.get("stream") {
        Some(v) if v.as_bool() == Some(true) => {}
        _ => {
            obj.insert("stream".to_string(), json!(true));
            stream_forced = true;
            modified = true;
        }
    }

    let new_body = if modified {
        serde_json::to_vec(&root).ok().map(Bytes::from)
    } else {
        None
    };

    NormalizeResult { body: new_body, stream_forced }
}

fn extract_model_from_json_bytes(body: &Bytes) -> Option<String> {
    serde_json::from_slice::<Value>(body)
        .ok()
        .and_then(|v| v.get("model").and_then(|m| m.as_str()).map(|m| m.to_string()))
}

fn extract_usage_from_json_bytes(body: &Bytes) -> UsageStats {
    let Ok(root) = serde_json::from_slice::<Value>(body) else {
        return UsageStats::default();
    };

    let Some(usage) = root
        .get("usage")
        .or_else(|| root.get("response").and_then(|v| v.get("usage")))
    else {
        return UsageStats::default();
    };

    let input_tokens = to_i64(
        usage
            .get("input_tokens")
            .or_else(|| usage.get("prompt_tokens")),
    );
    let output_tokens = to_i64(
        usage
            .get("output_tokens")
            .or_else(|| usage.get("completion_tokens")),
    );
    let total_tokens = {
        let explicit_total = to_i64(usage.get("total_tokens"));
        if explicit_total > 0 {
            explicit_total
        } else {
            input_tokens + output_tokens
        }
    };

    UsageStats {
        input_tokens,
        output_tokens,
        total_tokens,
    }
}

fn extract_error_message(body: &Bytes) -> Option<String> {
    if let Ok(root) = serde_json::from_slice::<Value>(body) {
        if let Some(msg) = root
            .pointer("/error/message")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            return Some(msg.to_string());
        }

        if let Some(msg) = root
            .get("message")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            return Some(msg.to_string());
        }

        if let Some(msg) = root
            .get("detail")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            return Some(msg.to_string());
        }
    }

    let text = String::from_utf8_lossy(body).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text.chars().take(300).collect())
    }
}

fn to_i64(value: Option<&Value>) -> i64 {
    value
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_u64().and_then(|n| i64::try_from(n).ok()))
        })
        .unwrap_or(0)
}

fn build_request_log(
    meta: &ForwardMeta,
    model: String,
    status: &str,
    usage: UsageStats,
    error_message: Option<String>,
) -> RequestLog {
    RequestLog {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        account_id: meta.account_id.clone(),
        account_email: meta.account_email.clone(),
        model,
        format: meta.format.clone(),
        input_tokens: usage.input_tokens,
        output_tokens: usage.output_tokens,
        total_tokens: usage.total_tokens,
        status: status.to_string(),
        error_message,
        request_duration_ms: Some(meta.started_at.elapsed().as_millis() as i64),
    }
}

async fn record_log(logger: Arc<RwLock<RequestLogger>>, storage: Option<Arc<CodexLogStorage>>, log: RequestLog) {
    let mut guard = logger.write().await;
    guard.add_log(log.clone());

    // 同时写入 SQLite 存储
    if let Some(s) = storage {
        s.add_log(log).await;
    }
}

/// 更新账户的 forbidden 状态到数据库
async fn mark_account_forbidden(state: &Arc<AppState>, account_id: &str) {
    let storage = {
        let guard = state.openai_storage_manager.lock().unwrap();
        guard.clone()
    };
    let Some(storage) = storage else {
        return;
    };

    // 获取账户并更新 quota.is_forbidden
    if let Ok(Some(mut account)) = storage.get_account(account_id).await {
        if let Some(ref mut quota) = account.quota {
            quota.is_forbidden = true;
        } else {
            let mut quota = crate::platforms::openai::models::QuotaData::new();
            quota.is_forbidden = true;
            account.quota = Some(quota);
        }
        let _ = storage.update_account(&account).await;
    }
}

async fn add_failed_log(
    logger: Arc<RwLock<RequestLogger>>,
    storage: Option<Arc<CodexLogStorage>>,
    model: &str,
    format: &str,
    error: String,
) {
    let log = RequestLog {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        account_id: String::new(),
        account_email: String::new(),
        model: model.to_string(),
        format: format.to_string(),
        input_tokens: 0,
        output_tokens: 0,
        total_tokens: 0,
        status: "error".to_string(),
        error_message: Some(error),
        request_duration_ms: None,
    };
    let mut guard = logger.write().await;
    guard.add_log(log.clone());

    // 同时写入 SQLite 存储
    if let Some(s) = storage {
        s.add_log(log).await;
    }
}

fn ensure_codex_enabled(state: &Arc<AppState>) -> Result<(), Rejection> {
    if state.codex_server.lock().unwrap().is_some() {
        return Ok(());
    }

    Err(warp::reject::custom(CodexRejection::ServiceUnavailable(
        "Codex service is disabled".to_string(),
    )))
}

fn get_runtime_or_reject(
    state: &Arc<AppState>,
) -> Result<
    (
        Arc<CodexPool>,
        Arc<CodexExecutor>,
        Arc<RwLock<RequestLogger>>,
        Option<Arc<CodexLogStorage>>,
    ),
    Rejection,
> {
    let pool = state.codex_pool.lock().unwrap().clone().ok_or_else(|| {
        warp::reject::custom(CodexRejection::ServiceUnavailable(
            "Codex pool is not initialized".to_string(),
        ))
    })?;

    let executor = state
        .codex_executor
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| {
            warp::reject::custom(CodexRejection::ServiceUnavailable(
                "Codex executor is not initialized".to_string(),
            ))
        })?;

    let logger = state.codex_logger.lock().unwrap().clone().ok_or_else(|| {
        warp::reject::custom(CodexRejection::ServiceUnavailable(
            "Codex logger is not initialized".to_string(),
        ))
    })?;

    let storage = state.codex_log_storage.lock().unwrap().clone();

    Ok((pool, executor, logger, storage))
}

fn validate_api_key(state: &Arc<AppState>, headers: &HeaderMap) -> Result<(), Rejection> {
    let configured_key = state
        .codex_server_config
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|cfg| cfg.api_key.as_ref())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    // API Key 必须设置，未设置则拒绝请求
    let Some(expected) = configured_key else {
        return Err(warp::reject::custom(CodexRejection::ExecutionError(
            "Unauthorized: API key not configured".to_string(),
        )));
    };

    let mut candidates: Vec<&str> = Vec::new();

    if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(token) = extract_bearer_token(auth) {
            candidates.push(token);
        }
    }

    if let Some(key) = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        candidates.push(key);
    }

    if candidates.into_iter().any(|provided| provided == expected) {
        return Ok(());
    }

    Err(warp::reject::custom(CodexRejection::ExecutionError(
        "Unauthorized: invalid API key".to_string(),
    )))
}

fn extract_bearer_token(header: &str) -> Option<&str> {
    let trimmed = header.trim();
    if trimmed.len() < 7 {
        return None;
    }

    let (scheme, rest) = trimmed.split_at(7);
    if !scheme.eq_ignore_ascii_case("bearer ") {
        return None;
    }

    let token = rest.trim();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

fn get_models() -> serde_json::Value {
    json!({
        "object": "list",
        "data": [
            {
                "id": "gpt-5",
                "object": "model",
                "created": 1728000000,
                "owned_by": "openai"
            },
            {
                "id": "gpt-5-codex",
                "object": "model",
                "created": 1728000000,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4.1",
                "object": "model",
                "created": 1728000000,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4o",
                "object": "model",
                "created": 1728000000,
                "owned_by": "openai"
            }
        ]
    })
}

// ==================== 错误类型 ====================

#[derive(Debug)]
pub enum CodexRejection {
    NoAvailableAccount,
    InvalidRequest(String),
    TranslationError(String),
    ExecutionError(String),
    ServiceUnavailable(String),
    InternalError(String),
}

impl warp::reject::Reject for CodexRejection {}
