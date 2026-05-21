//! Windsurf seamless account switch helpers.

use crate::http_client::create_proxy_client;
use crate::windsurf::models::Account;
use crate::windsurf::modules::{auth, devin};
use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;

const GET_ONE_TIME_AUTH_TOKEN_API: &str = "https://web-backend.windsurf.com/exa.seat_management_pb.SeatManagementService/GetOneTimeAuthToken";

/// 获取 Windsurf 桌面端 deep link 登录所需的一次性 auth_token。
pub async fn get_one_time_auth_token(account: &Account) -> Result<String, String> {
    let client = create_proxy_client()?;
    let auth_token_input = if account.is_devin_account() {
        account.token.access_token.clone()
    } else {
        if account.token.refresh_token.trim().is_empty() {
            return Err("Firebase account is missing refresh_token".to_string());
        }
        let refreshed = auth::refresh_access_token(&account.token.refresh_token).await?;
        refreshed.google_access_token.ok_or_else(|| {
            "Firebase refresh response did not contain Google access_token".to_string()
        })?
    };

    let mut body = Vec::new();
    devin::encode_proto_string_field(&mut body, 1, &auth_token_input);

    let request = client
        .post(GET_ONE_TIME_AUTH_TOKEN_API)
        .body(body)
        .header("accept", "application/proto")
        .header("accept-language", "zh-CN,zh;q=0.9")
        .header("cache-control", "no-cache")
        .header("connect-protocol-version", "1")
        .header("content-type", "application/proto")
        .header("origin", "https://windsurf.com")
        .header("pragma", "no-cache")
        .header("referer", "https://windsurf.com/")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-site")
        .header(
            "User-Agent",
            "Windsurf/1.4.2",
        );

    let request = if account.is_devin_account() {
        devin::apply_account_auth_headers(request, account)
    } else {
        request
    };

    let response = request
        .send()
        .await
        .map_err(|e| format!("GetOneTimeAuthToken request failed: {}", e))?;

    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read GetOneTimeAuthToken response: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "GetOneTimeAuthToken failed ({}): {}",
            status,
            String::from_utf8_lossy(&bytes)
        ));
    }

    parse_one_time_auth_token(&bytes)
}

/// 触发 Windsurf / Windsurf - Next 的 deep link 登录回调。
pub fn trigger_windsurf_callback(auth_token: &str, client_type: &str) -> Result<(), String> {
    let scheme = match client_type {
        "windsurf-next" => "windsurf-next",
        _ => "windsurf",
    };
    let state = Uuid::new_v4().to_string();
    let callback_url = format!(
        "{}://codeium.windsurf#access_token={}&state={}&token_type=Bearer",
        scheme,
        urlencoding::encode(auth_token),
        urlencoding::encode(&state),
    );

    open::that(&callback_url).map_err(|e| format!("Failed to open Windsurf callback URL: {}", e))
}

fn parse_one_time_auth_token(bytes: &[u8]) -> Result<String, String> {
    let decoded = decode_proto_response(bytes)?;
    let mut pos = 0;

    while pos < decoded.len() {
        let (tag, consumed) = decode_varint(&decoded, pos)
            .ok_or_else(|| "Failed to decode GetOneTimeAuthToken tag".to_string())?;
        pos += consumed;
        let field_no = (tag >> 3) as u32;
        let wire_type = (tag & 0x7) as u8;

        match wire_type {
            2 => {
                let (len, consumed_len) = decode_varint(&decoded, pos)
                    .ok_or_else(|| "Failed to decode GetOneTimeAuthToken length".to_string())?;
                pos += consumed_len;
                let end = pos + len as usize;
                if end > decoded.len() {
                    return Err("GetOneTimeAuthToken response length overflow".to_string());
                }
                if field_no == 1 {
                    let token = String::from_utf8_lossy(&decoded[pos..end]).to_string();
                    if !token.is_empty() {
                        return Ok(token);
                    }
                }
                pos = end;
            }
            0 => {
                let (_, consumed_value) = decode_varint(&decoded, pos)
                    .ok_or_else(|| "Failed to skip GetOneTimeAuthToken varint".to_string())?;
                pos += consumed_value;
            }
            1 => pos += 8,
            5 => pos += 4,
            _ => {
                return Err(format!(
                    "Unsupported GetOneTimeAuthToken wire type: {}",
                    wire_type
                ));
            }
        }
    }

    Err("GetOneTimeAuthToken response did not contain auth_token".to_string())
}

fn decode_proto_response(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.starts_with(b"data:application/proto;base64,") {
        return general_purpose::STANDARD
            .decode(&bytes["data:application/proto;base64,".len()..])
            .map_err(|e| format!("Base64 proto decode failed: {}", e));
    }
    Ok(bytes.to_vec())
}

fn decode_varint(bytes: &[u8], offset: usize) -> Option<(u64, usize)> {
    let mut result = 0u64;
    let mut shift = 0;
    let mut pos = offset;

    while pos < bytes.len() {
        let byte = bytes[pos];
        pos += 1;
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some((result, pos - offset));
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }

    None
}
