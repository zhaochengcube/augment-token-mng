use crate::antigravity::models::quota::{QuotaData, QuotaResponse, LoadProjectResponse};
use reqwest;
use serde_json::json;

const QUOTA_API_URL: &str = "https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels";
const LOAD_PROJECT_API_URL: &str = "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist";
const USER_AGENT: &str = "antigravity/1.11.3 Darwin/arm64";

/// 获取 Project ID
async fn fetch_project_id(access_token: &str) -> Option<String> {
    let client = reqwest::Client::new();
    let body = json!({
        "metadata": {
            "ideType": "ANTIGRAVITY"
        }
    });

    // 简单的重试
    for _ in 0..2 {
        match client
            .post(LOAD_PROJECT_API_URL)
            .bearer_auth(access_token)
            .header("User-Agent", USER_AGENT)
            .json(&body)
            .send()
            .await 
        {
            Ok(res) if res.status().is_success() => {
                if let Ok(data) = res.json::<LoadProjectResponse>().await {
                    return data.project_id;
                }
            }
            _ => tokio::time::sleep(std::time::Duration::from_millis(500)).await,
        }
    }
    None
}

/// 查询账号配额
pub async fn fetch_quota(access_token: &str) -> Result<QuotaData, String> {
    let client = reqwest::Client::new();
    
    // 1. 获取 Project ID
    let project_id = fetch_project_id(access_token).await;
    
    // 2. 构建请求体
    let mut payload = serde_json::Map::new();
    if let Some(pid) = project_id {
        payload.insert("project".to_string(), json!(pid));
    }
    
    let max_retries = 3;
    let mut last_error: Option<String> = None;

    for attempt in 1..=max_retries {
        match client
            .post(QUOTA_API_URL)
            .bearer_auth(access_token)
            .header("User-Agent", USER_AGENT)
            .json(&json!(payload))
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                
                // 特殊处理 403 Forbidden - 直接返回,不重试
                if status == reqwest::StatusCode::FORBIDDEN {
                    let mut q = QuotaData::new();
                    q.is_forbidden = true;
                    return Ok(q);
                }
                
                // 处理 401 Unauthorized
                if status == reqwest::StatusCode::UNAUTHORIZED {
                    return Err("Token expired or invalid (401)".to_string());
                }
                
                if status.is_success() {
                    let quota_response: QuotaResponse = response.json().await
                        .map_err(|e| format!("Failed to parse quota response: {}", e))?;
                    
                    let mut quota_data = QuotaData::new();
                    
                    for (name, info) in quota_response.models {
                        if let Some(quota_info) = info.quota_info {
                            let percentage = quota_info.remaining_fraction
                                .map(|f| (f * 100.0) as i32)
                                .unwrap_or(0);
                            
                            let reset_time = quota_info.reset_time.unwrap_or_default();
                            
                            // 只保存我们关心的模型
                            if name.contains("gemini") || name.contains("claude") {
                                quota_data.add_model(name, percentage, reset_time);
                            }
                        }
                    }
                    
                    return Ok(quota_data);
                }
                
                last_error = Some(format!("HTTP {}: {}", status, response.text().await.unwrap_or_default()));
            },
            Err(e) => {
                last_error = Some(format!("Request failed: {}", e));
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| "Quota fetch failed after retries".to_string()))
}

