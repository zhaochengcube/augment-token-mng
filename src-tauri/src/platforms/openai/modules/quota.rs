use crate::http_client::create_proxy_client;
use crate::platforms::openai::models::QuotaData;

const CHATGPT_CODEX_API_URL: &str = "https://chatgpt.com/backend-api/codex/responses";

/// 系统指令（用于 Codex API）
const DEFAULT_INSTRUCTIONS: &str = "You are Codex, based on GPT-5. You are running as a coding agent in the Codex CLI on a user's computer.";

/// 查询 OpenAI 账号配额
///
/// 通过调用 ChatGPT Codex API 获取配额信息
/// 配额信息从响应头中提取
pub async fn fetch_quota(
    access_token: &str,
    chatgpt_account_id: Option<&str>,
) -> Result<QuotaData, String> {
    println!("=== OpenAI fetch_quota ===");
    println!("access_token: {}...", &access_token.chars().take(20).collect::<String>());

    let client = create_proxy_client()?;

    // 构建请求 - 使用 Responses API 格式（必须启用流式）
    let request_body = serde_json::json!({
        "model": "gpt-5.1-codex",
        "input": [{
            "role": "user",
            "content": [{
                "type": "input_text",
                "text": "hi"
            }]
        }],
        "instructions": DEFAULT_INSTRUCTIONS,
        "store": false,
        "stream": true
    });

    // 构建请求
    let mut request_builder = client
        .post(CHATGPT_CODEX_API_URL)
        .header("authorization", format!("Bearer {}", access_token))
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .header("Host", "chatgpt.com")
        .json(&request_body);

    if let Some(account_id) = chatgpt_account_id {
        request_builder = request_builder.header("chatgpt-account-id", account_id);
    }

    let max_retries = 2;
    let mut last_error: Option<String> = None;

    for attempt in 1..=max_retries {
        match request_builder
            .try_clone()
            .unwrap()
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                println!("Response status (attempt {}): {}", attempt, status);

                // 处理 401 Unauthorized
                if status == reqwest::StatusCode::UNAUTHORIZED {
                    println!("Token expired or invalid (401)");
                    return Err("HTTP 401: Token expired or invalid".to_string());
                }

                // 处理 403 Forbidden
                if status == reqwest::StatusCode::FORBIDDEN {
                    println!("Account is forbidden (403)");
                    let mut quota = QuotaData::new();
                    quota.is_forbidden = true;
                    return Ok(quota);
                }

                if status.is_success() {
                    // 从响应头提取配额信息
                    if let Some(quota) = QuotaData::from_headers(response.headers()) {
                        println!("Successfully parsed quota from headers");
                        println!("  5h used: {:?}%", quota.codex_5h_used_percent);
                        println!("  7d used: {:?}%", quota.codex_7d_used_percent);
                        return Ok(quota);
                    } else {
                        println!("No quota headers found in response");
                        // 返回空的配额数据（表示没有配额限制或无法获取）
                        return Ok(QuotaData::new());
                    }
                }

                let error_text = response.text().await.unwrap_or_default();
                println!("HTTP error {}: {}", status, error_text);
                last_error = Some(format!("HTTP {}: {}", status, error_text));
            }
            Err(e) => {
                println!("Request failed (attempt {}): {}", attempt, e);
                last_error = Some(format!("Request failed: {}", e));
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    }

    let final_error = last_error.unwrap_or_else(|| "Quota fetch failed after retries".to_string());
    println!("Quota fetch failed: {}", final_error);
    Err(final_error)
}
