use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RedeemRequest {
    pub key_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedeemResponse {
    pub success: bool,
    pub data: Option<RedeemData>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedeemData {
    pub auth_session: String,
    pub message: String,
}

/// 从服务器兑换卡密获取 auth_session
pub async fn redeem_activation_key(
    server_url: &str,
    key_code: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/redeem", server_url);

    let request_body = RedeemRequest {
        key_code: key_code.to_string(),
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

    if !redeem_response.success {
        return Err(
            redeem_response
                .error
                .unwrap_or_else(|| "兑换失败".to_string())
        );
    }

    let data = redeem_response
        .data
        .ok_or_else(|| "响应数据为空".to_string())?;

    Ok(data.auth_session)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要实际的服务器才能运行
    async fn test_redeem_activation_key() {
        let server_url = "http://localhost:3000";
        let key_code = "test-key-code";

        let result = redeem_activation_key(server_url, key_code).await;
        println!("Redeem result: {:?}", result);
    }
}

