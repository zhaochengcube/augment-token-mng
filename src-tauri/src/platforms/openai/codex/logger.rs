//! 请求日志记录器
//!
//! 记录 Codex API 请求日志，包括 token 使用统计

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::models::{
    AccountTokenStats, LogPage, LogQuery, ModelTokenStats, PeriodTokenStats, RequestLog, TokenStats,
};

/// 请求日志记录器
pub struct RequestLogger {
    logs: Vec<RequestLog>,
    max_entries: usize,
}

impl RequestLogger {
    pub fn new(max_entries: usize) -> Self {
        Self {
            logs: Vec::with_capacity(max_entries),
            max_entries,
        }
    }

    /// 添加日志
    pub fn add_log(&mut self, log: RequestLog) {
        self.logs.push(log);

        // 超过限制时移除旧日志
        if self.logs.len() > self.max_entries {
            let remove_count = self.logs.len() - self.max_entries;
            for _ in 0..remove_count {
                self.logs.remove(0);
            }
        }
    }

    /// 获取最近的日志
    pub fn get_recent_logs(&self, limit: usize) -> Vec<RequestLog> {
        let start = if self.logs.len() > limit {
            self.logs.len() - limit
        } else {
            0
        };
        self.logs[start..].to_vec()
    }

    /// 获取统计信息
    pub fn get_stats(&self, start_date: &str, end_date: &str) -> Result<TokenStats, String> {
        let start_ts = parse_date(start_date)?;
        let end_ts = parse_date(end_date)?;

        let mut per_account: HashMap<String, AccountTokenStats> = HashMap::new();

        let mut total_requests = 0u64;
        let mut total_tokens = 0i64;

        for log in &self.logs {
            if log.timestamp >= start_ts && log.timestamp <= end_ts {
                total_requests += 1;
                total_tokens += log.total_tokens;

                let entry = per_account
                    .entry(log.account_id.clone())
                    .or_insert_with(|| AccountTokenStats {
                        account_id: log.account_id.clone(),
                        email: log.account_email.clone(),
                        requests: 0,
                        tokens: 0,
                    });
                entry.requests += 1;
                entry.tokens += log.total_tokens as u64;
            }
        }

        let mut stats_vec: Vec<_> = per_account.into_values().collect();
        stats_vec.sort_by(|a, b| b.tokens.cmp(&a.tokens));

        Ok(TokenStats {
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            total_requests,
            total_tokens: total_tokens as u64,
            per_account: stats_vec,
        })
    }

    pub fn get_model_stats(&self, start_ts: i64, end_ts: i64) -> Vec<ModelTokenStats> {
        let mut per_model: HashMap<String, ModelTokenStats> = HashMap::new();

        for log in &self.logs {
            if log.timestamp < start_ts || log.timestamp > end_ts {
                continue;
            }

            let model_key = if log.model.trim().is_empty() {
                "unknown".to_string()
            } else {
                log.model.clone()
            };

            let entry = per_model
                .entry(model_key.clone())
                .or_insert_with(|| ModelTokenStats {
                    model: model_key,
                    requests: 0,
                    input_tokens: 0,
                    output_tokens: 0,
                    total_tokens: 0,
                });
            entry.requests += 1;
            entry.input_tokens += log.input_tokens.max(0) as u64;
            entry.output_tokens += log.output_tokens.max(0) as u64;
            entry.total_tokens += log.total_tokens.max(0) as u64;
        }

        let mut models: Vec<_> = per_model.into_values().collect();
        models.sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));
        models
    }

    pub fn query_logs(&self, query: &LogQuery) -> LogPage {
        let mut filtered: Vec<RequestLog> = self
            .logs
            .iter()
            .filter(|log| matches_query(log, query))
            .cloned()
            .collect();

        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let total = filtered.len();
        let offset = query.offset.unwrap_or(0).min(total);
        let limit = query.limit.unwrap_or(100).max(1);
        let items = filtered.into_iter().skip(offset).take(limit).collect();

        LogPage { total, items }
    }

    pub fn get_period_stats(&self, now_ts: i64) -> PeriodTokenStats {
        let now = chrono::DateTime::from_timestamp(now_ts, 0)
            .unwrap_or_else(chrono::Utc::now)
            .with_timezone(&chrono::Utc);
        let today_start = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|d| d.and_utc().timestamp())
            .unwrap_or(0);
        let week_start = (now - chrono::Duration::days(6))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|d| d.and_utc().timestamp())
            .unwrap_or(0);
        let month_start = (now - chrono::Duration::days(29))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|d| d.and_utc().timestamp())
            .unwrap_or(0);

        let mut stats = PeriodTokenStats {
            today_requests: 0,
            today_tokens: 0,
            week_requests: 0,
            week_tokens: 0,
            month_requests: 0,
            month_tokens: 0,
        };

        for log in &self.logs {
            let ts = log.timestamp;
            let tokens = log.total_tokens.max(0) as u64;

            if ts >= month_start && ts <= now_ts {
                stats.month_requests += 1;
                stats.month_tokens += tokens;
            }
            if ts >= week_start && ts <= now_ts {
                stats.week_requests += 1;
                stats.week_tokens += tokens;
            }
            if ts >= today_start && ts <= now_ts {
                stats.today_requests += 1;
                stats.today_tokens += tokens;
            }
        }

        stats
    }

    /// 清空日志
    pub fn clear(&mut self) {
        self.logs.clear();
    }

    /// 获取所有日志数量
    pub fn len(&self) -> usize {
        self.logs.len()
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }
}

fn parse_date(date_str: &str) -> Result<i64, String> {
    // 尝试解析多种日期格式
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.timestamp());
    }

    // 尝试 YYYY-MM-DD 格式
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let midnight = chrono::NaiveTime::from_hms_opt(0, 0, 0)
            .ok_or_else(|| "Failed to build midnight time".to_string())?;
        return Ok(dt.and_time(midnight).and_utc().timestamp());
    }

    // 尝试时间戳
    if let Ok(ts) = date_str.parse::<i64>() {
        return Ok(ts);
    }

    Err(format!("Invalid date format: {}", date_str))
}

// ==================== TokenStats ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQueryParams {
    pub start_date: String,
    pub end_date: String,
    pub account_id: Option<String>,
    pub limit: Option<usize>,
}

fn matches_query(log: &RequestLog, query: &LogQuery) -> bool {
    if let Some(start_ts) = query.start_ts {
        if log.timestamp < start_ts {
            return false;
        }
    }
    if let Some(end_ts) = query.end_ts {
        if log.timestamp > end_ts {
            return false;
        }
    }

    if let Some(model) = &query.model {
        if !model.trim().is_empty() && !log.model.eq_ignore_ascii_case(model.trim()) {
            return false;
        }
    }
    if let Some(format) = &query.format {
        if !format.trim().is_empty() && !log.format.eq_ignore_ascii_case(format.trim()) {
            return false;
        }
    }
    if let Some(status) = &query.status {
        if !status.trim().is_empty() && !log.status.eq_ignore_ascii_case(status.trim()) {
            return false;
        }
    }
    if let Some(account_id) = &query.account_id {
        if !account_id.trim().is_empty() && log.account_id != account_id.trim() {
            return false;
        }
    }

    true
}
