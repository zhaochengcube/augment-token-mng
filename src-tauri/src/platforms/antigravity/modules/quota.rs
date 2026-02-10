use crate::antigravity::models::quota::{LoadProjectResponse, QuotaData, QuotaResponse};
use crate::http_client::create_proxy_client;
use serde_json::json;

const QUOTA_API_URL: &str = "https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels";
const LOAD_PROJECT_API_URL: &str = "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist";
const USER_AGENT: &str = "antigravity/1.11.3 Darwin/arm64";
const DEFAULT_PROJECT_ID: &str = "bamboo-precept-lgxtn";

/// 获取 Project ID
async fn fetch_project_id(access_token: &str) -> (Option<String>, Option<String>) {
    let client = match create_proxy_client() {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to create proxy client: {}", e);
            return (None, None);
        }
    };

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
                    let subscription_tier = data
                        .paid_tier
                        .and_then(|tier| tier.id)
                        .or_else(|| data.current_tier.and_then(|tier| tier.id));
                    return (data.project_id, subscription_tier);
                }
            }
            _ => tokio::time::sleep(std::time::Duration::from_millis(500)).await,
        }
    }
    (None, None)
}

/// 查询账号配额
pub async fn fetch_quota(access_token: &str) -> Result<(QuotaData, Option<String>), String> {
    println!("=== fetch_quota ===");
    println!(
        "access_token: {}...",
        &access_token.chars().take(20).collect::<String>()
    );

    let client = create_proxy_client()?;

    // 1. 获取 Project ID
    let (project_id, subscription_tier) = fetch_project_id(access_token).await;
    println!("project_id: {:?}", project_id);

    // 2. 构建请求体
    let final_project_id = project_id
        .clone()
        .unwrap_or_else(|| DEFAULT_PROJECT_ID.to_string());
    let mut payload = serde_json::Map::new();
    payload.insert("project".to_string(), json!(final_project_id));
    println!("payload: {:?}", payload);

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
                println!("Response status (attempt {}): {}", attempt, status);

                // 特殊处理 403 Forbidden - 直接返回,不重试
                if status == reqwest::StatusCode::FORBIDDEN {
                    println!("Account is forbidden (403)");
                    let mut q = QuotaData::new();
                    q.is_forbidden = true;
                    q.subscription_tier = subscription_tier.clone();
                    return Ok((q, project_id));
                }

                // 处理 401 Unauthorized
                if status == reqwest::StatusCode::UNAUTHORIZED {
                    println!("Token expired or invalid (401)");
                    return Err("HTTP 401: Token expired or invalid".to_string());
                }

                if status.is_success() {
                    let quota_response: QuotaResponse = response
                        .json()
                        .await
                        .map_err(|e| format!("Failed to parse quota response: {}", e))?;

                    println!(
                        "Quota response models count: {}",
                        quota_response.models.len()
                    );

                    let mut quota_data = QuotaData::new();
                    quota_data.subscription_tier = subscription_tier.clone();

                    println!("models: {:?}", quota_response.models);

                    for (name, info) in quota_response.models {
                        if let Some(quota_info) = info.quota_info {
                            let percentage = quota_info
                                .remaining_fraction
                                .map(|f| (f * 100.0) as i32)
                                .unwrap_or(0);

                            let reset_time = quota_info.reset_time.unwrap_or_default();

                            println!(
                                "Model: {}, percentage: {}%, reset_time: {}",
                                name, percentage, reset_time
                            );
                            quota_data.add_model(name, percentage, reset_time);
                        }
                    }

                    println!("Final quota_data: {:?}", quota_data);
                    return Ok((quota_data, project_id));
                }

                let error_text = response.text().await.unwrap_or_default();
                println!("HTTP error {}: {}", status, error_text);
                last_error = Some(format!("HTTP {}: {}", status, error_text));
            }
            Err(e) => {
                println!("Request failed (attempt {}): {}", attempt, e);
                last_error = Some(format!("Request failed: {}", e));
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    let final_error = last_error.unwrap_or_else(|| "Quota fetch failed after retries".to_string());
    println!("Quota fetch failed: {}", final_error);
    Err(final_error)
}
