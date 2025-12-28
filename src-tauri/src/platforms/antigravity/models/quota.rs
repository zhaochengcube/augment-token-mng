use serde::{Deserialize, Serialize};

/// 配额数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaData {
    pub models: Vec<ModelQuota>,
    pub last_updated: i64,
    pub is_forbidden: bool,
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
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
}

