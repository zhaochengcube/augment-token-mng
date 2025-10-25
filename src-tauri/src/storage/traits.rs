use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub id: String,
    pub tenant_url: String,
    pub access_token: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub portal_url: Option<String>,
    pub email_note: Option<String>,
    pub ban_status: Option<serde_json::Value>,
    pub portal_info: Option<serde_json::Value>,
    pub auth_session: Option<String>,
    pub suspensions: Option<serde_json::Value>,
    pub balance_color_mode: Option<String>,
    pub skip_check: Option<bool>,
}

impl TokenData {
    pub fn new(
        id: String,
        tenant_url: String,
        access_token: String,
        portal_url: Option<String>,
        email_note: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            tenant_url,
            access_token,
            created_at: now,
            updated_at: now,
            portal_url,
            email_note,
            ban_status: None,
            portal_info: None,
            auth_session: None,
            suspensions: None,
            balance_color_mode: None,
            skip_check: None,
        }
    }

    pub fn update_timestamp(&mut self) {
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub last_sync_at: Option<DateTime<Utc>>,
    pub sync_direction: String,
    pub status: String,
    pub error_message: Option<String>,
    pub tokens_synced: i32,
}

#[async_trait::async_trait]
pub trait TokenStorage: Send + Sync {
    async fn save_token(&self, token: &TokenData) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    async fn load_tokens(&self) -> Result<Vec<TokenData>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn update_token(&self, token: &TokenData) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    async fn delete_token(&self, token_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn get_token(&self, token_id: &str) -> Result<Option<TokenData>, Box<dyn std::error::Error + Send + Sync>>;
    
    async fn clear_all_tokens(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    fn storage_type(&self) -> &'static str;
    
    async fn is_available(&self) -> bool;
}

#[async_trait::async_trait]
pub trait SyncManager: Send + Sync {
    async fn sync_local_to_remote(&self) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>>;

    async fn sync_remote_to_local(&self) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>>;

    async fn bidirectional_sync(&self) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>>;

    async fn bidirectional_sync_with_tokens(&self, local_tokens: Vec<TokenData>) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_sync_status(&self) -> Result<Option<SyncStatus>, Box<dyn std::error::Error + Send + Sync>>;

    async fn resolve_conflicts(&self, local_tokens: Vec<TokenData>, remote_tokens: Vec<TokenData>) -> Result<Vec<TokenData>, Box<dyn std::error::Error + Send + Sync>>;
}

// 辅助函数：将旧格式的token转换为新格式
pub fn convert_legacy_token(legacy: &serde_json::Value) -> Result<TokenData, Box<dyn std::error::Error + Send + Sync>> {
    let id = legacy.get("id")
        .and_then(|v| v.as_str())
        .ok_or("Missing id field")?
        .to_string();
    
    let tenant_url = legacy.get("tenant_url")
        .and_then(|v| v.as_str())
        .ok_or("Missing tenant_url field")?
        .to_string();
    
    let access_token = legacy.get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("Missing access_token field")?
        .to_string();
    
    let created_at = legacy.get("created_at")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);

    let updated_at = legacy.get("updated_at")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or(created_at); // 如果没有updated_at字段，使用created_at作为默认值

    let portal_url = legacy.get("portal_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let email_note = legacy.get("email_note")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let ban_status = legacy.get("ban_status").cloned();
    let portal_info = legacy.get("portal_info").cloned();
    let auth_session = legacy.get("auth_session")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let suspensions = legacy.get("suspensions").cloned();
    let balance_color_mode = legacy.get("balance_color_mode")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let skip_check = legacy.get("skip_check")
        .and_then(|v| v.as_bool());

    Ok(TokenData {
        id,
        tenant_url,
        access_token,
        created_at,
        updated_at,
        portal_url,
        email_note,
        ban_status,
        portal_info,
        auth_session,
        suspensions,
        balance_color_mode,
        skip_check,
    })
}

// 辅助函数：将新格式的token转换为旧格式（用于向后兼容）
pub fn convert_to_legacy_format(token: &TokenData) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    
    map.insert("id".to_string(), serde_json::Value::String(token.id.clone()));
    map.insert("tenant_url".to_string(), serde_json::Value::String(token.tenant_url.clone()));
    map.insert("access_token".to_string(), serde_json::Value::String(token.access_token.clone()));
    map.insert("created_at".to_string(), serde_json::Value::String(token.created_at.to_rfc3339()));
    map.insert("updated_at".to_string(), serde_json::Value::String(token.updated_at.to_rfc3339()));
    
    if let Some(portal_url) = &token.portal_url {
        map.insert("portal_url".to_string(), serde_json::Value::String(portal_url.clone()));
    }
    
    if let Some(email_note) = &token.email_note {
        map.insert("email_note".to_string(), serde_json::Value::String(email_note.clone()));
    }
    
    if let Some(ban_status) = &token.ban_status {
        map.insert("ban_status".to_string(), ban_status.clone());
    }
    
    if let Some(portal_info) = &token.portal_info {
        map.insert("portal_info".to_string(), portal_info.clone());
    }

    if let Some(auth_session) = &token.auth_session {
        map.insert("auth_session".to_string(), serde_json::Value::String(auth_session.clone()));
    }

    if let Some(suspensions) = &token.suspensions {
        map.insert("suspensions".to_string(), suspensions.clone());
    }

    if let Some(balance_color_mode) = &token.balance_color_mode {
        map.insert("balance_color_mode".to_string(), serde_json::Value::String(balance_color_mode.clone()));
    }

    if let Some(skip_check) = token.skip_check {
        map.insert("skip_check".to_string(), serde_json::Value::Bool(skip_check));
    }

    serde_json::Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_data_creation() {
        let token = TokenData::new(
            "test_id".to_string(),
            "https://example.com".to_string(),
            "test_token".to_string(),
            Some("https://portal.example.com".to_string()),
            Some("test note".to_string()),
        );
        
        assert_eq!(token.id, "test_id");
        assert_eq!(token.tenant_url, "https://example.com");
        assert_eq!(token.access_token, "test_token");
        assert_eq!(token.portal_url, Some("https://portal.example.com".to_string()));
        assert_eq!(token.email_note, Some("test note".to_string()));
        assert!(token.created_at <= Utc::now());
        assert!(token.updated_at <= Utc::now());
    }

    #[test]
    fn test_legacy_conversion() {
        let legacy_json = serde_json::json!({
            "id": "test_id",
            "tenant_url": "https://example.com",
            "access_token": "test_token",
            "created_at": "2023-01-01T00:00:00Z",
            "portal_url": "https://portal.example.com",
            "email_note": "test note"
        });
        
        let token = convert_legacy_token(&legacy_json).unwrap();
        assert_eq!(token.id, "test_id");
        assert_eq!(token.tenant_url, "https://example.com");
        assert_eq!(token.access_token, "test_token");
        
        let converted_back = convert_to_legacy_format(&token);
        assert_eq!(converted_back["id"], "test_id");
        assert_eq!(converted_back["tenant_url"], "https://example.com");
        assert_eq!(converted_back["access_token"], "test_token");
    }
}
