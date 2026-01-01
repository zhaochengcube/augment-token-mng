use serde::{Deserialize, Serialize};

/// 配额数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaData {
    pub models: Vec<ModelQuota>,
    pub last_updated: i64,
    pub is_forbidden: bool,
    #[serde(default)]
    pub subscription_tier: Option<String>,
}

/// 单个模型的配额信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelQuota {
    pub name: String,
    pub percentage: i32,
    pub reset_time: String,
}

impl QuotaData {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            last_updated: chrono::Utc::now().timestamp(),
            is_forbidden: false,
            subscription_tier: None,
        }
    }

    pub fn add_model(&mut self, name: String, percentage: i32, reset_time: String) {
        self.models.push(ModelQuota {
            name,
            percentage,
            reset_time,
        });
    }
}

impl Default for QuotaData {
    fn default() -> Self {
        Self::new()
    }
}

/// API 响应结构
#[derive(Debug, Deserialize)]
pub struct QuotaResponse {
    pub models: std::collections::HashMap<String, ModelInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    #[serde(rename = "quotaInfo")]
    pub quota_info: Option<QuotaInfo>,
}

#[derive(Debug, Deserialize)]
pub struct QuotaInfo {
    #[serde(rename = "remainingFraction")]
    pub remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    pub reset_time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoadProjectResponse {
    #[serde(rename = "cloudaicompanionProject", alias = "projectId")]
    pub project_id: Option<String>,
    #[serde(rename = "currentTier")]
    pub current_tier: Option<Tier>,
    #[serde(rename = "paidTier")]
    pub paid_tier: Option<Tier>,
}

#[derive(Debug, Deserialize)]
pub struct Tier {
    pub id: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "quotaTier")]
    pub quota_tier: Option<String>,
    #[allow(dead_code)]
    pub name: Option<String>,
    #[allow(dead_code)]
    pub slug: Option<String>,
}
