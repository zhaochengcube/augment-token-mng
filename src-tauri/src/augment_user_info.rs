use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::http_client::{create_http_client_with_cookies, create_proxy_client};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub email: Option<String>,
    pub suspensions: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    #[serde(rename = "portalUrl")]
    pub portal_url: Option<String>,
    #[serde(rename = "billingPeriodEnd")]
    pub billing_period_end: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditsInfo {
    #[serde(rename = "usageUnitsAvailable")]
    pub usage_units_available: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteUserInfo {
    pub email_note: Option<String>,
    pub suspensions: Option<Value>,
    pub portal_url: Option<String>,
    pub ban_status: String,
}

/// 通过 auth session 交换 app session
pub async fn exchange_auth_session_for_app_session(auth_session: &str) -> Result<String, String> {
    use reqwest::cookie::Jar;
    use std::sync::Arc;

    // 创建 cookie jar
    let jar = Arc::new(Jar::default());

    // 设置 auth session cookie 到 auth.augmentcode.com 域
    let auth_url = "https://auth.augmentcode.com/".parse::<reqwest::Url>()
        .map_err(|e| format!("Failed to parse auth URL: {}", e))?;
    jar.add_cookie_str(
        &format!("session={}", auth_session),
        &auth_url
    );

    // 创建带 cookie store 的客户端
    let client = create_http_client_with_cookies(jar.clone())?;

    // 直接 GET /login 触发授权流,同时检查响应中的 cookies
    let login_response = client
        .get("https://app.augmentcode.com/login")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Failed to exchange session: {}", e))?;

    // 先尝试从 login 响应中提取 _session cookie
    for cookie in login_response.cookies() {
        if cookie.name() == "_session" {
            return Ok(urlencoding::decode(cookie.value())
                .unwrap_or_else(|_| cookie.value().into())
                .to_string());
        }
    }

    // 如果 login 响应中没有,再请求 /api/user
    let user_response = client
        .get("https://app.augmentcode.com/api/user")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Failed to get user info: {}", e))?;

    // 从 user 响应中提取 cookies
    for cookie in user_response.cookies() {
        if cookie.name() == "_session" {
            return Ok(urlencoding::decode(cookie.value())
                .unwrap_or_else(|_| cookie.value().into())
                .to_string());
        }
    }

    Err("Failed to extract app session cookie".to_string())
}

/// 获取用户信息
pub async fn fetch_app_user(app_session: &str) -> Result<UserInfo, String> {
    // 使用新的 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;
    let response = client
        .get("https://app.augmentcode.com/api/user")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user info: {}", e))?;

    response
        .json::<UserInfo>()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))
}

/// 获取订阅信息
pub async fn fetch_app_subscription(app_session: &str) -> Result<SubscriptionInfo, String> {
    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;
    let response = client
        .get("https://app.augmentcode.com/api/subscription")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch subscription info: {}", e))?;

    response
        .json::<SubscriptionInfo>()
        .await
        .map_err(|e| format!("Failed to parse subscription info: {}", e))
}

/// 获取积分信息
pub async fn fetch_app_credits(app_session: &str) -> Result<CreditsInfo, String> {
    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;
    let response = client
        .get("https://app.augmentcode.com/api/credits")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", format!("_session={}", urlencoding::encode(app_session)))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch credits info: {}", e))?;

    response
        .json::<CreditsInfo>()
        .await
        .map_err(|e| format!("Failed to parse credits info: {}", e))
}

/// 使用已有的 app_session 获取完整的用户信息
pub async fn get_user_info_with_app_session(app_session: &str) -> Result<CompleteUserInfo, String> {
    // 并发获取所有信息
    let (user_result, subscription_result, credits_result) = tokio::join!(
        fetch_app_user(app_session),
        fetch_app_subscription(app_session),
        fetch_app_credits(app_session)
    );

    let user_info = user_result.ok();
    let subscription_info: Option<SubscriptionInfo> = subscription_result.ok();
    let credits_info = credits_result.ok();

    // 计算 ban_status
    let ban_status = if let Some(ref user) = user_info {
        if let Some(ref suspensions) = user.suspensions {
            if let Some(arr) = suspensions.as_array() {
                if !arr.is_empty() {
                    if let Some(first) = arr.first() {
                        if let Some(suspension_type) = first.get("suspensionType").and_then(|v| v.as_str()) {
                            format!("BANNED-{}", suspension_type)
                        } else {
                            "BANNED".to_string()
                        }
                    } else {
                        "BANNED".to_string()
                    }
                } else {
                    "ACTIVE".to_string()
                }
            } else {
                "ACTIVE".to_string()
            }
        } else {
            "ACTIVE".to_string()
        }
    } else {
        "ACTIVE".to_string()
    };

    Ok(CompleteUserInfo {
        email_note: user_info.as_ref().and_then(|u| u.email.clone()),
        suspensions: user_info.and_then(|u| u.suspensions),
        portal_url: subscription_info.as_ref().and_then(|s| s.portal_url.clone()),
        ban_status,
    })
}

/// 获取完整的用户信息
pub async fn get_user_info(auth_session: &str) -> Result<CompleteUserInfo, String> {
    let app_session = exchange_auth_session_for_app_session(auth_session).await?;

    println!("App session obtained: {}", &app_session[..20.min(app_session.len())]);

    get_user_info_with_app_session(&app_session).await
}

