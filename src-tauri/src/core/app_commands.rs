use arboard::Clipboard;
use chrono;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::AppState;
use crate::http_client;
use crate::platforms::augment::oauth::add_token_from_session_internal_with_cache;
use crate::proxy_config;

// Update check structures
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    current_version: String,
    latest_version: String,
    has_update: bool,
    download_url: String,
    release_notes: Option<String>,
}

// RSS Feed 解析结构
#[derive(Debug, Deserialize)]
struct Feed {
    entry: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Entry {
    id: String,
    title: String,
    link: Link,
    content: Option<Content>,
}

#[derive(Debug, Deserialize)]
struct Link {
    #[serde(rename = "@href")]
    href: String,
}

#[derive(Debug, Deserialize)]
struct Content {
    #[serde(rename = "$value")]
    value: String,
}

// Version comparison helper
fn compare_versions(current: &str, latest: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let current_parts = parse_version(current);
    let latest_parts = parse_version(latest);

    for i in 0..std::cmp::max(current_parts.len(), latest_parts.len()) {
        let current_part = current_parts.get(i).unwrap_or(&0);
        let latest_part = latest_parts.get(i).unwrap_or(&0);

        if latest_part > current_part {
            return true;
        } else if latest_part < current_part {
            return false;
        }
    }

    false
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");

    // 使用 GitHub RSS Feed，避免 API 速率限制
    let client = http_client::create_proxy_client()?;
    let response = client
        .get("https://github.com/cubezhao/ai-tools-mng/releases.atom")
        .header("User-Agent", "Mozilla/5.0 (compatible; ATM-App/1.0)")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch RSS feed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub RSS feed returned status: {}",
            response.status()
        ));
    }

    let xml_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read RSS feed: {}", e))?;

    // 解析 XML
    let feed: Feed = quick_xml::de::from_str(&xml_text)
        .map_err(|e| format!("Failed to parse RSS feed: {}", e))?;

    // 获取第一个 entry (最新版本)
    let latest_entry = feed.entry.first().ok_or("No releases found in RSS feed")?;

    // 从 id 中提取版本号 (格式: tag:github.com,2008:Repository/.../v1.2.3)
    let latest_version = latest_entry
        .id
        .split('/')
        .last()
        .ok_or("Invalid release ID format")?
        .trim_start_matches('v');

    let has_update = compare_versions(current_version, latest_version);

    // 构建 GitHub Release 页面 URL
    let download_url = latest_entry.link.href.clone();

    // 从 content 中提取 release notes (如果有)
    let release_notes = latest_entry.content.as_ref().map(|c| c.value.clone());

    Ok(UpdateInfo {
        current_version: current_version.to_string(),
        latest_version: latest_version.to_string(),
        has_update,
        download_url,
        release_notes,
    })
}

#[tauri::command]
pub async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("Failed to set clipboard text: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn open_url(app: AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))
}

#[tauri::command]
pub async fn open_internal_browser(
    app: AppHandle,
    url: String,
    title: Option<String>,
) -> Result<String, String> {
    use std::time::Duration;
    use tauri::webview::PageLoadEvent;

    // 加载代理配置
    let proxy_config = proxy_config::load_proxy_config(app.clone()).await.ok();

    let window_label = format!("browser_{}", chrono::Utc::now().timestamp());
    let app_handle = app.clone();

    let mut builder = WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::External(url.parse().map_err(|e| format!("Invalid URL: {}", e))?),
    )
    .title(&title.unwrap_or_else(|| "内置浏览器".to_string()))
    .inner_size(1000.0, 700.0)
    .center()
    .resizable(true)
    .incognito(true); // 无痕模式,关闭后自动清除所有数据

    // 如果有代理配置，应用代理
    if let Some(config) = proxy_config {
        if config.enabled {
            if let Some(proxy_url_str) = config.build_proxy_url() {
                // Tauri WebView 只支持 http:// 和 socks5:// 代理
                // 将 https:// 转换为 http://（HTTPS 代理实际上也是通过 HTTP CONNECT 工作的）
                let normalized_proxy_url = if proxy_url_str.starts_with("https://") {
                    proxy_url_str.replace("https://", "http://")
                } else {
                    proxy_url_str.clone()
                };

                if normalized_proxy_url.starts_with("http://")
                    || normalized_proxy_url.starts_with("socks5://")
                {
                    match normalized_proxy_url.parse::<url::Url>() {
                        Ok(proxy_url) => {
                            builder = builder.proxy_url(proxy_url);
                            eprintln!("WebView proxy configured: {}", normalized_proxy_url);
                        }
                        Err(e) => {
                            eprintln!("Failed to parse proxy URL: {}", e);
                        }
                    }
                } else {
                    eprintln!(
                        "WebView only supports http:// and socks5:// proxies, got: {}",
                        normalized_proxy_url
                    );
                }
            } else {
                eprintln!("Proxy enabled but no proxy URL configured (System proxy or CustomUrl)");
            }
        }
    }

    let _window = builder
    .initialization_script(r#"
        console.log('[Tauri] Initialization script loaded');

        // 复制 URL 的函数
        async function copyCurrentUrl() {
            try {
                const currentUrl = window.location.href;
                await navigator.clipboard.writeText(currentUrl);

                // 显示复制成功提示
                showCopyNotification('✅ URL 已复制!', '#10b981');
                console.log('[Tauri] URL copied:', currentUrl);
            } catch (error) {
                console.error('[Tauri] Failed to copy URL:', error);
                showCopyNotification('❌ 复制失败', '#ef4444');
            }
        }

        // 显示复制通知的函数
        function showCopyNotification(message, bgColor) {
            // 移除已存在的通知
            const existingNotification = document.getElementById('tauri-copy-notification');
            if (existingNotification) {
                existingNotification.remove();
            }

            // 创建新通知
            const notification = document.createElement('div');
            notification.id = 'tauri-copy-notification';
            notification.textContent = message;
            notification.style.cssText = `
                position: fixed;
                top: 20px;
                right: 20px;
                background: ${bgColor};
                color: white;
                padding: 12px 24px;
                border-radius: 8px;
                font-size: 14px;
                font-weight: 500;
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
                box-shadow: 0 4px 12px rgba(0,0,0,0.15);
                z-index: 2147483647;
                animation: slideIn 0.3s ease-out;
            `;

            // 添加动画样式
            if (!document.getElementById('tauri-notification-style')) {
                const style = document.createElement('style');
                style.id = 'tauri-notification-style';
                style.textContent = `
                    @keyframes slideIn {
                        from {
                            transform: translateX(400px);
                            opacity: 0;
                        }
                        to {
                            transform: translateX(0);
                            opacity: 1;
                        }
                    }
                    @keyframes slideOut {
                        from {
                            transform: translateX(0);
                            opacity: 1;
                        }
                        to {
                            transform: translateX(400px);
                            opacity: 0;
                        }
                    }
                `;
                document.head.appendChild(style);
            }

            document.body.appendChild(notification);

            // 2秒后移除通知
            setTimeout(() => {
                notification.style.animation = 'slideOut 0.3s ease-out';
                setTimeout(() => notification.remove(), 300);
            }, 2000);
        }

        // 注册快捷键监听器
        document.addEventListener('keydown', function(event) {
            // 检测 Ctrl+Shift+C (Windows/Linux) 或 Cmd+Shift+C (Mac)
            const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
            const modifierKey = isMac ? event.metaKey : event.ctrlKey;

            if (modifierKey && event.shiftKey && event.key.toLowerCase() === 'c') {
                event.preventDefault(); // 阻止默认行为
                copyCurrentUrl();
                console.log('[Tauri] Keyboard shortcut triggered: ' + (isMac ? 'Cmd' : 'Ctrl') + '+Shift+C');
            }
        });

        console.log('[Tauri] Keyboard shortcut registered: Ctrl+Shift+C (Windows/Linux) or Cmd+Shift+C (Mac)');

        // 随机地址数据生成函数
        function generateRandomAddress() {
            // 美国免税州数据
            const taxFreeStates = [
                {
                    state: 'DE',
                    cities: [
                        { city: 'Wilmington', zip: '19801' },
                        { city: 'Dover', zip: '19901' },
                        { city: 'Newark', zip: '19711' },
                        { city: 'Middletown', zip: '19709' },
                        { city: 'Bear', zip: '19701' }
                    ]
                },
                {
                    state: 'MT',
                    cities: [
                        { city: 'Billings', zip: '59101' },
                        { city: 'Missoula', zip: '59801' },
                        { city: 'Great Falls', zip: '59401' },
                        { city: 'Bozeman', zip: '59715' },
                        { city: 'Helena', zip: '59601' }
                    ]
                },
                {
                    state: 'NH',
                    cities: [
                        { city: 'Manchester', zip: '03101' },
                        { city: 'Nashua', zip: '03060' },
                        { city: 'Concord', zip: '03301' },
                        { city: 'Derry', zip: '03038' },
                        { city: 'Dover', zip: '03820' }
                    ]
                },
                {
                    state: 'OR',
                    cities: [
                        { city: 'Portland', zip: '97201' },
                        { city: 'Eugene', zip: '97401' },
                        { city: 'Salem', zip: '97301' },
                        { city: 'Gresham', zip: '97030' },
                        { city: 'Hillsboro', zip: '97123' }
                    ]
                },
                {
                    state: 'AK',
                    cities: [
                        { city: 'Anchorage', zip: '99501' },
                        { city: 'Fairbanks', zip: '99701' },
                        { city: 'Juneau', zip: '99801' },
                        { city: 'Sitka', zip: '99835' },
                        { city: 'Ketchikan', zip: '99901' }
                    ]
                }
            ];

            const firstNames = [
                'James', 'John', 'Robert', 'Michael', 'William',
                'David', 'Richard', 'Joseph', 'Thomas', 'Charles',
                'Mary', 'Patricia', 'Jennifer', 'Linda', 'Barbara',
                'Elizabeth', 'Susan', 'Jessica', 'Sarah', 'Karen',
                'Daniel', 'Matthew', 'Anthony', 'Mark', 'Donald',
                'Steven', 'Paul', 'Andrew', 'Joshua', 'Kenneth'
            ];

            const lastNames = [
                'Smith', 'Johnson', 'Williams', 'Brown', 'Jones',
                'Garcia', 'Miller', 'Davis', 'Rodriguez', 'Martinez',
                'Hernandez', 'Lopez', 'Gonzalez', 'Wilson', 'Anderson',
                'Thomas', 'Taylor', 'Moore', 'Jackson', 'Martin',
                'Lee', 'Perez', 'Thompson', 'White', 'Harris',
                'Sanchez', 'Clark', 'Ramirez', 'Lewis', 'Robinson'
            ];

            const streetNames = [
                'Main St', 'Oak Ave', 'Maple Dr', 'Cedar Ln', 'Pine St',
                'Elm St', 'Washington Ave', 'Park Ave', 'Lake Dr', 'Hill Rd',
                'Forest Ave', 'River Rd', 'Sunset Blvd', 'Broadway', 'Market St',
                'Church St', 'Spring St', 'Center St', 'High St', 'School St'
            ];

            // 随机选择
            const randomInt = (max) => Math.floor(Math.random() * max);

            const firstName = firstNames[randomInt(firstNames.length)];
            const lastName = lastNames[randomInt(lastNames.length)];

            const stateData = taxFreeStates[randomInt(taxFreeStates.length)];
            const cityData = stateData.cities[randomInt(stateData.cities.length)];

            const streetNumber = 100 + randomInt(9899);
            const streetName = streetNames[randomInt(streetNames.length)];
            const street = streetNumber + ' ' + streetName;

            return {
                firstName: firstName,
                lastName: lastName,
                fullName: firstName + ' ' + lastName,
                street: street,
                city: cityData.city,
                state: stateData.state,
                zipCode: cityData.zip
            };
        }

        // 自动填充地址函数
        function autoFillAddress() {
            console.log('[Tauri] Auto-filling address...');

            const addressData = generateRandomAddress();
            console.log('[Tauri] Generated address:', addressData);

            let filledCount = 0;

            // 辅助函数：填充输入框
            function fillInput(selector, value) {
                const element = document.querySelector(selector);
                if (element) {
                    try {
                        // 使用原生 value setter，兼容 React/受控输入
                        const tag = (element.tagName || '').toUpperCase();
                        const inputSetter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value')?.set;
                        const textareaSetter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value')?.set;
                        if (tag === 'INPUT' && inputSetter) {
                            inputSetter.call(element, value);
                        } else if (tag === 'TEXTAREA' && textareaSetter) {
                            textareaSetter.call(element, value);
                        } else {
                            element.value = value;
                        }
                        // 同步属性值以兼容少量非受控场景
                        element.setAttribute('value', value);

                        // 触发事件以驱动框架更新内部状态
                        element.dispatchEvent(new Event('input', { bubbles: true }));
                        element.dispatchEvent(new Event('change', { bubbles: true }));

                        filledCount++;
                        console.log('[Tauri] Filled input (native setter):', selector, '=', value);
                        return true;
                    } catch (e) {
                        console.warn('[Tauri] Failed native set, fallback:', e);
                        element.value = value;
                        element.dispatchEvent(new Event('input', { bubbles: true }));
                        element.dispatchEvent(new Event('change', { bubbles: true }));
                        filledCount++;
                        return true;
                    }
                }
                return false;
            }

            // 辅助函数：填充下拉框
            function fillSelect(selector, value) {
                const element = document.querySelector(selector);
                if (element) {
                    const options = Array.from(element.options);
                    const matchingOption = options.find(opt =>
                        opt.value === value ||
                        opt.text === value ||
                        opt.value.toUpperCase() === value.toUpperCase()
                    );
                    if (matchingOption) {
                        element.value = matchingOption.value;
                        element.dispatchEvent(new Event('change', { bubbles: true }));
                        filledCount++;
                        console.log('[Tauri] Filled select:', selector, '=', value);
                        return true;
                    }
                }
                return false;
            }

            // 读取卡号输入框的值
            const cardInputElement = document.getElementById('tauri-card-input');
            let cardData = null;
            if (cardInputElement && cardInputElement.value.trim()) {
                const parts = cardInputElement.value.trim().split('|');
                if (parts.length === 4) {
                    cardData = {
                        number: parts[0].trim(),
                        month: parts[1].trim(),
                        year: parts[2].trim(),
                        cvv: parts[3].trim()
                    };
                    console.log('[Tauri] Parsed card data:', cardData);
                } else {
                    console.warn('[Tauri] Card input format invalid, expected 4 parts separated by |, got:', parts.length);
                }
            }

            // 1. 先填充国家（必须先设置为 US，否则其他字段可能不可用）
            fillSelect('select[name="billingCountry"]', 'US') || fillSelect('select[id="billingCountry"]', 'US');

            // 等待一小段时间让表单响应国家变更
            setTimeout(function() {
                // 2. 先填充州（下拉框），避免后续导致输入框被重置
                fillSelect('select[name="billingAdministrativeArea"]', addressData.state) || fillSelect('select[id="billingAdministrativeArea"]', addressData.state);

                // 3. 填充姓名
                fillInput('input[name="billingName"]', addressData.fullName) || fillInput('input[id="billingName"]', addressData.fullName);

                // 4. 填充地址第一行
                fillInput('input[name="billingAddressLine1"]', addressData.street) || fillInput('input[id="billingAddressLine1"]', addressData.street);

                // 5. 填充城市
                fillInput('input[name="billingLocality"]', addressData.city) || fillInput('input[id="billingLocality"]', addressData.city);

                // 6. 填充邮编
                fillInput('input[name="billingPostalCode"]', addressData.zipCode) || fillInput('input[id="billingPostalCode"]', addressData.zipCode);

                // 7. 填充卡号信息（如果有）
                if (cardData) {
                    // 填充卡号
                    fillInput('input[name="cardNumber"]', cardData.number) || fillInput('input[id="cardNumber"]', cardData.number);

                    // 填充有效期（MM/YY 格式）
                    const expiry = cardData.month.padStart(2, '0') + '/' + cardData.year.slice(-2);
                    fillInput('input[name="cardExpiry"]', expiry) || fillInput('input[id="cardExpiry"]', expiry);

                    // 填充 CVV
                    fillInput('input[name="cardCvc"]', cardData.cvv) || fillInput('input[id="cardCvc"]', cardData.cvv);
                }

                if (filledCount > 0) {
                    showCopyNotification('✅ 已填充 ' + filledCount + ' 个字段', '#10b981');
                } else {
                    showCopyNotification('⚠️ 未找到可填充的字段', '#f59e0b');
                }
            }, 300);
            
            // 获取提交按钮
            const submitButton = document.querySelector('button[type="submit"]');
            if (submitButton) {
                submitButton.click();
            }
        }

        // 创建导航栏的函数
        function createNavbar() {
            console.log('[Tauri] Creating navbar...');

            // 检查是否已存在
            if (document.getElementById('tauri-navbar')) {
                console.log('[Tauri] Navbar already exists');
                return;
            }

            const navbar = document.createElement('div');
            navbar.id = 'tauri-navbar';
            navbar.style.cssText = 'position: fixed; top: 50%; right: 20px; transform: translateY(-50%); z-index: 2147483647; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; display: flex; flex-direction: column; gap: 10px;';

            // 检查是否在 augmentcode.com 或 billing.augmentcode.com 域名下
            const isAugmentDomain = window.location.hostname.includes('augmentcode.com');
            const isBillingPage = window.location.hostname.includes('billing.augmentcode.com');

            // 只在 augmentcode.com 域名下显示导入按钮
            if (isAugmentDomain && !isBillingPage) {
                // 创建导入按钮（仅在特定页面显示）
                const button = document.createElement('button');
                button.id = 'tauri-import-button';

                // 检查当前页面状态
                const isLoginPage = window.location.hostname.includes('login.augmentcode.com') ||
                                    window.location.href.includes('/login');
                const isAppPage = window.location.hostname.includes('app.augmentcode.com');
                // 只有带 auto_import=true 参数的 auth 页面才显示"正在导入..."
                const isAuthPage = window.location.hostname.includes('auth.augmentcode.com') &&
                                   window.location.href.includes('auto_import=true');

                // 根据状态设置按钮
                if (isLoginPage) {
                    // 在登录页面,提示登录后会自动导入
                    button.innerHTML = '<div style="text-align: center;">🔒 登录后点击导入<br><span style="font-size: 12px; opacity: 0.8;">Login then Click to Import</span></div>';
                    button.disabled = true;
                    button.style.cssText = 'background: #fef3c7; color: #92400e; border: 1px solid #fbbf24; padding: 12px 20px; border-radius: 8px; cursor: not-allowed; font-size: 14px; font-weight: 500; opacity: 0.9; box-shadow: 0 4px 12px rgba(0,0,0,0.15); line-height: 1.4;';
                    navbar.appendChild(button);
                } else if (isAuthPage) {
                    // Auth页面,显示正在导入
                    button.innerHTML = '<div style="text-align: center;">⏳ 正在导入...<br><span style="font-size: 12px; opacity: 0.8;">Importing...</span></div>';
                    button.disabled = true;
                    button.style.cssText = 'background: #f3f4f6; color: #6b7280; border: 1px solid #d1d5db; padding: 12px 20px; border-radius: 8px; cursor: not-allowed; font-size: 14px; font-weight: 500; opacity: 0.7; box-shadow: 0 4px 12px rgba(0,0,0,0.15); line-height: 1.4;';
                    navbar.appendChild(button);
                } else if (isAppPage) {
                    // App页面,显示可点击按钮
                    button.innerHTML = '<div style="text-align: center;">📥 点击导入<br><span style="font-size: 12px; opacity: 0.9;">Click to Import</span></div>';
                    button.disabled = false;
                    button.style.cssText = 'background: #3b82f6; color: white; border: 1px solid #2563eb; padding: 12px 20px; border-radius: 8px; cursor: pointer; font-size: 14px; font-weight: 500; box-shadow: 0 4px 12px rgba(0,0,0,0.15); line-height: 1.4; transition: all 0.2s;';
                    button.onmouseover = function() {
                        this.style.background = '#2563eb';
                    };
                    button.onmouseout = function() {
                        this.style.background = '#3b82f6';
                    };
                    button.onclick = function() {
                        // 跳转到 auth 页面触发自动导入,添加参数标记这是手动导入
                        window.location.href = 'https://auth.augmentcode.com?auto_import=true';
                    };
                    navbar.appendChild(button);
                }
            }

            // 创建自动填充地址按钮和卡号输入框（仅在 billing.augmentcode.com 显示）
            if (isBillingPage) {
                // 创建容器
                const fillContainer = document.createElement('div');
                fillContainer.id = 'tauri-autofill-container';
                fillContainer.style.cssText = 'background: rgba(255, 255, 255, 0.95); border: 1px solid #d1d5db; border-radius: 8px; padding: 8px; margin-bottom: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.15); display: flex; flex-direction: column;';

                // 创建卡号输入框
                const cardInput = document.createElement('input');
                cardInput.id = 'tauri-card-input';
                cardInput.type = 'text';
                cardInput.placeholder = 'XXXX|XX|XXXX|XXX';
                cardInput.style.cssText = 'width: 220px; padding: 6px 10px; border: 1px solid #d1d5db; border-radius: 6px; font-size: 12px; margin-bottom: 6px; box-sizing: border-box; font-family: monospace;';

                // 创建自动填充按钮
                const fillButton = document.createElement('button');
                fillButton.id = 'tauri-autofill-button';
                fillButton.innerHTML = '<div style="text-align: center;">📝 自动填充地址<br><span style="font-size: 11px; opacity: 0.9;">Auto Fill Address</span></div>';
                fillButton.style.cssText = 'width: 220px; background: #10b981; color: white; border: 1px solid #059669; padding: 8px 12px; border-radius: 6px; cursor: pointer; font-size: 13px; font-weight: 500; box-shadow: 0 2px 6px rgba(0,0,0,0.1); line-height: 1.3; transition: all 0.2s;';
                fillButton.onmouseover = function() {
                    this.style.background = '#059669';
                };
                fillButton.onmouseout = function() {
                    this.style.background = '#10b981';
                };
                fillButton.onclick = function() {
                    autoFillAddress();
                };

                // 组装容器
                fillContainer.appendChild(cardInput);
                fillContainer.appendChild(fillButton);
                navbar.appendChild(fillContainer);
            }

            // 插入到页面
            if (document.body) {
                document.body.appendChild(navbar);
                console.log('[Tauri] Navbar inserted at right middle');
            } else if (document.documentElement) {
                document.documentElement.appendChild(navbar);
                console.log('[Tauri] Navbar inserted to documentElement');
            }
        }

        // 多种方式尝试插入导航栏
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', createNavbar);
        } else {
            createNavbar();
        }

        // 监听页面变化,确保导航栏始终存在
        setInterval(function() {
            if (!document.getElementById('tauri-navbar')) {
                createNavbar();
            }
        }, 1000);
    "#)
    .on_page_load(move |window, payload| {
        if payload.event() == PageLoadEvent::Finished {
            let url_str = payload.url().to_string();

            // 检查是否是 auth.augmentcode.com 且带有 auto_import=true 参数
            // 只有手动点击"点击导入"按钮才会带这个参数,避免注册流程触发自动导入
            if url_str.contains("auth.augmentcode.com") && url_str.contains("auto_import=true") {
                let window_clone = window.clone();
                let app_handle_clone = app_handle.clone();

                // 在后台线程中获取 Cookie (使用 tauri 的 async runtime)
                tauri::async_runtime::spawn(async move {
                    // 等待一小段时间确保 Cookie 已设置
                    tokio::time::sleep(Duration::from_millis(1000)).await;

                    match window_clone.cookies_for_url(
                        "https://auth.augmentcode.com".parse().unwrap()
                    ) {
                        Ok(cookies) => {
                            // 查找 session Cookie
                            if let Some(session_cookie) = cookies.iter()
                                .find(|c| c.name() == "session")
                            {
                                let session_value = session_cookie.value().to_string();
                                eprintln!("Found session cookie, attempting to import token...");

                                // 获取 AppState 并调用带缓存的内部函数
                                let state = app_handle_clone.state::<AppState>();
                                match add_token_from_session_internal_with_cache(&session_value, &state).await {
                                    Ok(token_data) => {
                                        eprintln!("Successfully imported token from session");

                                        // 发送成功事件到前端，包含 session
                                        let _ = app_handle_clone.emit(
                                            "session-auto-imported",
                                            serde_json::json!({
                                                "success": true,
                                                "token": token_data,
                                                "session": session_value
                                            })
                                        );

                                        // 延迟关闭浏览器窗口,让用户看到成功提示
                                        tokio::time::sleep(Duration::from_millis(500)).await;
                                        let _ = window_clone.close();

                                        // 聚焦主窗口
                                        if let Some(main_window) = app_handle_clone.get_webview_window("main") {
                                            let _ = main_window.set_focus();
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to import token: {}", e);
                                        // 发送失败事件
                                        let _ = app_handle_clone.emit(
                                            "session-auto-import-failed",
                                            serde_json::json!({
                                                "success": false,
                                                "error": e.to_string()
                                            })
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to get cookies: {}", e);
                        }
                    }
                });
            }
        }
    })
    .build()
    .map_err(|e| format!("Failed to create browser window: {}", e))?;

    Ok(window_label)
}

#[tauri::command]
pub async fn close_window(app: AppHandle, window_label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&window_label) {
        window
            .close()
            .map_err(|e| format!("Failed to close window: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_data_folder(app: AppHandle) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    // Open folder using system default file manager
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&app_data_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&app_data_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&app_data_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn create_jetbrains_token_file(
    editor_type: String,
    token_data: String,
) -> Result<String, String> {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    // 获取用户主目录
    let home_dir = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map_err(|_| "Failed to get home directory".to_string())?;

    let augment_dir = PathBuf::from(&home_dir).join(".augment");

    // 确保 .augment 目录存在
    fs::create_dir_all(&augment_dir)
        .map_err(|e| format!("Failed to create .augment directory: {}", e))?;

    // 创建文件路径
    let file_name = format!("{}_token.json", editor_type);
    let file_path = augment_dir.join(&file_name);

    // 写入文件
    fs::write(&file_path, token_data).map_err(|e| format!("Failed to write token file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn configure_vim_augment(
    access_token: String,
    tenant_url: String,
) -> Result<String, String> {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    // 获取用户主目录
    let home_dir = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map_err(|_| "Failed to get home directory".to_string())?;

    // 配置文件路径 (所有系统统一使用 .local/share/vim-augment)
    let vim_augment_dir = PathBuf::from(&home_dir)
        .join(".local")
        .join("share")
        .join("vim-augment");

    // 确保目录存在
    fs::create_dir_all(&vim_augment_dir)
        .map_err(|e| format!("Failed to create vim-augment directory: {}", e))?;

    let secrets_path = vim_augment_dir.join("secrets.json");

    // 如果文件已存在，先删除
    if secrets_path.exists() {
        fs::remove_file(&secrets_path)
            .map_err(|e| format!("Failed to remove existing secrets.json: {}", e))?;
    }

    // 构建内层 JSON 对象
    let inner_json = serde_json::json!({
        "accessToken": access_token,
        "tenantURL": tenant_url,
        "scopes": ["email"]
    });

    // 将内层 JSON 转换为字符串（这会自动转义引号）
    let inner_json_str = serde_json::to_string(&inner_json)
        .map_err(|e| format!("Failed to serialize inner JSON: {}", e))?;

    // 构建外层 JSON 对象
    let outer_json = serde_json::json!({
        "augment.sessions": inner_json_str
    });

    // 将外层 JSON 转换为格式化的字符串
    let json_content = serde_json::to_string_pretty(&outer_json)
        .map_err(|e| format!("Failed to serialize outer JSON: {}", e))?;

    // 写入文件
    fs::write(&secrets_path, json_content)
        .map_err(|e| format!("Failed to write secrets.json: {}", e))?;

    // 在 Unix 系统上设置文件权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&secrets_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .permissions();
        perms.set_mode(0o600); // 仅所有者可读写
        fs::set_permissions(&secrets_path, perms)
            .map_err(|e| format!("Failed to set file permissions: {}", e))?;
    }

    Ok(secrets_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn configure_auggie(access_token: String, tenant_url: String) -> Result<String, String> {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    // 获取用户主目录
    let home_dir = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .map_err(|_| "Failed to get home directory".to_string())?;

    // 确定 .augment 目录路径
    let augment_dir = PathBuf::from(&home_dir).join(".augment");

    // 确保目录存在
    fs::create_dir_all(&augment_dir)
        .map_err(|e| format!("Failed to create .augment directory: {}", e))?;

    let session_path = augment_dir.join("session.json");

    // 如果文件已存在，先删除
    if session_path.exists() {
        fs::remove_file(&session_path)
            .map_err(|e| format!("Failed to remove existing session.json: {}", e))?;
    }

    // 构建 JSON 对象
    let session_json = serde_json::json!({
        "accessToken": access_token,
        "tenantURL": tenant_url,
        "scopes": ["read", "write"]
    });

    // 将 JSON 转换为格式化的字符串
    let json_content = serde_json::to_string_pretty(&session_json)
        .map_err(|e| format!("Failed to serialize session JSON: {}", e))?;

    // 写入文件
    fs::write(&session_path, json_content)
        .map_err(|e| format!("Failed to write session.json: {}", e))?;

    // 在 Unix 系统上设置文件权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&session_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .permissions();
        perms.set_mode(0o600); // 仅所有者可读写
        fs::set_permissions(&session_path, perms)
            .map_err(|e| format!("Failed to set file permissions: {}", e))?;
    }

    Ok(session_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn open_editor_with_protocol(app: AppHandle, protocol_url: String) -> Result<(), String> {
    println!("Opening editor with protocol URL: {}", protocol_url);

    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(protocol_url, None::<&str>)
        .map_err(|e| format!("Failed to open editor with protocol: {}", e))
}
