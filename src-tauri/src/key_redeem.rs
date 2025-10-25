use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct RedeemRequest {
    key_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    count: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedeemData {
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_session: Option<String>,  // 单个兑换时使用
    #[serde(skip_serializing_if = "Option::is_none")]
    sessions: Option<Vec<String>>,  // 批量兑换时使用
    #[serde(skip_serializing_if = "Option::is_none")]
    remaining_uses: Option<u32>,  // 剩余使用次数
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RedeemResponse {
    success: bool,
    data: Option<RedeemData>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchRedeemResult {
    pub sessions: Vec<String>,
    pub remaining_uses: u32,
}

/// 调用服务端 API 批量兑换卡密
pub async fn redeem_activation_keys_batch(
    server_url: &str,
    key_code: &str,
    count: u32,
) -> Result<BatchRedeemResult, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/redeem", server_url);

    let request_body = RedeemRequest {
        key_code: key_code.to_string(),
        count: Some(count),
    };

    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "未知错误".to_string());
        return Err(format!("服务器返回错误 {}: {}", status, error_text));
    }

    let redeem_response: RedeemResponse = response
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    if redeem_response.success {
        if let Some(data) = redeem_response.data {
            let sessions = data.sessions.unwrap_or_default();
            let remaining_uses = data.remaining_uses.unwrap_or(0);

            if sessions.is_empty() {
                return Err("服务器未返回任何 Session".to_string());
            }

            Ok(BatchRedeemResult {
                sessions,
                remaining_uses,
            })
        } else {
            Err("服务器未返回数据".to_string())
        }
    } else {
        Err(redeem_response
            .error
            .unwrap_or_else(|| "兑换失败".to_string()))
    }
}

