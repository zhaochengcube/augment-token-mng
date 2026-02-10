use serde::{Deserialize, Serialize};

/// OpenAI Token 数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    pub expires_in: i64,
    pub expires_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
}

impl TokenData {
    pub fn new(
        access_token: String,
        refresh_token: Option<String>,
        id_token: Option<String>,
        expires_in: i64,
        expires_at: i64,
        token_type: Option<String>,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            id_token,
            expires_in,
            expires_at,
            token_type,
        }
    }
}
