use crate::core::http_client::create_http_client;
use super::models::*;

const API_BASE: &str = "https://api.raindrop.io/rest/v1";
const PAGE_SIZE: usize = 50;

/// Raindrop.io API 客户端
pub struct RaindropClient {
    client: reqwest::Client,
    token: String,
}

impl RaindropClient {
    pub fn new(token: String) -> Result<Self, String> {
        let client = create_http_client()?;
        Ok(Self { client, token })
    }

    /// 获取所有集合（用于映射 collection_id → name/color）
    pub async fn fetch_collections(&self) -> Result<Vec<RaindropCollectionInfo>, String> {
        let url = format!("{}/collections", API_BASE);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch collections: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Raindrop API error {}: {}", status, body));
        }

        let data: CollectionsResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse collections response: {}", e))?;

        if !data.result {
            return Err("Raindrop API returned result=false for collections".to_string());
        }

        // 也获取子集合
        let child_url = format!("{}/collections/childrens", API_BASE);
        let child_resp = self
            .client
            .get(&child_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await;

        let mut all = data.items;
        if let Ok(resp) = child_resp {
            if resp.status().is_success() {
                if let Ok(child_data) = resp.json::<CollectionsResponse>().await {
                    if child_data.result {
                        all.extend(child_data.items);
                    }
                }
            }
        }

        Ok(all)
    }

    /// 分页获取书签（collectionId=0 表示全部），按 lastUpdate 降序排列
    /// 如果 since 不为 None，遇到 last_update <= since 的项时停止分页（增量同步）
    pub async fn fetch_all_raindrops(
        &self,
        since: Option<&str>,
    ) -> Result<Vec<RaindropItem>, String> {
        let since_ts = since.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.timestamp())
                .ok()
        });

        let mut all_items = Vec::new();
        let mut page = 0;
        let mut should_stop = false;

        loop {
            let url = format!(
                "{}/raindrops/0?perpage={}&page={}&sort=-lastUpdate",
                API_BASE, PAGE_SIZE, page
            );

            let resp = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .send()
                .await
                .map_err(|e| format!("Failed to fetch raindrops page {}: {}", page, e))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("Raindrop API error {}: {}", status, body));
            }

            let data: RaindropsResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse raindrops response: {}", e))?;

            if !data.result {
                return Err("Raindrop API returned result=false".to_string());
            }

            let count = data.items.len();

            for item in data.items {
                // 增量同步：如果该项的 last_update 早于上次同步时间，停止
                if let Some(since_ts) = since_ts {
                    if let Some(ref last_update) = item.last_update {
                        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(last_update) {
                            if dt.timestamp() <= since_ts {
                                should_stop = true;
                                break;
                            }
                        }
                    }
                }
                all_items.push(item);
            }

            if should_stop || count < PAGE_SIZE {
                break;
            }

            page += 1;

            // 安全限制：最多获取 100 页（5000 条）
            if page >= 100 {
                eprintln!("Warning: Raindrop sync reached page limit (100 pages)");
                break;
            }
        }

        Ok(all_items)
    }

    /// 验证 token 是否有效（通过获取用户信息）
    pub async fn validate_token(&self) -> Result<bool, String> {
        let url = format!("{}/user", API_BASE);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await
            .map_err(|e| format!("Failed to validate token: {}", e))?;

        Ok(resp.status().is_success())
    }
}
