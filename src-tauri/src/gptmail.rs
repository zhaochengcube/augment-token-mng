use crate::http_client::create_http_client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateEmailResponse {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub from: String,
    pub subject: String,
    pub content: String,
    #[serde(rename = "htmlContent")]
    pub html_content: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEmailsResponse {
    pub emails: Vec<Email>,
}

/// 生成随机邮箱
#[tauri::command]
pub async fn generate_random_email() -> Result<GenerateEmailResponse, String> {
    let client = create_http_client().map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get("https://mail.chatgpt.org.uk/api/generate-email")
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP 错误: {}", response.status()));
    }

    let data = response
        .json::<GenerateEmailResponse>()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    Ok(data)
}

/// 获取邮箱邮件
#[tauri::command]
pub async fn get_emails(email: String) -> Result<GetEmailsResponse, String> {
    let client = create_http_client().map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let url = format!("https://mail.chatgpt.org.uk/api/get-emails?email={}", email);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP 错误: {}", response.status()));
    }

    let data = response
        .json::<GetEmailsResponse>()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    Ok(data)
}

