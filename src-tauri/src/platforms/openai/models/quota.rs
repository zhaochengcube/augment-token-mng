use serde::{Deserialize, Serialize};

/// OpenAI 配额数据
///
/// 通过调用 ChatGPT Codex API 获取配额信息
/// API 端点: https://chatgpt.com/backend-api/codex/responses
/// 配额信息从响应头中提取
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaData {
    /// 5小时窗口使用率百分比 (0-100)
    #[serde(rename = "codex_5h_used_percent", skip_serializing_if = "Option::is_none")]
    pub codex_5h_used_percent: Option<f64>,

    /// 5小时窗口重置倒计时（秒）
    #[serde(rename = "codex_5h_reset_after_seconds", skip_serializing_if = "Option::is_none")]
    pub codex_5h_reset_after_seconds: Option<i64>,

    /// 5小时窗口时长（分钟）
    #[serde(rename = "codex_5h_window_minutes", skip_serializing_if = "Option::is_none")]
    pub codex_5h_window_minutes: Option<i64>,

    /// 7天窗口使用率百分比 (0-100)
    #[serde(rename = "codex_7d_used_percent", skip_serializing_if = "Option::is_none")]
    pub codex_7d_used_percent: Option<f64>,

    /// 7天窗口重置倒计时（秒）
    #[serde(rename = "codex_7d_reset_after_seconds", skip_serializing_if = "Option::is_none")]
    pub codex_7d_reset_after_seconds: Option<i64>,

    /// 7天窗口时长（分钟）
    #[serde(rename = "codex_7d_window_minutes", skip_serializing_if = "Option::is_none")]
    pub codex_7d_window_minutes: Option<i64>,

    /// 主窗口超过次要窗口限制百分比
    #[serde(rename = "codex_primary_over_secondary_percent", skip_serializing_if = "Option::is_none")]
    pub codex_primary_over_secondary_percent: Option<f64>,

    /// 配额信息最后更新时间
    #[serde(rename = "codex_usage_updated_at")]
    pub codex_usage_updated_at: i64,

    /// 是否为禁用账户（如 403 禁止访问）
    #[serde(default)]
    pub is_forbidden: bool,
}

impl QuotaData {
    pub fn new() -> Self {
        Self {
            codex_5h_used_percent: None,
            codex_5h_reset_after_seconds: None,
            codex_5h_window_minutes: None,
            codex_7d_used_percent: None,
            codex_7d_reset_after_seconds: None,
            codex_7d_window_minutes: None,
            codex_primary_over_secondary_percent: None,
            codex_usage_updated_at: chrono::Utc::now().timestamp(),
            is_forbidden: false,
        }
    }

    /// 从响应头解析配额信息
    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Option<Self> {
        let mut quota = Self::new();

        // 解析主窗口（通常是5小时窗口）
        if let Some(value) = headers.get("x-codex-primary-used-percent") {
            if let Ok(percent) = value.to_str() {
                quota.codex_5h_used_percent = percent.parse().ok();
            }
        }
        if let Some(value) = headers.get("x-codex-primary-reset-after-seconds") {
            if let Ok(seconds) = value.to_str() {
                quota.codex_5h_reset_after_seconds = seconds.parse().ok();
            }
        }
        if let Some(value) = headers.get("x-codex-primary-window-minutes") {
            if let Ok(minutes) = value.to_str() {
                quota.codex_5h_window_minutes = minutes.parse().ok();
            }
        }

        // 解析次要窗口（通常是7天窗口）
        if let Some(value) = headers.get("x-codex-secondary-used-percent") {
            if let Ok(percent) = value.to_str() {
                quota.codex_7d_used_percent = percent.parse().ok();
            }
        }
        if let Some(value) = headers.get("x-codex-secondary-reset-after-seconds") {
            if let Ok(seconds) = value.to_str() {
                quota.codex_7d_reset_after_seconds = seconds.parse().ok();
            }
        }
        if let Some(value) = headers.get("x-codex-secondary-window-minutes") {
            if let Ok(minutes) = value.to_str() {
                quota.codex_7d_window_minutes = minutes.parse().ok();
            }
        }

        // 解析主窗口超过次要窗口限制的百分比
        if let Some(value) = headers.get("x-codex-primary-over-secondary-limit-percent") {
            if let Ok(percent) = value.to_str() {
                quota.codex_primary_over_secondary_percent = percent.parse().ok();
            }
        }

        let primary_used_percent = headers
            .get("x-codex-primary-used-percent")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());
        let primary_reset_after_seconds = headers
            .get("x-codex-primary-reset-after-seconds")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());
        let primary_window_minutes = headers
            .get("x-codex-primary-window-minutes")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());

        let secondary_used_percent = headers
            .get("x-codex-secondary-used-percent")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());
        let secondary_reset_after_seconds = headers
            .get("x-codex-secondary-reset-after-seconds")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());
        let secondary_window_minutes = headers
            .get("x-codex-secondary-window-minutes")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());

        quota.codex_5h_used_percent = None;
        quota.codex_5h_reset_after_seconds = None;
        quota.codex_5h_window_minutes = None;
        quota.codex_7d_used_percent = None;
        quota.codex_7d_reset_after_seconds = None;
        quota.codex_7d_window_minutes = None;

        let has_primary_window = primary_window_minutes.is_some();
        let has_secondary_window = secondary_window_minutes.is_some();
        let mut use5h_from_primary = false;
        let mut use7d_from_primary = false;
        let mut use5h_from_secondary = false;
        let mut use7d_from_secondary = false;

        if has_primary_window && has_secondary_window {
            let primary = primary_window_minutes.unwrap_or(0);
            let secondary = secondary_window_minutes.unwrap_or(0);
            if primary <= secondary {
                use5h_from_primary = true;
                use7d_from_secondary = true;
            } else {
                use5h_from_secondary = true;
                use7d_from_primary = true;
            }
        } else if has_primary_window {
            let primary = primary_window_minutes.unwrap_or(0);
            if primary <= 360 {
                use5h_from_primary = true;
            } else {
                use7d_from_primary = true;
            }
        } else if has_secondary_window {
            let secondary = secondary_window_minutes.unwrap_or(0);
            if secondary <= 360 {
                use5h_from_secondary = true;
            } else {
                use7d_from_secondary = true;
            }
        } else {
            if primary_used_percent.is_some()
                || primary_reset_after_seconds.is_some()
                || primary_window_minutes.is_some()
            {
                use7d_from_primary = true;
            }
            if secondary_used_percent.is_some()
                || secondary_reset_after_seconds.is_some()
                || secondary_window_minutes.is_some()
            {
                use5h_from_secondary = true;
            }
        }

        if use5h_from_primary {
            quota.codex_5h_used_percent = primary_used_percent;
            quota.codex_5h_reset_after_seconds = primary_reset_after_seconds;
            quota.codex_5h_window_minutes = primary_window_minutes;
        } else if use5h_from_secondary {
            quota.codex_5h_used_percent = secondary_used_percent;
            quota.codex_5h_reset_after_seconds = secondary_reset_after_seconds;
            quota.codex_5h_window_minutes = secondary_window_minutes;
        }

        if use7d_from_primary {
            quota.codex_7d_used_percent = primary_used_percent;
            quota.codex_7d_reset_after_seconds = primary_reset_after_seconds;
            quota.codex_7d_window_minutes = primary_window_minutes;
        } else if use7d_from_secondary {
            quota.codex_7d_used_percent = secondary_used_percent;
            quota.codex_7d_reset_after_seconds = secondary_reset_after_seconds;
            quota.codex_7d_window_minutes = secondary_window_minutes;
        }

        // 检查是否有任何配额数据
        if quota.codex_5h_used_percent.is_some()
            || quota.codex_7d_used_percent.is_some()
        {
            Some(quota)
        } else {
            None
        }
    }

    /// 检查配额数据是否有效
    pub fn is_valid(&self) -> bool {
        self.codex_5h_used_percent.is_some() || self.codex_7d_used_percent.is_some()
    }
}

impl Default for QuotaData {
    fn default() -> Self {
        Self::new()
    }
}
