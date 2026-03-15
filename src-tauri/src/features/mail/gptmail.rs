use crate::http_client::create_http_client;
use crate::AppState;
use super::gptmail_storage::{GptMailRecord, GptMailStorage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub daily_limit: Option<i64>,
    #[serde(default)]
    pub used_today: Option<i64>,
    #[serde(default)]
    pub remaining_today: Option<i64>,
    #[serde(default)]
    pub total_limit: Option<i64>,
    #[serde(default)]
    pub total_usage: Option<i64>,
    #[serde(default)]
    pub remaining_total: Option<i64>,
}

fn parse_usage(body: &Value) -> Option<Usage> {
    body.get("usage")
        .or_else(|| body.get("data").and_then(|d| d.get("usage")))
        .and_then(|v| serde_json::from_value::<Usage>(v.clone()).ok())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateEmailResponse {
    pub email: String,
    pub usage: Option<Usage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    #[serde(alias = "from_address")]
    pub from: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub content: String,
    #[serde(rename = "htmlContent", alias = "html_content", default)]
    pub html_content: String,
    #[serde(default)]
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEmailsResponse {
    pub emails: Vec<Email>,
    pub usage: Option<Usage>,
}

/// Extract API error message from a JSON body, returns None if no error found.
fn extract_api_error(body: &Value) -> Option<String> {
    if body.get("success").and_then(|v| v.as_bool()) == Some(false) {
        let msg = body
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("未知 API 错误");
        return Some(msg.to_string());
    }
    None
}

/// Try to parse a non-success HTTP response body for a meaningful error message.
fn parse_http_error(status: reqwest::StatusCode, body_text: &str) -> String {
    if let Ok(body) = serde_json::from_str::<Value>(body_text) {
        if let Some(err) = body.get("error").and_then(|v| v.as_str()) {
            return err.to_string();
        }
    }
    format!("HTTP 错误: {}", status)
}

/// 生成随机邮箱
#[tauri::command]
pub async fn generate_random_email(api_key: Option<String>) -> Result<GenerateEmailResponse, String> {
    let client = create_http_client().map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get("https://mail.chatgpt.org.uk/api/generate-email")
        .header("X-API-Key", api_key.as_deref().unwrap_or("gpt-test"))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = response.status();
    let body_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    if !status.is_success() {
        return Err(parse_http_error(status, &body_text));
    }

    let body: Value =
        serde_json::from_str(&body_text).map_err(|e| format!("解析响应失败: {}", e))?;

    if let Some(err) = extract_api_error(&body) {
        return Err(err);
    }

    let email = body
        .get("data")
        .and_then(|d| d.get("email"))
        .and_then(|e| e.as_str())
        .or_else(|| body.get("email").and_then(|e| e.as_str()))
        .ok_or("API 响应中未找到邮箱地址")?
        .to_string();

    let usage = parse_usage(&body);

    Ok(GenerateEmailResponse { email, usage })
}

/// 获取邮箱邮件
#[tauri::command]
pub async fn get_emails(email: String, api_key: Option<String>) -> Result<GetEmailsResponse, String> {
    let client = create_http_client().map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("https://mail.chatgpt.org.uk/api/emails?email={}", email);

    let response = client
        .get(&url)
        .header("X-API-Key", api_key.as_deref().unwrap_or("gpt-test"))
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let status = response.status();
    let body_text = response
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    if !status.is_success() {
        return Err(parse_http_error(status, &body_text));
    }

    let body: Value =
        serde_json::from_str(&body_text).map_err(|e| format!("解析响应失败: {}", e))?;

    if let Some(err) = extract_api_error(&body) {
        return Err(err);
    }

    let emails_value = body
        .get("data")
        .and_then(|d| d.get("emails"))
        .or_else(|| body.get("emails"))
        .ok_or("API 响应中未找到邮件列表")?;

    let emails: Vec<Email> = serde_json::from_value(emails_value.clone())
        .map_err(|e| format!("解析邮件列表失败: {}", e))?;

    let usage = parse_usage(&body);

    Ok(GetEmailsResponse { emails, usage })
}

// ---------------------------------------------------------------------------
// GPTMail local storage commands
// ---------------------------------------------------------------------------

fn get_gptmail_storage(state: &State<'_, AppState>) -> Result<Arc<GptMailStorage>, String> {
    state
        .gptmail_storage
        .lock()
        .map_err(|_| "Failed to access GPTMail storage state".to_string())?
        .clone()
        .ok_or_else(|| "GPTMail storage not initialized".to_string())
}

#[tauri::command]
pub async fn gptmail_list_history(
    search: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<GptMailRecord>, String> {
    let storage = get_gptmail_storage(&state)?;
    storage.load_all(search.as_deref())
}

#[tauri::command]
pub async fn gptmail_save_email(
    email: String,
    label: Option<String>,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<GptMailRecord, String> {
    let storage = get_gptmail_storage(&state)?;
    storage.save(&email, label.as_deref().unwrap_or(""), description.as_deref().unwrap_or(""))
}

#[tauri::command]
pub async fn gptmail_update_email(
    id: i64,
    label: Option<String>,
    description: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let storage = get_gptmail_storage(&state)?;
    storage.update(id, label.as_deref().unwrap_or(""), description.as_deref().unwrap_or(""))
}

#[tauri::command]
pub async fn gptmail_delete_emails(
    ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let storage = get_gptmail_storage(&state)?;
    storage.delete_batch(&ids)
}

#[tauri::command]
pub async fn gptmail_update_tag(
    id: i64,
    tag: Option<String>,
    tag_color: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let storage = get_gptmail_storage(&state)?;
    storage.update_tag(id, tag.as_deref(), tag_color.as_deref())
}
