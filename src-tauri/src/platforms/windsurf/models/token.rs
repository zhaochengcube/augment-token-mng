use serde::{Deserialize, Serialize};

/// Windsurf Token 数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: String,
    pub expiry_timestamp: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

impl TokenData {
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        email: Option<String>,
        user_id: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            access_token,
            refresh_token,
            expiry_timestamp: now + expires_in,
            email,
            user_id,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expiry_timestamp <= now
    }

    pub fn needs_refresh(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        // 5 分钟内过期就需要刷新
        self.expiry_timestamp <= now + 300
    }
}

/// Firebase Token 响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseTokenResponse {
    #[serde(rename = "idToken")]
    pub id_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    #[serde(rename = "expiresIn")]
    pub expires_in: String,
    #[serde(rename = "localId", default)]
    pub local_id: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

impl FirebaseTokenResponse {
    pub fn expires_in_seconds(&self) -> i64 {
        self.expires_in.parse().unwrap_or(3600)
    }
}

/// Firebase 用户信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseUserInfo {
    #[serde(rename = "localId")]
    pub local_id: String,
    pub email: String,
    #[serde(rename = "displayName", default)]
    pub display_name: Option<String>,
    #[serde(rename = "emailVerified", default)]
    pub email_verified: bool,
}
