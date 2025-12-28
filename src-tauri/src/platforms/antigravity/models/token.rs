use serde::{Deserialize, Serialize};

/// Google OAuth Token 数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub expiry_timestamp: i64,
    pub token_type: String,
    pub email: Option<String>,
    /// Google Cloud 项目ID，用于 API 请求标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,  // Antigravity sessionId
}

impl TokenData {
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        email: Option<String>,
        project_id: Option<String>,
        session_id: Option<String>,
    ) -> Self {
        let expiry_timestamp = chrono::Utc::now().timestamp() + expires_in;
        Self {
            access_token,
            refresh_token,
            expires_in,
            expiry_timestamp,
            token_type: "Bearer".to_string(),
            email,
            project_id,
            session_id,
        }
    }
}

/// Google OAuth Token 响应
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    #[serde(default)]
    pub token_type: String,
}

/// Google 用户信息
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub picture: Option<String>,
}

impl UserInfo {
    pub fn get_display_name(&self) -> Option<String> {
        self.name.clone().or_else(|| Some(self.email.split('@').next()?.to_string()))
    }
}

