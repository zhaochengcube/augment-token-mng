use serde::{Deserialize, Serialize};

/// Cursor Token 数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: String,
    pub expiry_timestamp: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// WorkOS Cursor Session Token (用于 Dashboard API 调用)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workos_cursor_session_token: Option<String>,
    /// Session Token 的过期时间戳
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_expiry_timestamp: Option<i64>,
}

impl TokenData {
    pub fn new(
        access_token: String,
        refresh_token: String,
        expiry_timestamp: i64,
        user_id: Option<String>,
        workos_cursor_session_token: Option<String>,
        session_expiry_timestamp: Option<i64>,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            expiry_timestamp,
            user_id,
            workos_cursor_session_token,
            session_expiry_timestamp,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expiry_timestamp <= now
    }

    /// 检查 session token 是否过期
    pub fn is_session_expired(&self) -> bool {
        match self.session_expiry_timestamp {
            Some(exp) => chrono::Utc::now().timestamp() >= exp,
            None => false,
        }
    }
}
