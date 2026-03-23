use crate::AppState;
use crate::http_client;
use super::outlook_storage::OutlookStorage;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use imap::Session;
use native_tls::TlsStream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::Arc;
use tauri::State;

// XOAUTH2 认证器
struct XOAuth2 {
    user: String,
    access_token: String,
}

impl imap::Authenticator for XOAuth2 {
    type Response = String;

    fn process(&self, _data: &[u8]) -> Self::Response {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}

// 数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlookCredentials {
    pub email: String,
    pub refresh_token: String,
    pub client_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailItem {
    pub message_id: String,
    pub folder: String,
    pub subject: String,
    pub from_email: String,
    pub date: String,
    pub is_read: bool,
    pub has_attachments: bool,
    pub sender_initial: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailListResponse {
    pub email_id: String,
    pub folder_view: String,
    pub page: i32,
    pub page_size: i32,
    pub total_emails: i32,
    pub emails: Vec<EmailItem>,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDetailsResponse {
    pub message_id: String,
    pub subject: String,
    pub from_email: String,
    pub to_email: String,
    pub cc_email: Option<String>,
    pub date: String,
    pub body_plain: Option<String>,
    pub body_html: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatus {
    pub email: String,
    pub status: String, // "active", "inactive", "unknown"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResult {
    pub email: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRefreshResponse {
    pub total: i32,
    pub success_count: i32,
    pub failed_count: i32,
    pub results: Vec<RefreshResult>,
}

// OAuth2 令牌响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: Option<String>,
}

// Graph API 响应结构
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GraphEmailAddress {
    address: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GraphEmailSender {
    #[serde(rename = "emailAddress")]
    email_address: Option<GraphEmailAddress>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GraphEmailBody {
    #[serde(rename = "contentType")]
    content_type: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GraphMessage {
    id: Option<String>,
    subject: Option<String>,
    body: Option<GraphEmailBody>,
    from: Option<GraphEmailSender>,
    #[serde(rename = "toRecipients")]
    to_recipients: Option<Vec<GraphEmailSender>>,
    #[serde(rename = "ccRecipients")]
    cc_recipients: Option<Vec<GraphEmailSender>>,
    #[serde(rename = "receivedDateTime")]
    received_date_time: Option<String>,
    #[serde(rename = "isRead")]
    is_read: Option<bool>,
    #[serde(rename = "hasAttachments")]
    has_attachments: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GraphMessagesResponse {
    value: Option<Vec<GraphMessage>>,
    #[serde(rename = "@odata.count")]
    odata_count: Option<i32>,
    #[serde(rename = "@odata.nextLink")]
    odata_next_link: Option<String>,
}

// 简化的邮件管理器
pub struct OutlookManager {
    pub(crate) credentials: HashMap<String, OutlookCredentials>,
}

// 获取 Outlook 存储
fn get_outlook_storage(state: &State<'_, AppState>) -> Result<Arc<OutlookStorage>, String> {
    state
        .outlook_storage
        .lock()
        .map_err(|_| "Failed to access Outlook storage state".to_string())?
        .clone()
        .ok_or_else(|| "Outlook storage not initialized".to_string())
}

// 确保内存中已加载凭证（懒加载）
fn ensure_loaded(state: &State<'_, AppState>) {
    let mut manager = state.outlook_manager.lock().unwrap();
    if manager.is_empty() {
        if let Ok(storage) = get_outlook_storage(state) {
            let _ = manager.load_from_storage(&storage);
        }
    }
}

// 持久化新的 refresh_token（内存 + SQLite）
fn persist_new_refresh_token(
    state: &AppState,
    email: &str,
    new_refresh_token: &str,
) {
    // 更新内存
    {
        let mut manager = state.outlook_manager.lock().unwrap();
        if let Some(cred) = manager.credentials.get_mut(email) {
            cred.refresh_token = new_refresh_token.to_string();
        }
    }
    // 更新 SQLite
    if let Ok(storage) = state
        .outlook_storage
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .ok_or(())
    {
        if let Err(e) = storage.update_refresh_token(email, new_refresh_token) {
            eprintln!("[outlook] Failed to persist new refresh_token for {}: {}", email, e);
        }
    }
}

// 后台定时刷新所有 Token（供 lib.rs spawn 调用）
pub async fn scheduled_refresh_tokens(state: &AppState) {
    // 确保内存中已加载
    {
        let mut manager = state.outlook_manager.lock().unwrap();
        if manager.is_empty() {
            if let Ok(storage) = state
                .outlook_storage
                .lock()
                .ok()
                .and_then(|g| g.clone())
                .ok_or(())
            {
                let _ = manager.load_from_storage(&storage);
            }
        }
    }

    let all_credentials: Vec<OutlookCredentials> = {
        let manager = state.outlook_manager.lock().unwrap();
        manager.credentials.values().cloned().collect()
    };

    if all_credentials.is_empty() {
        return;
    }

    eprintln!(
        "[outlook] Scheduled token refresh starting for {} accounts",
        all_credentials.len()
    );

    let temp_manager = OutlookManager::new();
    let mut success = 0;
    let mut failed = 0;

    for (i, cred) in all_credentials.iter().enumerate() {
        match temp_manager.get_graph_access_token(cred).await {
            Ok((_access_token, new_rt)) => {
                if let Some(ref rt) = new_rt {
                    persist_new_refresh_token(state, &cred.email, rt);
                }
                success += 1;
            }
            Err(_) => match temp_manager.get_access_token(cred).await {
                Ok((_access_token, _server, new_rt)) => {
                    if let Some(ref rt) = new_rt {
                        persist_new_refresh_token(state, &cred.email, rt);
                    }
                    success += 1;
                }
                Err(e) => {
                    failed += 1;
                    eprintln!(
                        "[outlook] Scheduled refresh failed for {}: {}",
                        cred.email, e
                    );
                }
            },
        }
        // 间隔 3 秒，防止微软限流（最后一个不延迟）
        if i + 1 < all_credentials.len() {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    eprintln!(
        "[outlook] Scheduled token refresh complete: {}/{} success, {} failed",
        success,
        all_credentials.len(),
        failed
    );
}

// Outlook 邮箱管理命令
#[tauri::command]
pub async fn outlook_save_credentials(
    email: String,
    refresh_token: String,
    client_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 持久化到 SQLite
    let storage = get_outlook_storage(&state)?;
    storage.save(&email, &refresh_token, &client_id)?;

    // 同步到内存
    let credentials = OutlookCredentials {
        email,
        refresh_token,
        client_id,
        created_at: chrono::Utc::now(),
    };

    let mut manager = state.outlook_manager.lock().unwrap();
    manager.save_credentials(credentials)
}

#[tauri::command]
pub async fn outlook_get_all_accounts_info(
    state: State<'_, AppState>,
) -> Result<Vec<AccountInfo>, String> {
    ensure_loaded(&state);
    let manager = state.outlook_manager.lock().unwrap();
    manager.get_all_accounts_info()
}

#[tauri::command]
pub async fn outlook_get_account_statuses(
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let storage = get_outlook_storage(&state)?;
    let statuses = storage.load_all_statuses()?;
    Ok(statuses.into_iter().collect())
}

#[tauri::command]
pub async fn outlook_delete_account(
    email: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    // 从 SQLite 删除
    if let Ok(storage) = get_outlook_storage(&state) {
        let _ = storage.delete(&email);
    }

    // 从内存删除
    let mut manager = state.outlook_manager.lock().unwrap();
    manager.delete_account(&email)
}

#[tauri::command]
pub async fn outlook_check_account_status(
    email: String,
    state: State<'_, AppState>,
) -> Result<AccountStatus, String> {
    ensure_loaded(&state);
    let credentials = {
        let manager = state.outlook_manager.lock().unwrap();
        manager.get_credentials(&email)?
    };

    let temp_manager = OutlookManager::new();
    let result = temp_manager
        .check_account_status_with_credentials(&credentials)
        .await?;

    // 持久化状态
    if let Ok(storage) = get_outlook_storage(&state) {
        let _ = storage.update_status(&email, &result.status);
    }

    Ok(result)
}

#[tauri::command]
pub async fn outlook_get_emails(
    email: String,
    folder: String,
    page: i32,
    page_size: i32,
    state: State<'_, AppState>,
) -> Result<EmailListResponse, String> {
    ensure_loaded(&state);
    let credentials = {
        let manager = state.outlook_manager.lock().unwrap();
        manager.get_credentials(&email)?
    };

    let temp_manager = OutlookManager::new();

    // 自动回退: Graph API → IMAP
    match temp_manager
        .graph_get_emails_with_credentials(&credentials, &folder, page, page_size)
        .await
    {
        Ok(response) => return Ok(response),
        Err(graph_err) => {
            eprintln!("[outlook] Graph API failed for {}: {}, falling back to IMAP", email, graph_err);
            match temp_manager
                .get_emails_with_credentials(&credentials, &folder, page, page_size)
                .await
            {
                Ok(response) => Ok(response),
                Err(imap_err) => Err(format!(
                    "All methods failed. Graph: {}; IMAP: {}",
                    graph_err, imap_err
                )),
            }
        }
    }
}

#[tauri::command]
pub async fn outlook_get_email_details(
    email: String,
    message_id: String,
    method: Option<String>,
    state: State<'_, AppState>,
) -> Result<EmailDetailsResponse, String> {
    ensure_loaded(&state);
    let credentials = {
        let manager = state.outlook_manager.lock().unwrap();
        manager.get_credentials(&email)?
    };

    let temp_manager = OutlookManager::new();

    // 根据 method 提示选择 API，默认 graph
    let use_method = method.unwrap_or_else(|| {
        // 自动检测: IMAP message_id 格式为 "folder-number"
        if let Some(pos) = message_id.find('-') {
            if message_id[pos + 1..].parse::<u32>().is_ok() {
                return "imap".to_string();
            }
        }
        "graph".to_string()
    });

    if use_method == "imap" {
        temp_manager
            .get_email_details_with_credentials(&credentials, &message_id)
            .await
    } else {
        temp_manager
            .graph_get_email_details_with_credentials(&credentials, &message_id)
            .await
    }
}

#[tauri::command]
pub async fn outlook_refresh_all_tokens(
    emails: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<BatchRefreshResponse, String> {
    ensure_loaded(&state);
    let all_credentials: Vec<OutlookCredentials> = {
        let manager = state.outlook_manager.lock().unwrap();
        match &emails {
            Some(list) => manager
                .credentials
                .values()
                .filter(|c| list.contains(&c.email))
                .cloned()
                .collect(),
            None => manager.credentials.values().cloned().collect(),
        }
    };

    let total = all_credentials.len() as i32;
    let mut success_count = 0i32;
    let mut failed_count = 0i32;
    let mut results = Vec::new();

    let temp_manager = OutlookManager::new();

    for (i, cred) in all_credentials.iter().enumerate() {
        // 尝试获取 access token 来验证 refresh token 是否有效
        let (success, error_msg) = match temp_manager.get_graph_access_token(cred).await {
            Ok((_access_token, new_rt)) => {
                if let Some(ref rt) = new_rt {
                    persist_new_refresh_token(state.inner(), &cred.email, rt);
                }
                (true, None)
            }
            Err(graph_err) => {
                // Graph 失败，再尝试 IMAP token
                match temp_manager.get_access_token(cred).await {
                    Ok((_access_token, _server, new_rt)) => {
                        if let Some(ref rt) = new_rt {
                            persist_new_refresh_token(state.inner(), &cred.email, rt);
                        }
                        (true, None)
                    }
                    Err(imap_err) => {
                        (false, Some(format!("Graph: {}; IMAP: {}", graph_err, imap_err)))
                    }
                }
            }
        };

        // 持久化状态到 SQLite
        let status_str = if success {
            "active"
        } else if error_msg.as_deref().map_or(false, |e| e.contains("[BANNED]")) {
            "banned"
        } else {
            "inactive"
        };
        if let Ok(storage) = get_outlook_storage(&state) {
            let _ = storage.update_status(&cred.email, status_str);
        }

        if success {
            success_count += 1;
        } else {
            failed_count += 1;
        }
        results.push(RefreshResult {
            email: cred.email.clone(),
            success,
            error: error_msg,
        });
        // 间隔 3 秒，防止微软限流（最后一个不延迟）
        if i + 1 < all_credentials.len() {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    Ok(BatchRefreshResponse {
        total,
        success_count,
        failed_count,
        results,
    })
}

// ==================== OAuth2 Authorization Code Flow ====================

const OAUTH_CLIENT_ID: Option<&str> = option_env!("ATM_OUTLOOK_CLIENT_ID");
const OAUTH_REDIRECT_URI: &str = "http://localhost:8080";
const OAUTH_SCOPES: &str = "offline_access https://graph.microsoft.com/Mail.Read https://graph.microsoft.com/Mail.ReadWrite https://graph.microsoft.com/User.Read";

/// Single in-flight Outlook OAuth: CSRF `state` + PKCE `code_verifier` (RFC 7636).
#[derive(Clone)]
pub struct OutlookOAuthPending {
    pub state: String,
    pub code_verifier: String,
}

fn generate_pkce_verifier() -> String {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn pkce_challenge_s256(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

#[tauri::command]
pub fn outlook_oauth_available() -> bool {
    OAUTH_CLIENT_ID.is_some()
}

#[derive(Serialize)]
pub struct OAuthAuthUrl {
    pub auth_url: String,
    pub client_id: String,
    pub redirect_uri: String,
}

#[derive(Serialize)]
pub struct OAuthTokenResult {
    pub refresh_token: String,
    pub client_id: String,
    pub email: Option<String>,
}

fn resolve_oauth_client_id(custom_client_id: Option<String>) -> Result<String, String> {
    custom_client_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or(OAUTH_CLIENT_ID)
        .map(String::from)
        .ok_or_else(|| "OAUTH_CLIENT_ID_NOT_CONFIGURED".to_string())
}

#[tauri::command]
pub async fn outlook_get_oauth_auth_url(
    custom_client_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<OAuthAuthUrl, String> {
    let client_id = resolve_oauth_client_id(custom_client_id)?;

    let oauth_state = uuid::Uuid::new_v4().simple().to_string();
    let code_verifier = generate_pkce_verifier();
    let code_challenge = pkce_challenge_s256(&code_verifier);

    *state.outlook_oauth_pending.lock().unwrap() = Some(OutlookOAuthPending {
        state: oauth_state.clone(),
        code_verifier,
    });

    let auth_url = format!(
        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope={}&state={}&code_challenge_method=S256&code_challenge={}",
        client_id,
        urlencoding::encode(OAUTH_REDIRECT_URI),
        urlencoding::encode(OAUTH_SCOPES),
        urlencoding::encode(&oauth_state),
        urlencoding::encode(&code_challenge),
    );

    Ok(OAuthAuthUrl {
        auth_url,
        client_id,
        redirect_uri: OAUTH_REDIRECT_URI.to_string(),
    })
}

#[tauri::command]
pub async fn outlook_exchange_oauth_token(
    redirected_url: String,
    custom_client_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<OAuthTokenResult, String> {
    let client_id = resolve_oauth_client_id(custom_client_id)?;

    let url = url::Url::parse(&redirected_url)
        .map_err(|e| format!("Failed to parse URL: {}", e))?;

    let returned_state = url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.to_string());

    let code_verifier = {
        let mut pending_lock = state.outlook_oauth_pending.lock().unwrap();
        let pending = pending_lock.take();
        match (returned_state, pending) {
            (None, _) => return Err("OAUTH_STATE_MISSING".to_string()),
            (Some(_), None) => return Err("OAUTH_STATE_MISMATCH".to_string()),
            (Some(got), Some(p)) if got != p.state => {
                return Err("OAUTH_STATE_MISMATCH".to_string());
            }
            (Some(_), Some(p)) => p.code_verifier,
        }
    };

    let code = url
        .query_pairs()
        .find(|(k, _)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| "Authorization code not found in URL".to_string())?;

    let client = http_client::create_proxy_client()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let token_url = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
    let params = [
        ("client_id", client_id.as_str()),
        ("code", code.as_str()),
        ("redirect_uri", OAUTH_REDIRECT_URI),
        ("grant_type", "authorization_code"),
        ("scope", OAUTH_SCOPES),
        ("code_verifier", code_verifier.as_str()),
    ];

    let response = client
        .post(token_url)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token request failed: {}", e))?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed: {}", &body[..body.len().min(300)]));
    }

    let token_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    let refresh_token = token_data["refresh_token"]
        .as_str()
        .ok_or_else(|| "Response does not contain refresh_token".to_string())?
        .to_string();

    let email = if let Some(access_token) = token_data["access_token"].as_str() {
        let me_client = http_client::create_proxy_client()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        match me_client
            .get("https://graph.microsoft.com/v1.0/me")
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let me: serde_json::Value = resp.json().await.unwrap_or_default();
                me["mail"].as_str()
                    .or_else(|| me["userPrincipalName"].as_str())
                    .map(|s| s.to_string())
            }
            _ => None,
        }
    } else {
        None
    };

    Ok(OAuthTokenResult {
        refresh_token,
        client_id,
        email,
    })
}

// ==================== 邮件删除 ====================

#[derive(Serialize)]
pub struct DeleteEmailsResponse {
    pub success_count: i32,
    pub failed_count: i32,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn outlook_delete_emails(
    email: String,
    message_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<DeleteEmailsResponse, String> {
    ensure_loaded(&state);
    let credentials = {
        let manager = state.outlook_manager.lock().unwrap();
        manager.get_credentials(&email)?
    };

    let temp_manager = OutlookManager::new();
    let (access_token, _new_rt) = temp_manager
        .get_graph_access_token(&credentials)
        .await
        .map_err(|e| format!("获取 Graph Token 失败: {}", e))?;

    if let Some(ref rt) = _new_rt {
        persist_new_refresh_token(state.inner(), &email, rt);
    }

    let client = http_client::create_proxy_client()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let mut success_count = 0i32;
    let mut failed_count = 0i32;
    let mut errors = Vec::new();

    // Graph API $batch 每批最多 20 个请求
    for chunk in message_ids.chunks(20) {
        let requests: Vec<serde_json::Value> = chunk
            .iter()
            .enumerate()
            .map(|(idx, msg_id)| {
                serde_json::json!({
                    "id": idx.to_string(),
                    "method": "DELETE",
                    "url": format!("/me/messages/{}", msg_id)
                })
            })
            .collect();

        let batch_body = serde_json::json!({ "requests": requests });

        match client
            .post("https://graph.microsoft.com/v1.0/$batch")
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Content-Type", "application/json")
            .body(batch_body.to_string())
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                let body: serde_json::Value = response.json().await.unwrap_or_default();
                if let Some(responses) = body["responses"].as_array() {
                    for res in responses {
                        let status = res["status"].as_u64().unwrap_or(0);
                        if status == 200 || status == 204 {
                            success_count += 1;
                        } else {
                            failed_count += 1;
                            let id_idx = res["id"].as_str().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
                            let msg_id = chunk.get(id_idx).map(|s| s.as_str()).unwrap_or("?");
                            errors.push(format!("ID {}: status {}", msg_id, status));
                        }
                    }
                }
            }
            Ok(response) => {
                let status = response.status();
                failed_count += chunk.len() as i32;
                errors.push(format!("Batch 请求失败: {}", status));
            }
            Err(e) => {
                failed_count += chunk.len() as i32;
                errors.push(format!("网络错误: {}", e));
            }
        }
    }

    Ok(DeleteEmailsResponse {
        success_count,
        failed_count,
        errors,
    })
}

impl OutlookManager {
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
        }
    }

    // 是否为空
    pub fn is_empty(&self) -> bool {
        self.credentials.is_empty()
    }

    // 从 SQLite 存储加载凭证到内存
    pub fn load_from_storage(&mut self, storage: &OutlookStorage) -> Result<(), String> {
        let records = storage.load_all()?;
        for record in records {
            let created_at = chrono::DateTime::parse_from_rfc3339(&record.created_at)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            let credentials = OutlookCredentials {
                email: record.email.clone(),
                refresh_token: record.refresh_token,
                client_id: record.client_id,
                created_at,
            };
            self.credentials.insert(record.email, credentials);
        }
        Ok(())
    }

    // 保存账户凭证（内存中）
    pub fn save_credentials(&mut self, credentials: OutlookCredentials) -> Result<(), String> {
        self.credentials
            .insert(credentials.email.clone(), credentials);
        Ok(())
    }

    // 获取账户凭证
    pub fn get_credentials(&self, email: &str) -> Result<OutlookCredentials, String> {
        self.credentials
            .get(email)
            .cloned()
            .ok_or_else(|| format!("Account not found: {}", email))
    }

    // 获取所有账户详细信息（按添加时间排序）
    pub fn get_all_accounts_info(&self) -> Result<Vec<AccountInfo>, String> {
        let mut accounts: Vec<_> = self.credentials.iter().collect();
        // 按创建时间排序（最新的在前）
        accounts.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));
        Ok(accounts
            .into_iter()
            .map(|(email, creds)| AccountInfo {
                email: email.clone(),
                created_at: creds.created_at,
            })
            .collect())
    }

    // 删除账户
    pub fn delete_account(&mut self, email: &str) -> Result<bool, String> {
        Ok(self.credentials.remove(email).is_some())
    }

    // 获取访问令牌（多端点降级，每次重新获取）
    // 返回 (access_token, imap_server, Option<new_refresh_token>)
    pub async fn get_access_token(
        &self,
        credentials: &OutlookCredentials,
    ) -> Result<(String, String, Option<String>), String> {
        let client = http_client::create_proxy_client()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let mut errors: Vec<String> = Vec::new();

        // 端点 1: login.live.com (无 scope)
        {
            let token_url = "https://login.live.com/oauth20_token.srf";
            let params = [
                ("client_id", credentials.client_id.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", credentials.refresh_token.as_str()),
            ];
            match client.post(token_url).form(&params).send().await {
                Ok(response) if response.status().is_success() => {
                    match response.json::<TokenResponse>().await {
                        Ok(tr) => {
                            return Ok((
                                tr.access_token,
                                "outlook.office365.com".to_string(),
                                tr.refresh_token,
                            ))
                        }
                        Err(e) => {
                            errors.push(format!("Parse token from {} failed: {}", token_url, e));
                        }
                    }
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    if body.contains("service abuse") || body.contains("account is found to be") {
                        return Err(format!("[BANNED] 账号被封禁 (service abuse mode)"));
                    }
                    errors.push(format!("Token request to {} failed: {} - {}", token_url, status, &body[..body.len().min(150)]));
                }
                Err(e) => {
                    errors.push(format!("HTTP request to {} failed: {}", token_url, e));
                }
            }
        }

        // 端点 2: login.microsoftonline.com/consumers (带 IMAP scope)
        {
            let token_url =
                "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
            let params = [
                ("client_id", credentials.client_id.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", credentials.refresh_token.as_str()),
                (
                    "scope",
                    "https://outlook.office.com/IMAP.AccessAsUser.All offline_access",
                ),
            ];
            match client.post(token_url).form(&params).send().await {
                Ok(response) if response.status().is_success() => {
                    match response.json::<TokenResponse>().await {
                        Ok(tr) => {
                            return Ok((tr.access_token, "outlook.live.com".to_string(), tr.refresh_token))
                        }
                        Err(e) => {
                            errors.push(format!("Parse token from {} failed: {}", token_url, e));
                        }
                    }
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    if body.contains("service abuse") || body.contains("account is found to be") {
                        return Err(format!("[BANNED] 账号被封禁 (service abuse mode)"));
                    }
                    errors.push(format!("Token request to {} failed: {} - {}", token_url, status, &body[..body.len().min(150)]));
                }
                Err(e) => {
                    errors.push(format!("HTTP request to {} failed: {}", token_url, e));
                }
            }
        }

        Err(format!("All token endpoints failed: {}", errors.join("; ")))
    }

    // 验证账户状态
    #[allow(dead_code)]
    pub async fn check_account_status(&self, email: &str) -> Result<AccountStatus, String> {
        let credentials = self.get_credentials(email)?;
        self.check_account_status_with_credentials(&credentials)
            .await
    }

    // 使用凭证验证账户状态（避免跨 await 持有锁）
    pub async fn check_account_status_with_credentials(
        &self,
        credentials: &OutlookCredentials,
    ) -> Result<AccountStatus, String> {
        // 先尝试 Graph，再尝试 IMAP
        let is_active = self.get_graph_access_token(credentials).await.is_ok()
            || self.get_access_token(credentials).await.is_ok();
        Ok(AccountStatus {
            email: credentials.email.clone(),
            status: if is_active { "active" } else { "inactive" }.to_string(),
        })
    }

    // 创建 IMAP 连接（每次新建）
    async fn create_imap_connection(
        &self,
        credentials: &OutlookCredentials,
    ) -> Result<Session<TlsStream<TcpStream>>, String> {
        let (access_token, imap_server, _new_rt) = self.get_access_token(credentials).await?;

        // 在异步上下文中运行同步IMAP代码
        let email = credentials.email.clone();
        tokio::task::spawn_blocking(move || {
            let tls = native_tls::TlsConnector::builder()
                .build()
                .map_err(|e| format!("TLS connector failed: {}", e))?;

            let client = imap::connect(
                (imap_server.as_str(), 993),
                &imap_server,
                &tls,
            )
            .map_err(|e| format!("IMAP connect failed: {}", e))?;

            // XOAUTH2 认证
            let auth = XOAuth2 {
                user: email,
                access_token,
            };
            let session = client
                .authenticate("XOAUTH2", &auth)
                .map_err(|e| format!("IMAP authentication failed: {:?}", e))?;

            Ok(session)
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
    }

    // 获取邮件详情
    #[allow(dead_code)]
    pub async fn get_email_details(
        &self,
        email: &str,
        message_id: &str,
    ) -> Result<EmailDetailsResponse, String> {
        let credentials = self.get_credentials(email)?;
        self.get_email_details_with_credentials(&credentials, message_id)
            .await
    }

    // 使用凭证获取邮件详情（避免跨 await 持有锁）
    pub async fn get_email_details_with_credentials(
        &self,
        credentials: &OutlookCredentials,
        message_id: &str,
    ) -> Result<EmailDetailsResponse, String> {
        let (access_token, imap_server, _new_rt) = self.get_access_token(credentials).await?;

        // 解析 message_id (格式: folder-id)
        let parts: Vec<&str> = message_id.split('-').collect();
        if parts.len() != 2 {
            return Err("Invalid message_id format".to_string());
        }
        let folder_name = parts[0].to_string();
        let msg_id = parts[1].to_string();

        let email_clone = credentials.email.clone();
        let message_id_clone = message_id.to_string();

        tokio::task::spawn_blocking(move || {
            let tls = native_tls::TlsConnector::builder()
                .build()
                .map_err(|e| format!("TLS connector failed: {}", e))?;

            let client = imap::connect(
                (imap_server.as_str(), 993),
                &imap_server,
                &tls,
            )
            .map_err(|e| format!("IMAP connect failed: {}", e))?;

            let auth = XOAuth2 {
                user: email_clone.clone(),
                access_token,
            };
            let mut session = client
                .authenticate("XOAUTH2", &auth)
                .map_err(|e| format!("IMAP authentication failed: {:?}", e))?;

            session
                .select(&folder_name)
                .map_err(|e| format!("Failed to select folder: {:?}", e))?;

            // 获取完整邮件内容
            let messages = session
                .fetch(&msg_id, "RFC822")
                .map_err(|e| format!("Failed to fetch message: {:?}", e))?;

            if let Some(message) = messages.iter().next() {
                let body = message.body().ok_or("No message body found")?;

                // 解析邮件
                let parsed = std::str::from_utf8(body)
                    .map_err(|e| format!("Failed to parse email: {}", e))?;

                // 解析邮件头部和正文
                let (headers, body_content) = Self::parse_email_content(parsed)?;

                let subject = headers
                    .get("Subject")
                    .cloned()
                    .unwrap_or_else(|| "(No Subject)".to_string());
                let from_email = headers
                    .get("From")
                    .cloned()
                    .unwrap_or_else(|| "(Unknown Sender)".to_string());
                let to_email = headers
                    .get("To")
                    .cloned()
                    .unwrap_or_else(|| "(Unknown Recipient)".to_string());
                let date = headers
                    .get("Date")
                    .cloned()
                    .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                // 解析邮件正文
                let (body_plain, body_html) = Self::extract_email_body(&body_content)?;

                session.logout().ok();

                Ok(EmailDetailsResponse {
                    message_id: message_id_clone,
                    subject,
                    from_email,
                    to_email,
                    cc_email: None,
                    date,
                    body_plain,
                    body_html,
                })
            } else {
                Err("Message not found".to_string())
            }
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
    }

    // 获取邮件列表
    #[allow(dead_code)]
    pub async fn get_emails(
        &self,
        email: &str,
        folder: &str,
        page: i32,
        page_size: i32,
    ) -> Result<EmailListResponse, String> {
        let credentials = self.get_credentials(email)?;
        self.get_emails_with_credentials(&credentials, folder, page, page_size)
            .await
    }

    // 使用凭证获取邮件列表（避免跨 await 持有锁）
    pub async fn get_emails_with_credentials(
        &self,
        credentials: &OutlookCredentials,
        folder: &str,
        page: i32,
        page_size: i32,
    ) -> Result<EmailListResponse, String> {
        let mut session = self.create_imap_connection(credentials).await?;

        let folder_name = match folder {
            "inbox" => "INBOX",
            "junk" => "Junk",
            _ => "INBOX",
        };

        // 在异步上下文中运行同步IMAP代码
        let email_clone = credentials.email.clone();
        let folder_clone = folder.to_string();

        tokio::task::spawn_blocking(move || {
            session
                .select(folder_name)
                .map_err(|e| format!("Failed to select folder: {:?}", e))?;

            let messages = session
                .search("ALL")
                .map_err(|e| format!("Failed to search messages: {:?}", e))?;

            let mut message_vec: Vec<u32> = messages.into_iter().collect();
            // 按消息ID倒序排列，确保最新邮件在前
            message_vec.sort_by(|a, b| b.cmp(a));

            let total_emails = message_vec.len() as i32;
            let start_idx = ((page - 1) * page_size) as usize;
            let end_idx = std::cmp::min(start_idx + page_size as usize, message_vec.len());

            let mut emails = Vec::new();

            if start_idx < message_vec.len() {
                let page_messages = &message_vec[start_idx..end_idx];

                for &msg_id in page_messages.iter() {
                    // 按消息ID顺序（通常是时间倒序）
                    if let Ok(messages) = session.fetch(msg_id.to_string(), "ENVELOPE") {
                        for msg in messages.iter() {
                            if let Some(envelope) = msg.envelope() {
                                let subject = envelope
                                    .subject
                                    .and_then(|s| std::str::from_utf8(s).ok())
                                    .map(|s| Self::decode_header_value(s))
                                    .unwrap_or_else(|| "(No Subject)".to_string());

                                let from_email = envelope
                                    .from
                                    .as_ref()
                                    .and_then(|addrs| addrs.first())
                                    .map(|addr| {
                                        let mailbox = addr.mailbox
                                            .and_then(|mb| std::str::from_utf8(mb).ok())
                                            .unwrap_or("unknown");
                                        let host = addr.host
                                            .and_then(|h| std::str::from_utf8(h).ok())
                                            .unwrap_or("");
                                        if host.is_empty() {
                                            mailbox.to_string()
                                        } else {
                                            format!("{}@{}", mailbox, host)
                                        }
                                    })
                                    .unwrap_or_else(|| "(Unknown)".to_string());

                                let date = envelope
                                    .date
                                    .and_then(|d| std::str::from_utf8(d).ok())
                                    .unwrap_or("")
                                    .to_string();

                                let sender_initial = from_email
                                    .chars()
                                    .next()
                                    .unwrap_or('?')
                                    .to_uppercase()
                                    .to_string();

                                emails.push(EmailItem {
                                    message_id: format!("{}-{}", folder_name, msg_id),
                                    folder: folder_name.to_string(),
                                    subject,
                                    from_email,
                                    date,
                                    is_read: false,         // 简化处理
                                    has_attachments: false, // 简化处理
                                    sender_initial,
                                });
                            }
                        }
                    }
                }
            }

            session.logout().ok();

            Ok(EmailListResponse {
                email_id: email_clone,
                folder_view: folder_clone,
                page,
                page_size,
                total_emails,
                emails,
                method: "imap".to_string(),
            })
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
    }

    // 解析邮件头部和正文
    fn parse_email_content(
        email_content: &str,
    ) -> Result<(HashMap<String, String>, String), String> {
        let mut headers = HashMap::new();
        let mut body = String::new();
        let mut in_headers = true;
        let mut current_header = String::new();
        let mut current_value = String::new();

        for line in email_content.lines() {
            if in_headers {
                if line.is_empty() {
                    // 保存最后一个头部
                    if !current_header.is_empty() {
                        headers.insert(
                            current_header.clone(),
                            Self::decode_header_value(&current_value),
                        );
                    }
                    in_headers = false;
                    continue;
                }

                if line.starts_with(' ') || line.starts_with('\t') {
                    // 续行
                    current_value.push(' ');
                    current_value.push_str(line.trim());
                } else if let Some(colon_pos) = line.find(':') {
                    // 保存上一个头部
                    if !current_header.is_empty() {
                        headers.insert(
                            current_header.clone(),
                            Self::decode_header_value(&current_value),
                        );
                    }
                    // 开始新头部
                    current_header = line[..colon_pos].to_string();
                    current_value = line[colon_pos + 1..].trim().to_string();
                }
            } else {
                body.push_str(line);
                body.push('\n');
            }
        }

        Ok((headers, body))
    }

    // 解码邮件头部值 (RFC 2047)
    fn decode_header_value(value: &str) -> String {
        let mut result = String::new();
        let mut remaining = value;

        while let Some(start) = remaining.find("=?") {
            // 添加编码部分之前的纯文本
            result.push_str(&remaining[..start]);
            remaining = &remaining[start + 2..];

            // 解析 charset?encoding?encoded_text?=
            let parts: Vec<&str> = remaining.splitn(4, '?').collect();
            if parts.len() >= 3 {
                let _charset = parts[0];
                let encoding = parts[1].to_uppercase();
                let encoded_text = parts[2];

                // 找到 ?= 结束标记
                if let Some(end_pos) = remaining.find("?=") {
                    let decoded = match encoding.as_str() {
                        "B" => {
                            base64::engine::general_purpose::STANDARD
                                .decode(encoded_text)
                                .ok()
                                .and_then(|bytes| String::from_utf8(bytes).ok())
                        }
                        "Q" => {
                            // Quoted-Printable 解码
                            let qp = encoded_text
                                .replace('_', " ")
                                .chars()
                                .collect::<String>();
                            let mut bytes = Vec::new();
                            let mut chars = qp.chars().peekable();
                            while let Some(c) = chars.next() {
                                if c == '=' {
                                    let hex: String = chars.by_ref().take(2).collect();
                                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                        bytes.push(byte);
                                    }
                                } else {
                                    bytes.push(c as u8);
                                }
                            }
                            String::from_utf8(bytes).ok()
                        }
                        _ => None,
                    };

                    result.push_str(&decoded.unwrap_or_else(|| encoded_text.to_string()));
                    remaining = &remaining[end_pos + 2..];
                    // 跳过编码词之间的空格（RFC 2047 规定）
                    remaining = remaining.trim_start();
                    continue;
                }
            }
            // 解析失败，原样输出
            result.push_str("=?");
        }

        result.push_str(remaining);
        result
    }

    // 提取邮件正文
    fn extract_email_body(body_content: &str) -> Result<(Option<String>, Option<String>), String> {
        let mut body_plain = None;
        let mut body_html = None;

        // 检查是否是多部分邮件
        if body_content.contains("multipart/") && body_content.contains("boundary") {
            // 查找边界
            let boundary = Self::find_boundary(body_content);
            if let Some(boundary_str) = boundary {
                let boundary_marker = format!("--{}", boundary_str);
                let parts: Vec<&str> = body_content.split(&boundary_marker).collect();

                for part in parts {
                    if part.trim().is_empty() {
                        continue;
                    }

                    if part.contains("text/plain") {
                        if let Some(content) = Self::extract_part_content(part) {
                            body_plain = Some(content);
                        }
                    } else if part.contains("text/html") {
                        if let Some(content) = Self::extract_part_content(part) {
                            body_html = Some(content);
                        }
                    }
                }
            }
        } else {
            // 单部分邮件 - 直接提取内容
            let cleaned_content = Self::extract_simple_body(body_content);
            if !cleaned_content.trim().is_empty() {
                if body_content.contains("text/html") {
                    body_html = Some(cleaned_content);
                } else {
                    body_plain = Some(cleaned_content);
                }
            }
        }

        // 如果没有找到任何内容，尝试提取所有可见文本
        if body_plain.is_none() && body_html.is_none() {
            let fallback_content = Self::extract_fallback_content(body_content);
            if !fallback_content.trim().is_empty() {
                body_plain = Some(fallback_content);
            }
        }

        Ok((body_plain, body_html))
    }

    // 查找边界字符串
    fn find_boundary(content: &str) -> Option<String> {
        // 查找 boundary= 后面的值
        if let Some(start) = content.find("boundary=") {
            let after_boundary = &content[start + 9..];
            let boundary_line = after_boundary.lines().next().unwrap_or("");

            // 移除引号和分号
            let boundary = boundary_line
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            if !boundary.is_empty() {
                Some(boundary.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    // 提取部分内容
    fn extract_part_content(part: &str) -> Option<String> {
        let lines: Vec<&str> = part.lines().collect();
        let mut content_start = 0;

        // 找到空行，表示头部结束
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                content_start = i + 1;
                break;
            }
        }

        if content_start < lines.len() {
            let content_lines = &lines[content_start..];
            let content = content_lines
                .iter()
                .filter(|line| !line.starts_with("--")) // 过滤边界标记
                .map(|line| line.trim())
                .filter(|line| !line.is_empty())
                .collect::<Vec<_>>()
                .join("\n");

            if !content.trim().is_empty() {
                Some(Self::decode_content(&content))
            } else {
                None
            }
        } else {
            None
        }
    }

    // 提取简单邮件正文
    fn extract_simple_body(content: &str) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let mut body_start = 0;

        // 跳过头部，找到第一个空行后的内容
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                body_start = i + 1;
                break;
            }
        }

        if body_start < lines.len() {
            let body_lines = &lines[body_start..];
            let content = body_lines
                .iter()
                .filter(|line| !line.starts_with("Content-"))
                .filter(|line| !line.starts_with("MIME-"))
                .filter(|line| !line.starts_with("--"))
                .map(|line| *line)
                .collect::<Vec<_>>()
                .join("\n");

            Self::decode_content(&content)
        } else {
            String::new()
        }
    }

    // 提取备用内容（当其他方法都失败时）
    fn extract_fallback_content(content: &str) -> String {
        // 简单地提取所有看起来像正文的内容
        content
            .lines()
            .skip_while(|line| {
                line.starts_with("Content-")
                    || line.starts_with("MIME-")
                    || line.starts_with("Date:")
                    || line.starts_with("From:")
                    || line.starts_with("To:")
                    || line.starts_with("Subject:")
                    || line.starts_with("Message-ID:")
                    || line.contains("boundary=")
                    || line.trim().is_empty()
            })
            .filter(|line| {
                !line.starts_with("--")
                    && !line.starts_with("Content-")
                    && !line.starts_with("MIME-")
                    && !line.trim().is_empty()
            })
            .take(50) // 限制行数，避免过长
            .collect::<Vec<_>>()
            .join("\n")
    }

    // 解码内容
    fn decode_content(content: &str) -> String {
        // 处理 Quoted-Printable 编码
        let decoded = if content.contains("=\n") || content.contains("=20") || content.contains("=3D") {
            content
                .replace("=\n", "")
                .replace("=20", " ")
                .replace("=3D", "=")
                .replace("=0A", "\n")
                .replace("=0D", "\r")
        } else {
            content.to_string()
        };

        // 限制长度
        if decoded.len() > 5000 {
            let mut truncated = decoded.chars().take(5000).collect::<String>();
            truncated.push_str("\n\n[内容已截断...]");
            truncated
        } else {
            decoded
        }
    }

    // ==================== Graph API 方法 ====================

    // 获取 Graph API 访问令牌
    // 返回 (access_token, Option<new_refresh_token>)
    pub async fn get_graph_access_token(
        &self,
        credentials: &OutlookCredentials,
    ) -> Result<(String, Option<String>), String> {
        let client = http_client::create_proxy_client()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let token_url = "https://login.microsoftonline.com/common/oauth2/v2.0/token";
        let params = [
            ("client_id", credentials.client_id.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", credentials.refresh_token.as_str()),
            ("scope", "https://graph.microsoft.com/.default"),
        ];

        let response = client
            .post(token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            if body.contains("service abuse") || body.contains("account is found to be") {
                return Err(format!("[BANNED] 账号被封禁 (service abuse mode)"));
            }
            return Err(format!(
                "Graph token request failed: {} - {}",
                status,
                &body[..body.len().min(200)]
            ));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Graph token response: {}", e))?;

        Ok((token_response.access_token, token_response.refresh_token))
    }

    // 通过 Graph API 获取邮件列表
    pub async fn graph_get_emails_with_credentials(
        &self,
        credentials: &OutlookCredentials,
        folder: &str,
        page: i32,
        page_size: i32,
    ) -> Result<EmailListResponse, String> {
        let (access_token, _new_rt) = self.get_graph_access_token(credentials).await?;
        let client = http_client::create_proxy_client()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let folder_name = match folder {
            "inbox" => "inbox",
            "junk" => "junkemail",
            _ => "inbox",
        };

        let skip = (page - 1) * page_size;
        let url = format!(
            "https://graph.microsoft.com/v1.0/me/mailFolders/{}/messages?$select=id,subject,from,receivedDateTime,isRead,hasAttachments&$orderby=receivedDateTime%20desc&$top={}&$skip={}&$count=true",
            folder_name, page_size, skip
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .header("ConsistencyLevel", "eventual")
            .send()
            .await
            .map_err(|e| format!("Graph API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Graph API failed: {} - {}",
                status,
                &body[..body.len().min(200)]
            ));
        }

        let graph_response: GraphMessagesResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Graph response: {}", e))?;

        let messages = graph_response.value.unwrap_or_default();
        let total_emails = graph_response.odata_count.unwrap_or(messages.len() as i32);

        let emails: Vec<EmailItem> = messages
            .iter()
            .map(|msg| {
                let from_email = msg
                    .from
                    .as_ref()
                    .and_then(|f| f.email_address.as_ref())
                    .and_then(|ea| ea.address.as_ref())
                    .cloned()
                    .unwrap_or_else(|| "(Unknown)".to_string());

                let sender_initial = from_email
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string();

                EmailItem {
                    message_id: msg.id.clone().unwrap_or_default(),
                    folder: folder.to_string(),
                    subject: msg
                        .subject
                        .clone()
                        .unwrap_or_else(|| "(No Subject)".to_string()),
                    from_email,
                    date: msg.received_date_time.clone().unwrap_or_default(),
                    is_read: msg.is_read.unwrap_or(false),
                    has_attachments: msg.has_attachments.unwrap_or(false),
                    sender_initial,
                }
            })
            .collect();

        Ok(EmailListResponse {
            email_id: credentials.email.clone(),
            folder_view: folder.to_string(),
            page,
            page_size,
            total_emails,
            emails,
            method: "graph".to_string(),
        })
    }

    // 通过 Graph API 获取邮件详情
    pub async fn graph_get_email_details_with_credentials(
        &self,
        credentials: &OutlookCredentials,
        message_id: &str,
    ) -> Result<EmailDetailsResponse, String> {
        let (access_token, _new_rt) = self.get_graph_access_token(credentials).await?;
        let client = http_client::create_proxy_client()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let url = format!(
            "https://graph.microsoft.com/v1.0/me/messages/{}?$select=id,subject,from,toRecipients,ccRecipients,receivedDateTime,body",
            message_id
        );

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Graph API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Graph API failed: {} - {}",
                status,
                &body[..body.len().min(200)]
            ));
        }

        let msg: GraphMessage = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Graph message: {}", e))?;

        let from_email = msg
            .from
            .as_ref()
            .and_then(|f| f.email_address.as_ref())
            .and_then(|ea| ea.address.as_ref())
            .cloned()
            .unwrap_or_else(|| "(Unknown Sender)".to_string());

        let to_email = msg
            .to_recipients
            .as_ref()
            .map(|recipients| {
                recipients
                    .iter()
                    .filter_map(|r| {
                        r.email_address
                            .as_ref()
                            .and_then(|ea| ea.address.as_ref())
                            .cloned()
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "(Unknown Recipient)".to_string());

        let body_content = msg
            .body
            .as_ref()
            .and_then(|b| b.content.as_ref())
            .cloned();

        let content_type = msg
            .body
            .as_ref()
            .and_then(|b| b.content_type.as_ref())
            .map(|ct| ct.to_lowercase());

        let (body_plain, body_html) = match content_type.as_deref() {
            Some("html") => (None, body_content),
            Some("text") => (body_content, None),
            _ => (None, body_content),
        };

        let cc_email = msg
            .cc_recipients
            .as_ref()
            .map(|recipients| {
                recipients
                    .iter()
                    .filter_map(|r| {
                        r.email_address
                            .as_ref()
                            .and_then(|ea| ea.address.as_ref())
                            .cloned()
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|s| !s.is_empty());

        Ok(EmailDetailsResponse {
            message_id: msg.id.unwrap_or_else(|| message_id.to_string()),
            subject: msg
                .subject
                .unwrap_or_else(|| "(No Subject)".to_string()),
            from_email,
            to_email,
            cc_email,
            date: msg
                .received_date_time
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            body_plain,
            body_html,
        })
    }
}
