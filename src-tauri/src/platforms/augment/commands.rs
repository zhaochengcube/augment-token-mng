use std::time::SystemTime;
use tauri::{AppHandle, Emitter, State};

use crate::{AppSessionCache, AppState};
use super::modules::{account, oauth};
use super::models::{
    AccountStatus,
    AugmentTokenResponse,
    BatchCreditConsumptionResponse,
    PaymentMethodLinkResult,
    SessionRefreshResult,
    TokenFromSessionResponse,
    TokenInfo,
    TokenStatusResult,
};

#[tauri::command]
pub async fn generate_auth_url(state: State<'_, AppState>) -> Result<String, String> {
    let augment_oauth_state = oauth::create_augment_oauth_state();
    let auth_url = oauth::generate_augment_authorize_url(&augment_oauth_state)
        .map_err(|e| format!("Failed to generate auth URL: {}", e))?;

    *state.augment_oauth_state.lock().unwrap() = Some(augment_oauth_state);

    Ok(auth_url)
}

#[tauri::command]
pub async fn generate_augment_auth_url(state: State<'_, AppState>) -> Result<String, String> {
    let augment_oauth_state = oauth::create_augment_oauth_state();
    let auth_url = oauth::generate_augment_authorize_url(&augment_oauth_state)
        .map_err(|e| format!("Failed to generate Augment auth URL: {}", e))?;

    *state.augment_oauth_state.lock().unwrap() = Some(augment_oauth_state);

    Ok(auth_url)
}

#[tauri::command]
pub async fn get_token(code: String, state: State<'_, AppState>) -> Result<AugmentTokenResponse, String> {
    let augment_oauth_state = {
        let guard = state.augment_oauth_state.lock().unwrap();
        guard.clone()
            .ok_or("No Augment OAuth state found. Please generate auth URL first.")?
    };

    oauth::complete_augment_oauth_flow(&augment_oauth_state, &code)
        .await
        .map_err(|e| format!("Failed to complete OAuth flow: {}", e))
}

#[tauri::command]
pub async fn get_augment_token(code: String, state: State<'_, AppState>) -> Result<AugmentTokenResponse, String> {
    let augment_oauth_state = {
        let guard = state.augment_oauth_state.lock().unwrap();
        guard.clone()
            .ok_or("No Augment OAuth state found. Please generate auth URL first.")?
    };

    oauth::complete_augment_oauth_flow(&augment_oauth_state, &code)
        .await
        .map_err(|e| format!("Failed to complete Augment OAuth flow: {}", e))
}

#[tauri::command]
pub async fn check_account_status(token: String, tenant_url: String) -> Result<AccountStatus, String> {
    account::check_account_ban_status(&token, &tenant_url)
        .await
        .map_err(|e| format!("Failed to check account status: {}", e))
}

#[tauri::command]
pub async fn batch_check_tokens_status(
    tokens: Vec<TokenInfo>,
    state: State<'_, AppState>,
) -> Result<Vec<TokenStatusResult>, String> {
    account::batch_check_account_status(tokens, state.app_session_cache.clone())
        .await
        .map_err(|e| format!("Failed to batch check tokens status: {}", e))
}

/// 批量获取 Credit 消费数据(stats 和 chart),使用缓存的 app_session
#[tauri::command]
pub async fn fetch_batch_credit_consumption(
    auth_session: String,
    state: State<'_, AppState>,
) -> Result<BatchCreditConsumptionResponse, String> {
    println!("fetch_batch_credit_consumption called");
    let cached_app_session = {
        let cache = state.app_session_cache.lock().unwrap();
        cache.get(&auth_session).map(|c| c.app_session.clone())
    };

    if let Some(app_session) = cached_app_session {
        println!("Using cached app_session for credit consumption");

        match account::get_batch_credit_consumption_with_app_session(&app_session).await {
            Ok(result) => {
                println!("Successfully fetched credit data with cached app_session");
                return Ok(result);
            }
            Err(e) => {
                println!("Cached app_session failed: {}, will refresh", e);
            }
        }
    }

    println!("Exchanging auth_session for new app_session...");
    let app_session = account::exchange_auth_session_for_app_session(&auth_session).await?;
    println!("New app session obtained: {}", &app_session[..20.min(app_session.len())]);

    {
        let mut cache = state.app_session_cache.lock().unwrap();
        cache.insert(
            auth_session.clone(),
            AppSessionCache {
                app_session: app_session.clone(),
                created_at: SystemTime::now(),
            },
        );
        println!("App session cached for future use");
    }

    account::get_batch_credit_consumption_with_app_session(&app_session).await
}

#[tauri::command]
pub async fn add_token_from_session(
    session: String,
    app: AppHandle,
    _state: State<'_, AppState>,
) -> Result<TokenFromSessionResponse, String> {
    let _ = app.emit("session-import-progress", "sessionImportExtractingToken");
    let token_response = oauth::extract_token_from_session(&session).await?;

    let _ = app.emit("session-import-progress", "sessionImportComplete");

    Ok(TokenFromSessionResponse {
        access_token: token_response.access_token,
        tenant_url: token_response.tenant_url,
        email: token_response.email,
    })
}

#[tauri::command]
pub async fn fetch_payment_method_link_command(
    auth_session: String,
) -> Result<PaymentMethodLinkResult, String> {
    let full_cookies = account::exchange_auth_session_for_full_cookies(&auth_session).await?;
    let payment_link = account::fetch_payment_method_link(&full_cookies).await?;

    Ok(PaymentMethodLinkResult {
        payment_method_link: payment_link,
    })
}


/// Session 刷新请求
#[derive(serde::Deserialize)]
pub struct SessionRefreshRequest {
    pub id: String,
    pub session: String,
}

/// 批量刷新 auth_session
/// 前端传入 { id, session } 列表，后端只负责刷新并返回新的 session
#[tauri::command]
pub async fn batch_refresh_sessions(
    requests: Vec<SessionRefreshRequest>,
) -> Result<Vec<SessionRefreshResult>, String> {
    let mut results = Vec::new();

    for request in requests {
        match oauth::refresh_auth_session(&request.session).await {
            Ok(new_session) => {
                results.push(SessionRefreshResult {
                    token_id: request.id,
                    success: true,
                    error: None,
                    new_session: Some(new_session),
                });
            }
            Err(e) => {
                results.push(SessionRefreshResult {
                    token_id: request.id,
                    success: false,
                    error: Some(e.to_string()),
                    new_session: None,
                });
            }
        }
    }

    Ok(results)
}

// ============ 团队管理相关 Commands ============

/// 获取团队信息
#[tauri::command]
pub async fn fetch_team_info(
    auth_session: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let app_session = get_or_refresh_app_session(&auth_session, &state).await?;
    account::fetch_team_info(&app_session).await
}

/// 修改团队席位数
#[tauri::command]
pub async fn update_team_seats(
    auth_session: String,
    seats: u32,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let app_session = get_or_refresh_app_session(&auth_session, &state).await?;
    account::update_team_seats(&app_session, seats).await
}

/// 邀请团队成员
#[tauri::command]
pub async fn invite_team_members(
    auth_session: String,
    emails: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let app_session = get_or_refresh_app_session(&auth_session, &state).await?;
    account::invite_team_members(&app_session, emails).await
}

/// 删除团队成员
#[tauri::command]
pub async fn delete_team_member(
    auth_session: String,
    user_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let app_session = get_or_refresh_app_session(&auth_session, &state).await?;
    account::delete_team_member(&app_session, &user_id).await
}

/// 删除团队邀请
#[tauri::command]
pub async fn delete_team_invitation(
    auth_session: String,
    invitation_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let app_session = get_or_refresh_app_session(&auth_session, &state).await?;
    account::delete_team_invitation(&app_session, &invitation_id).await
}

// ============ 辅助函数 ============

/// 获取或刷新 app_session
async fn get_or_refresh_app_session(
    auth_session: &str,
    state: &State<'_, AppState>,
) -> Result<String, String> {
    // 先检查缓存
    let cached = {
        let cache = state.app_session_cache.lock().unwrap();
        cache.get(auth_session).map(|c| c.app_session.clone())
    };

    if let Some(app_session) = cached {
        return Ok(app_session);
    }

    // 缓存未命中，交换新的 app_session
    let app_session = account::exchange_auth_session_for_app_session(auth_session).await?;

    // 存入缓存
    {
        let mut cache = state.app_session_cache.lock().unwrap();
        cache.insert(auth_session.to_string(), AppSessionCache {
            app_session: app_session.clone(),
            created_at: SystemTime::now(),
        });
    }

    Ok(app_session)
}
