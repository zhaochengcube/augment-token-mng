//! Codex 号池管理器
//!
//! 管理用于 Codex API 请求的 OAuth 账号池，支持轮询和单个选择策略

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::models::{CodexPoolAccount, PoolStatus, PoolStrategy};

/// Codex 号池管理器
pub struct CodexPool {
    accounts: Arc<RwLock<Vec<CodexPoolAccount>>>,
    current_index: Arc<RwLock<usize>>,
    request_counter: Arc<RwLock<u64>>,
    strategy: Arc<RwLock<PoolStrategy>>,
    selected_account_id: Arc<RwLock<Option<String>>>, // Single 策略时选中的账号 ID
}

impl CodexPool {
    /// 创建新的号池
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(Vec::new())),
            current_index: Arc::new(RwLock::new(0)),
            request_counter: Arc::new(RwLock::new(0)),
            strategy: Arc::new(RwLock::new(PoolStrategy::RoundRobin)),
            selected_account_id: Arc::new(RwLock::new(None)),
        }
    }

    /// 从 OpenAI OAuth 账号列表初始化号池
    pub async fn init_from_accounts(&self, accounts: &[crate::platforms::openai::models::Account]) {
        let mut pool = self.accounts.write().await;
        *pool = accounts
            .iter()
            .filter_map(|acc| CodexPoolAccount::from_openai_account(acc))
            .collect();
    }

    /// 添加账号到号池
    pub async fn add_account(&self, account: CodexPoolAccount) {
        let mut pool = self.accounts.write().await;
        pool.push(account);
    }

    /// 移除账号
    pub async fn remove_account(&self, account_id: &str) -> bool {
        let mut pool = self.accounts.write().await;
        if let Some(pos) = pool.iter().position(|a| a.id == account_id) {
            pool.remove(pos);
            return true;
        }
        false
    }

    /// 更新账号（刷新 token 后）
    pub async fn update_account(&self, updated: &CodexPoolAccount) -> bool {
        let mut pool = self.accounts.write().await;
        if let Some(account) = pool.iter_mut().find(|a| a.id == updated.id) {
            *account = updated.clone();
            return true;
        }
        false
    }

    /// 设置选择策略
    pub async fn set_strategy(&self, strategy: PoolStrategy) {
        let mut s = self.strategy.write().await;
        *s = strategy;
    }

    /// 设置 Single 策略选中的账号
    pub async fn set_selected_account_id(&self, account_id: String) {
        let mut s = self.selected_account_id.write().await;
        *s = Some(account_id);
    }

    /// 获取选中的账号 ID
    pub async fn get_selected_account_id(&self) -> Option<String> {
        self.selected_account_id.read().await.clone()
    }

    /// 获取下一个可用账号
    pub async fn next_account(&self) -> Option<CodexPoolAccount> {
        let pool = self.accounts.read().await;
        if pool.is_empty() {
            return None;
        }

        let strategy = *self.strategy.read().await;

        match strategy {
            PoolStrategy::RoundRobin => self.select_round_robin(&pool).await,
            PoolStrategy::Single => self.select_single(&pool).await,
            PoolStrategy::Smart => self.select_smart(&pool).await,
        }
    }

    /// 轮询选择
    async fn select_round_robin(&self, pool: &[CodexPoolAccount]) -> Option<CodexPoolAccount> {
        let mut index = self.current_index.write().await;

        // 过滤出可用账号
        let active_accounts: Vec<_> = pool.iter().filter(|a| a.is_available()).collect();

        if active_accounts.is_empty() {
            return None;
        }

        // 找到当前索引对应的账号
        let account = loop {
            let acc = pool.get(*index)?;
            if acc.is_available() {
                break acc;
            }
            *index = (*index + 1) % pool.len();
        };

        *index = (*index + 1) % pool.len();

        // 增加请求计数
        let mut counter = self.request_counter.write().await;
        *counter += 1;

        Some(account.clone())
    }

    /// 单个账号选择
    async fn select_single(&self, pool: &[CodexPoolAccount]) -> Option<CodexPoolAccount> {
        let selected_id = self.selected_account_id.read().await.clone();

        let account = if let Some(id) = selected_id {
            pool.iter().find(|a| a.id == id && a.is_available()).cloned()
        } else {
            // 没有选中账号时，使用第一个可用账号
            pool.iter().find(|a| a.is_available()).cloned()
        };

        if account.is_some() {
            // 增加请求计数
            let mut counter = self.request_counter.write().await;
            *counter += 1;
        }

        account
    }

    /// 智能选号：按打分从高到低选择，同分随机打破平衡
    async fn select_smart(&self, pool: &[CodexPoolAccount]) -> Option<CodexPoolAccount> {
        let available: Vec<&CodexPoolAccount> =
            pool.iter().filter(|a| a.is_available()).collect();

        if available.is_empty() {
            return None;
        }

        // 计算每个账号的分数
        let mut scored: Vec<(f64, usize)> = available
            .iter()
            .enumerate()
            .map(|(i, a)| (a.compute_score(), i))
            .collect();

        // 按分数降序排列
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        // 取最高分的一批（分差在 1 分以内的视为同档），从中轮询选一个避免全打到同一个号
        let top_score = scored[0].0;
        let top_tier: Vec<usize> = scored
            .iter()
            .filter(|(s, _)| top_score - s < 1.0)
            .map(|(_, i)| *i)
            .collect();

        let mut index = self.current_index.write().await;
        let pick = top_tier[*index % top_tier.len()];
        *index = index.wrapping_add(1);

        let mut counter = self.request_counter.write().await;
        *counter += 1;

        Some(available[pick].clone())
    }

    /// 记录 token 使用
    pub async fn record_usage(&self, account_id: &str, tokens: i64) {
        let mut pool = self.accounts.write().await;
        if let Some(account) = pool.iter_mut().find(|a| a.id == account_id) {
            account.add_usage(tokens);
        }
    }

    /// 记录账号请求失败并进入冷却
    /// 返回 Some(account_id) 如果账号被标记为 forbidden
    pub async fn record_failure(&self, account_id: &str, status_code: Option<u16>) -> Option<String> {
        let mut pool = self.accounts.write().await;
        let Some(account) = pool.iter_mut().find(|a| a.id == account_id) else {
            return None;
        };

        let now = chrono::Utc::now().timestamp();
        account.last_error_status = status_code;
        // 仅对明确与账号状态相关的错误做处理
        match status_code {
            Some(401) => {
                account.unavailable_reason = Some("unauthorized".to_string());
                account.cooldown_until = Some(now + 30 * 60);
                None
            }
            Some(402) | Some(403) => {
                // 标记为 forbidden，不再使用该账号
                account.is_forbidden = true;
                account.unavailable_reason = Some("payment_required".to_string());
                Some(account_id.to_string())
            }
            Some(429) => {
                account.unavailable_reason = Some("quota".to_string());
                account.cooldown_until = Some(now + 5 * 60);
                None
            }
            _ => {
                account.unavailable_reason = None;
                account.cooldown_until = None;
                None
            }
        }
    }

    /// 请求成功后清理账号错误状态
    pub async fn record_success(&self, account_id: &str) {
        let mut pool = self.accounts.write().await;
        if let Some(account) = pool.iter_mut().find(|a| a.id == account_id) {
            account.is_active = true;
            account.last_error_status = None;
            account.unavailable_reason = None;
            account.cooldown_until = None;
        }
    }

    /// 获取所有账号
    pub async fn get_accounts(&self) -> Vec<CodexPoolAccount> {
        self.accounts.read().await.clone()
    }

    /// 获取账号
    pub async fn get_account(&self, account_id: &str) -> Option<CodexPoolAccount> {
        let pool = self.accounts.read().await;
        pool.iter().find(|a| a.id == account_id).cloned()
    }

    /// 获取状态
    pub async fn status(&self) -> PoolStatus {
        let pool = self.accounts.read().await;
        let counter = self.request_counter.read().await;
        let strategy = *self.strategy.read().await;
        let selected_id = self.selected_account_id.read().await.clone();

        let total = pool.len();
        let expired = pool.iter().filter(|a| a.is_expired()).count();
        let cooling = pool
            .iter()
            .filter(|a| !a.is_expired() && a.is_in_cooldown())
            .count();
        let unauthorized = pool
            .iter()
            .filter(|a| {
                !a.is_expired()
                    && a.is_in_cooldown()
                    && a.unavailable_reason.as_deref() == Some("unauthorized")
            })
            .count();
        let payment_required = pool
            .iter()
            .filter(|a| {
                !a.is_expired()
                    && a.is_in_cooldown()
                    && a.unavailable_reason.as_deref() == Some("payment_required")
            })
            .count();
        let active = total.saturating_sub(expired + cooling);

        let total_tokens: i64 = pool.iter().map(|a| a.total_tokens_used).sum();

        // 获取选中账号的邮箱
        let selected_account_email = if matches!(strategy, PoolStrategy::Single) {
            if let Some(id) = &selected_id {
                pool.iter()
                    .find(|a| &a.id == id)
                    .map(|a| a.email.clone())
            } else {
                None
            }
        } else {
            None
        };

        PoolStatus {
            total_accounts: total,
            active_accounts: active,
            expired_accounts: expired,
            cooling_accounts: cooling,
            unauthorized_accounts: unauthorized,
            payment_required_accounts: payment_required,
            total_requests_today: *counter,
            total_tokens_used: total_tokens as u64,
            strategy,
            selected_account_email,
        }
    }

    /// 获取有效账号数量
    pub async fn active_count(&self) -> usize {
        let pool = self.accounts.read().await;
        pool.iter().filter(|a| a.is_available()).count()
    }

    /// 检查是否有可用账号
    pub async fn has_active_account(&self) -> bool {
        self.active_count().await > 0
    }

    /// 清理过期账号
    pub async fn cleanup_expired(&self) -> usize {
        let mut pool = self.accounts.write().await;
        let original_len = pool.len();
        pool.retain(|a| !a.is_expired());
        original_len - pool.len()
    }

    /// 刷新所有账号状态
    pub async fn refresh_from_accounts(
        &self,
        accounts: &[crate::platforms::openai::models::Account],
    ) {
        let mut pool = self.accounts.write().await;
        let existing: HashMap<String, CodexPoolAccount> =
            pool.iter().cloned().map(|a| (a.id.clone(), a)).collect();
        *pool = accounts
            .iter()
            .filter_map(|acc| {
                let mut next = CodexPoolAccount::from_openai_account(acc)?;
                if let Some(prev) = existing.get(&next.id) {
                    next.used_quota = prev.used_quota;
                    next.total_tokens_used = prev.total_tokens_used;
                    // 仅保留账号级问题导致的冷却；历史版本中的 network/transient 冷却在刷新时自动清理。
                    let keep_cooldown = matches!(
                        prev.unavailable_reason.as_deref(),
                        Some("unauthorized") | Some("payment_required") | Some("quota")
                    );
                    if keep_cooldown {
                        next.cooldown_until = prev.cooldown_until;
                        next.unavailable_reason = prev.unavailable_reason.clone();
                    } else {
                        next.cooldown_until = None;
                        next.unavailable_reason = None;
                    }
                    next.last_error_status = prev.last_error_status;
                    if prev.last_used.unwrap_or_default() > next.last_used.unwrap_or_default() {
                        next.last_used = prev.last_used;
                    }
                }
                Some(next)
            })
            .collect();
    }
}

impl Default for CodexPool {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Codex Server 状态 ====================

/// Codex API 服务器状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodexServerStatus {
    pub running: bool,
    pub port: u16,
    pub address: String,
    pub pool_status: Option<PoolStatus>,
}

/// Codex 服务器配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodexServerConfig {
    pub port: u16,
    #[serde(default = "default_codex_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub pool_strategy: String,
    #[serde(default)]
    pub selected_account_id: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
}

fn default_codex_enabled() -> bool {
    true
}

impl Default for CodexServerConfig {
    fn default() -> Self {
        Self {
            port: 8766,
            enabled: true,
            pool_strategy: "round-robin".to_string(),
            selected_account_id: None,
            api_key: None,
        }
    }
}
