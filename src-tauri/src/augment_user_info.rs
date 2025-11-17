use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::http_client::{create_http_client_with_cookies, create_proxy_client};
use std::time::SystemTime;

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
pub struct CompleteUserInfo {
    pub suspensions: Option<Value>,
    pub ban_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentMethodLinkResponse {
    pub success: bool,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub url: String,
}


/// 通过 auth session 交换 app session
pub async fn exchange_auth_session_for_app_session(auth_session: &str) -> Result<String, String> {
    use reqwest::cookie::{Jar, CookieStore};
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

    // 直接 GET /login 触发授权流
    let _login_response = client
        .get("https://app.augmentcode.com/login")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Failed to exchange session: {}", e))?;

    // 从 cookie jar 中获取 app.augmentcode.com 的 cookies
    let app_url = "https://app.augmentcode.com/".parse::<reqwest::Url>()
        .map_err(|e| format!("Failed to parse app URL: {}", e))?;

    if let Some(header_value) = jar.cookies(&app_url) {
        if let Ok(cookie_str) = header_value.to_str() {
            let cookie_string = cookie_str.to_string();
            // 提取 _session 的值
            if let Some(start) = cookie_string.find("_session=") {
                let session_part = &cookie_string[start + 9..];
                // 可能有多个 cookie，用分号分隔，只取第一个
                let session_value = session_part.split(';').next().unwrap_or(session_part);
                return Ok(urlencoding::decode(session_value)
                    .unwrap_or_else(|_| session_value.into())
                    .to_string());
            }
        }
    }

    Err("Failed to extract app session cookie".to_string())
}

/// 通过 auth session 交换完整的 Cookie 字符串(专门用于绑卡链接)
pub async fn exchange_auth_session_for_full_cookies(auth_session: &str) -> Result<String, String> {
    // 复用第一个方法获取 _session 值
    let session_value = exchange_auth_session_for_app_session(auth_session).await?;

    // 使用固定的tracking cookies模板,只替换 _session 值
    let full_cookie = format!(
        "_ga=GA1.1.242842833.1762031996; _fbp=fb.1.1762262295646.930716998848089033; _rdt_uuid=1762262295652.34187976-0dc0-4e33-8a70-645aa454b403; _gcl_au=1.1.1646447425.1762031996.1424920390.1763188023.1763188269; _ga_F6GPDJDCJY=GS2.1.s1763188002$o15$g1$t1763188269$j31$l0$h0; _session={}",
        urlencoding::encode(&session_value)
    );

    Ok(full_cookie)
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

/// 获取绑卡链接
/// 参数 cookie_string: 完整的Cookie字符串,包含 _session 和其他cookies
pub async fn fetch_payment_method_link(cookie_string: &str) -> Result<String, String> {
    // 使用 ProxyClient，自动处理 Edge Function
    let client = create_proxy_client()?;

    // 创建空的JSON body
    let body = "{}";

    let cookie_header_value = format!("Cookie={}", cookie_string);

    let response = client
        .post("https://app.augmentcode.com/api/setup-trial-payment-method")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .header("Cookie", cookie_header_value)
        .header("Content-Type", "application/json")
        .header("Content-Length", body.len().to_string())
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch payment method link: {}", e))?;

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response text: {}", e))?;

    let payment_response: PaymentMethodLinkResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse payment method link response: {}. Response was: {}", e, response_text))?;

    Ok(payment_response.url)
}


/// 使用已有的 app_session 获取完整的用户信息
pub async fn get_user_info_with_app_session(app_session: &str) -> Result<CompleteUserInfo, String> {
    // 只获取用户信息
    let user_info = fetch_app_user(app_session).await.ok();

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
        suspensions: user_info.and_then(|u| u.suspensions),
        ban_status,
    })
}

/// 获取完整的用户信息 (使用缓存的 app_session)
pub async fn get_user_info(
    auth_session: &str,
    app_session_cache: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, crate::AppSessionCache>>>,
) -> Result<CompleteUserInfo, String> {
    // 1. 检查缓存中是否有有效的 app_session
    let cached_app_session = {
        let cache = app_session_cache.lock().unwrap();
        cache.get(auth_session).map(|c| c.app_session.clone())
    };

    // 2. 尝试使用缓存的 app_session
    if let Some(app_session) = cached_app_session {
        match get_user_info_with_app_session(&app_session).await {
            Ok(user_info) => return Ok(user_info),
            Err(e) => println!("Cached app_session failed: {}, will refresh", e),
        }
    }

    // 3. 交换新的 app_session
    let app_session = exchange_auth_session_for_app_session(auth_session).await?;

    println!("App session obtained: {}", &app_session[..20.min(app_session.len())]);

    // 4. 更新缓存
    {
        let mut cache = app_session_cache.lock().unwrap();
        cache.insert(auth_session.to_string(), crate::AppSessionCache {
            app_session: app_session.clone(),
            created_at: SystemTime::now(),
        });
    }

    // 5. 获取用户信息
    get_user_info_with_app_session(&app_session).await
}

