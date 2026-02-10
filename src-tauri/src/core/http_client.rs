use crate::proxy_config::{ProxyConfig, ProxyType};
use crate::proxy_helper::ProxyClient;
use reqwest;
use serde_json;
use std::sync::Arc;

/// 尝试从配置文件加载代理配置
/// 路径与 Tauri 的 app_data_dir() 保持一致
fn try_load_proxy_config() -> Option<ProxyConfig> {
    // 根据不同平台构造应用数据目录
    // macOS: ~/Library/Application Support/com.cubezhao.atm
    // Linux: ~/.local/share/atm
    // Windows: C:\Users\<user>\AppData\Roaming\com.cubezhao.atm
    #[cfg(target_os = "macos")]
    let config_path = dirs::data_dir()?
        .join("com.cubezhao.atm")
        .join("proxy_config.json");

    #[cfg(target_os = "linux")]
    let config_path = dirs::data_local_dir()?
        .join("atm")
        .join("proxy_config.json");

    #[cfg(target_os = "windows")]
    let config_path = dirs::data_dir()?
        .join("com.cubezhao.atm")
        .join("proxy_config.json");

    if config_path.exists() {
        if let Ok(json) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<ProxyConfig>(&json) {
                return Some(config);
            }
        }
    }
    None
}

/// 检查是否配置了 CustomUrl 类型的代理
#[allow(dead_code)]
pub fn is_using_custom_url_proxy() -> bool {
    matches!(get_proxy_type(), Some(ProxyType::CustomUrl))
}

#[allow(dead_code)]
pub fn get_custom_proxy_url() -> Option<String> {
    match get_proxy_type() {
        Some(ProxyType::CustomUrl) => get_proxy_config().and_then(|c| c.custom_url.clone()),
        _ => None,
    }
}

/// 获取代理类型
fn get_proxy_type() -> Option<ProxyType> {
    if let Some(config) = try_load_proxy_config() {
        if config.enabled {
            return Some(config.proxy_type);
        }
    }
    None
}

/// 获取代理配置
fn get_proxy_config() -> Option<ProxyConfig> {
    if let Some(config) = try_load_proxy_config() {
        if config.enabled {
            return Some(config);
        }
    }
    None
}

/// 创建代理客户端，自动处理 Edge Function 代理
pub fn create_proxy_client() -> Result<ProxyClient, String> {
    let proxy_config = try_load_proxy_config();

    let client = if let Some(config) = &proxy_config {
        config.create_client()?
    } else {
        ProxyConfig::default().create_client()?
    };

    // 如果配置了 CustomUrl，获取 Edge Function URL
    let edge_function_url = if let Some(config) = proxy_config {
        if config.enabled && config.proxy_type == ProxyType::CustomUrl {
            config.custom_url
        } else {
            None
        }
    } else {
        None
    };

    Ok(ProxyClient::new(client, edge_function_url))
}

/// 创建标准的 HTTP 客户端，自动加载代理配置
///
/// 注意：对于 CustomUrl 类型（如 Supabase Edge Functions），
/// 返回的是普通客户端。调用方需要自行处理代理逻辑。
///
/// @deprecated 请使用 create_proxy_client() 来获得完整的 Edge Function 支持
pub fn create_http_client() -> Result<reqwest::Client, String> {
    let proxy_config = try_load_proxy_config();

    if let Some(config) = proxy_config {
        // 如果是 CustomUrl 类型，记录警告
        if config.enabled && config.proxy_type == ProxyType::CustomUrl {
            eprintln!(
                "Warning: CustomUrl proxy is configured but not automatically applied. Use create_proxy_client() instead."
            );
        }
        config.create_client()
    } else {
        // 如果没有配置文件，使用默认配置创建客户端
        // 默认配置: enabled=false, 不使用代理
        ProxyConfig::default().create_client()
    }
}

/// 创建带 Cookie 支持的 HTTP 客户端，自动加载代理配置
pub fn create_http_client_with_cookies(
    jar: Arc<reqwest::cookie::Jar>,
) -> Result<reqwest::Client, String> {
    let proxy_config = try_load_proxy_config();

    if let Some(config) = proxy_config {
        config.create_client_with_cookies(jar)
    } else {
        // 如果没有配置文件，使用默认配置创建客户端
        // 默认配置: enabled=false, 不使用代理
        ProxyConfig::default().create_client_with_cookies(jar)
    }
}
