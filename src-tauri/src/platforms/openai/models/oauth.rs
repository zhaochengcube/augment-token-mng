use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIOAuthSession {
    pub state: String,
    pub code_verifier: String,
    pub redirect_uri: String,
    pub created_at: SystemTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIAuthUrlResult {
    pub auth_url: String,
    pub session_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAITokenInfo {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_in: i64,
    pub expires_at: i64,
    pub email: Option<String>,
    pub chatgpt_account_id: Option<String>,
    pub chatgpt_user_id: Option<String>,
    pub organization_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIUserInfo {
    pub email: Option<String>,
    pub chatgpt_account_id: Option<String>,
    pub chatgpt_user_id: Option<String>,
    pub organization_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    pub sub: Option<String>,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub iss: Option<String>,
    pub aud: Option<Vec<String>>,
    pub exp: Option<i64>,
    pub iat: Option<i64>,
    #[serde(rename = "https://api.openai.com/auth")]
    pub openai_auth: Option<OpenAIAuthClaims>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIAuthClaims {
    pub chatgpt_account_id: Option<String>,
    pub chatgpt_user_id: Option<String>,
    pub user_id: Option<String>,
    pub organizations: Option<Vec<OrganizationClaim>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrganizationClaim {
    pub id: Option<String>,
    pub role: Option<String>,
    pub title: Option<String>,
    pub is_default: Option<bool>,
}

