use crate::http_client::create_proxy_client;
use crate::windsurf::models::{Account, QuotaData};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const DEVIN_AUTH_BASE_URL: &str = "https://windsurf.com/_devin-auth";
const WINDSURF_BACKEND_URL: &str = "https://web-backend.windsurf.com";
const DEVIN_SESSION_EXPIRES_IN: i64 = 32 * 24 * 60 * 60;
const DEFAULT_API_SERVER_URL: &str = "https://server.self-serve.windsurf.com";

#[derive(Debug, Deserialize)]
struct PasswordLoginResponse {
    #[serde(alias = "token", alias = "auth1Token", alias = "auth_token")]
    auth1_token: String,
    #[serde(default, alias = "user_id", alias = "userId", alias = "accountId")]
    account_id: Option<String>,
    #[serde(default)]
    _email: Option<String>,
    #[serde(flatten)]
    _extra: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WindsurfOrg {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DevinLoginResult {
    pub session_token: String,
    pub auth1_token: String,
    pub account_id: Option<String>,
    pub primary_org_id: Option<String>,
    pub orgs: Vec<WindsurfOrg>,
}

#[derive(Debug, Clone, Default)]
pub struct CurrentUserInfo {
    pub api_key: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub user_id: Option<String>,
    pub plan_name: Option<String>,
    pub used_credits: Option<i64>,
    pub total_credits: Option<i64>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum WireType {
    Varint = 0,
    Fixed64 = 1,
    LengthDelimited = 2,
    StartGroup = 3,
    EndGroup = 4,
    Fixed32 = 5,
}

pub(crate) struct ProtobufParser {
    data: Vec<u8>,
    position: usize,
}

impl ProtobufParser {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        Self { data, position: 0 }
    }

    pub(crate) fn parse_message(&mut self) -> Result<Value, String> {
        let mut message = serde_json::Map::new();

        while self.position < self.data.len() {
            let Ok((field_number, wire_type)) = self.read_tag() else {
                break;
            };
            if field_number == 0 {
                break;
            }

            let value = self.read_field(wire_type)?;
            let field_name = self.get_field_name(field_number, wire_type, &value);

            if message.contains_key(&field_name) {
                let existing = message.get_mut(&field_name).unwrap();
                if !existing.is_array() {
                    let temp = existing.clone();
                    *existing = json!([temp]);
                }
                if let Some(arr) = existing.as_array_mut() {
                    arr.push(value);
                }
            } else {
                message.insert(field_name, value);
            }
        }

        Ok(Value::Object(message))
    }

    fn read_tag(&mut self) -> Result<(u32, WireType), String> {
        if self.position >= self.data.len() {
            return Ok((0, WireType::Varint));
        }

        let tag = self.read_varint()?;
        let field_number = tag >> 3;
        let wire_type = match tag & 0x07 {
            0 => WireType::Varint,
            1 => WireType::Fixed64,
            2 => WireType::LengthDelimited,
            3 => WireType::StartGroup,
            4 => WireType::EndGroup,
            5 => WireType::Fixed32,
            _ => return Err(format!("Unknown protobuf wire type: {}", tag & 0x07)),
        };

        Ok((field_number as u32, wire_type))
    }

    fn read_varint(&mut self) -> Result<u64, String> {
        let mut result: u64 = 0;
        let mut shift = 0;

        while self.position < self.data.len() {
            let byte = self.data[self.position];
            self.position += 1;
            result |= ((byte & 0x7F) as u64) << shift;

            if (byte & 0x80) == 0 {
                return Ok(result);
            }

            shift += 7;
            if shift >= 64 {
                return Err("Protobuf varint too long".to_string());
            }
        }

        Err("Unexpected end while reading protobuf varint".to_string())
    }

    fn read_fixed32(&mut self) -> Result<f32, String> {
        if self.position + 4 > self.data.len() {
            return Err("Unexpected end while reading fixed32".to_string());
        }
        let bytes = &self.data[self.position..self.position + 4];
        self.position += 4;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_fixed64(&mut self) -> Result<f64, String> {
        if self.position + 8 > self.data.len() {
            return Err("Unexpected end while reading fixed64".to_string());
        }
        let bytes = &self.data[self.position..self.position + 8];
        self.position += 8;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Ok(f64::from_le_bytes(arr))
    }

    fn read_length_delimited(&mut self) -> Result<Vec<u8>, String> {
        let length = self.read_varint()? as usize;
        if self.position + length > self.data.len() {
            return Err(format!(
                "Unexpected end while reading length-delimited field: need {}, remaining {}",
                length,
                self.data.len().saturating_sub(self.position)
            ));
        }
        let value = self.data[self.position..self.position + length].to_vec();
        self.position += length;
        Ok(value)
    }

    fn read_field(&mut self, wire_type: WireType) -> Result<Value, String> {
        match wire_type {
            WireType::Varint => Ok(json!(self.read_varint()?)),
            WireType::Fixed64 => Ok(json!(self.read_fixed64()?)),
            WireType::Fixed32 => Ok(json!(self.read_fixed32()?)),
            WireType::LengthDelimited => {
                let data = self.read_length_delimited()?;
                if let Ok(text) = String::from_utf8(data.clone()) {
                    if text
                        .chars()
                        .all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                        && !text.is_empty()
                    {
                        return Ok(json!(text));
                    }
                }

                let mut parser = ProtobufParser::new(data.clone());
                if let Ok(sub_message) = parser.parse_message() {
                    if sub_message.as_object().map_or(false, |o| !o.is_empty()) {
                        return Ok(sub_message);
                    }
                }

                Ok(json!({
                    "length": data.len(),
                    "preview": &data[..32.min(data.len())]
                }))
            }
            WireType::StartGroup | WireType::EndGroup => {
                Err("Unsupported protobuf group field".to_string())
            }
        }
    }

    fn get_field_name(&self, field_number: u32, wire_type: WireType, value: &Value) -> String {
        match wire_type {
            WireType::Varint => format!("int_{}", field_number),
            WireType::LengthDelimited => {
                if value.is_string() {
                    format!("string_{}", field_number)
                } else if value.is_object() {
                    format!("subMesssage_{}", field_number)
                } else {
                    format!("bytes_{}", field_number)
                }
            }
            WireType::Fixed32 => format!("float_{}", field_number),
            WireType::Fixed64 => format!("double_{}", field_number),
            _ => format!("field_{}", field_number),
        }
    }
}

fn encode_varint(buf: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        buf.push((value as u8 & 0x7F) | 0x80);
        value >>= 7;
    }
    buf.push(value as u8);
}

pub(crate) fn encode_proto_string_field(buf: &mut Vec<u8>, field_no: u32, value: &str) {
    let tag = (field_no << 3) | 2;
    encode_varint(buf, tag as u64);
    let bytes = value.as_bytes();
    encode_varint(buf, bytes.len() as u64);
    buf.extend_from_slice(bytes);
}

fn decode_varint(bytes: &[u8], offset: usize) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0;
    let mut i = offset;

    while i < bytes.len() {
        let b = bytes[i];
        result |= ((b & 0x7F) as u64) << shift;
        i += 1;
        if b & 0x80 == 0 {
            return Some((result, i - offset));
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }

    None
}

fn apply_common_headers(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    builder
        .header("Content-Type", "application/json")
        .header("Accept", "*/*")
        .header("Accept-Language", "zh-CN,zh;q=0.9")
        .header("Origin", "https://windsurf.com")
        .header("Referer", "https://windsurf.com/account/login")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "same-origin")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36",
        )
}

pub(crate) fn apply_account_auth_headers(
    builder: reqwest::RequestBuilder,
    account: &Account,
) -> reqwest::RequestBuilder {
    let mut req = builder.header("x-auth-token", &account.token.access_token);

    if account.is_devin_account() {
        req = req.header("x-devin-session-token", &account.token.access_token);
        if let Some(account_id) = &account.devin_account_id {
            req = req.header("x-devin-account-id", account_id);
        }
        if let Some(auth1_token) = &account.devin_auth1_token {
            req = req.header("x-devin-auth1-token", auth1_token);
        }
        if let Some(primary_org_id) = &account.devin_primary_org_id {
            req = req.header("x-devin-primary-org-id", primary_org_id);
        }
    }

    req
}

async fn password_login(email: &str, password: &str) -> Result<PasswordLoginResponse, String> {
    let client = create_proxy_client()?;
    let url = format!("{}/password/login", DEVIN_AUTH_BASE_URL);
    let body = json!({
        "email": email,
        "password": password,
    });

    let response = apply_common_headers(client.post(&url))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Devin login request failed: {}", e))?;

    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        let lower = text.to_lowercase();
        if lower.contains("invalid")
            && (lower.contains("password") || lower.contains("credentials"))
        {
            return Err("Devin login failed: invalid email or password".to_string());
        }
        if lower.contains("not found") || lower.contains("no such") {
            return Err("Devin login failed: account not found in Devin/Auth1".to_string());
        }
        if status.as_u16() == 429 || lower.contains("too many") || lower.contains("rate") {
            return Err(
                "Devin login failed: too many attempts, please try again later".to_string(),
            );
        }
        return Err(format!("Devin login failed ({}): {}", status, text));
    }

    serde_json::from_str::<PasswordLoginResponse>(&text).map_err(|e| {
        format!(
            "Failed to parse Devin login response: {}. Body: {}",
            e, text
        )
    })
}

pub async fn login_with_email_password(
    email: &str,
    password: &str,
) -> Result<DevinLoginResult, String> {
    let login = password_login(email, password).await?;
    let mut result =
        post_auth_to_login_result(&login.auth1_token, login.account_id.clone(), None).await?;

    if result.account_id.is_none() {
        result.account_id = login.account_id;
    }
    if result.session_token.is_empty() {
        return Err(
            "Devin login failed: WindsurfPostAuth did not return a session token".to_string(),
        );
    }

    Ok(result)
}

pub async fn login_with_auth1_token(
    auth1_token: &str,
    org_id: Option<&str>,
) -> Result<DevinLoginResult, String> {
    post_auth_to_login_result(auth1_token, None, org_id).await
}

pub async fn windsurf_post_auth(
    auth1_token: &str,
    org_id: &str,
) -> Result<DevinLoginResult, String> {
    let post_auth = post_auth_raw(auth1_token, org_id).await?;
    let session_token = post_auth.session_token;
    let auth1_token = post_auth
        .auth1_token
        .unwrap_or_else(|| auth1_token.to_string());

    Ok(DevinLoginResult {
        session_token,
        auth1_token,
        account_id: post_auth.account_id,
        primary_org_id: post_auth.primary_org_id,
        orgs: post_auth.orgs,
    })
}

async fn post_auth_to_login_result(
    auth1_token: &str,
    fallback_account_id: Option<String>,
    org_id: Option<&str>,
) -> Result<DevinLoginResult, String> {
    let mut result = windsurf_post_auth(auth1_token, org_id.unwrap_or("")).await?;

    if result.session_token.is_empty() && !result.orgs.is_empty() {
        let picked_org = result
            .primary_org_id
            .clone()
            .or_else(|| result.orgs.first().map(|org| org.id.clone()))
            .ok_or_else(|| {
                "Devin login requires organization selection, but no organization was returned"
                    .to_string()
            })?;
        result = windsurf_post_auth(auth1_token, &picked_org).await?;
        if result.primary_org_id.is_none() {
            result.primary_org_id = Some(picked_org);
        }
    }

    if result.account_id.is_none() {
        result.account_id = fallback_account_id;
    }

    Ok(result)
}

#[derive(Debug, Default)]
struct RawPostAuthResult {
    session_token: String,
    auth1_token: Option<String>,
    account_id: Option<String>,
    primary_org_id: Option<String>,
    orgs: Vec<WindsurfOrg>,
}

async fn post_auth_raw(auth1_token: &str, org_id: &str) -> Result<RawPostAuthResult, String> {
    let client = create_proxy_client()?;
    let url = format!(
        "{}/exa.seat_management_pb.SeatManagementService/WindsurfPostAuth",
        WINDSURF_BACKEND_URL
    );

    let mut body = Vec::with_capacity(auth1_token.len() + org_id.len() + 4);
    encode_proto_string_field(&mut body, 1, auth1_token);
    if !org_id.is_empty() {
        encode_proto_string_field(&mut body, 2, org_id);
    }

    let response = client
        .post(&url)
        .body(body)
        .header("accept", "*/*")
        .header("accept-language", "zh-CN,zh;q=0.9")
        .header("connect-protocol-version", "1")
        .header("content-type", "application/proto")
        .header("origin", "https://windsurf.com")
        .header("referer", "https://windsurf.com/account/login")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-site")
        .header("X-Devin-Auth1-Token", auth1_token)
        .send()
        .await
        .map_err(|e| format!("WindsurfPostAuth request failed: {}", e))?;

    let status = response.status();
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!(
            "WindsurfPostAuth failed ({}): {}",
            status,
            String::from_utf8_lossy(&bytes)
        ));
    }

    parse_windsurf_post_auth_response(&bytes)
}

fn parse_windsurf_post_auth_response(bytes: &[u8]) -> Result<RawPostAuthResult, String> {
    let mut result = RawPostAuthResult::default();
    let mut i = 0;

    while i < bytes.len() {
        let (tag, consumed) = decode_varint(bytes, i)
            .ok_or_else(|| "Failed to decode WindsurfPostAuth tag".to_string())?;
        i += consumed;
        let field_no = (tag >> 3) as u32;
        let wire_type = (tag & 0x7) as u8;

        if wire_type == 2 {
            let (len, consumed_len) = decode_varint(bytes, i)
                .ok_or_else(|| "Failed to decode WindsurfPostAuth length".to_string())?;
            i += consumed_len;
            let end = i + len as usize;
            if end > bytes.len() {
                return Err("WindsurfPostAuth response length overflow".to_string());
            }

            let payload = &bytes[i..end];
            match field_no {
                1 => result.session_token = String::from_utf8_lossy(payload).into_owned(),
                2 => {
                    if let Some(org) = parse_windsurf_org(payload) {
                        result.orgs.push(org);
                    }
                }
                3 => result.auth1_token = Some(String::from_utf8_lossy(payload).into_owned()),
                4 => result.account_id = Some(String::from_utf8_lossy(payload).into_owned()),
                5 => result.primary_org_id = Some(String::from_utf8_lossy(payload).into_owned()),
                _ => {}
            }
            i = end;
        } else {
            match wire_type {
                0 => {
                    let (_, c) = decode_varint(bytes, i)
                        .ok_or_else(|| "Failed to skip protobuf varint".to_string())?;
                    i += c;
                }
                1 => i += 8,
                5 => i += 4,
                _ => {
                    return Err(format!(
                        "Unsupported WindsurfPostAuth wire type: {}",
                        wire_type
                    ));
                }
            }
        }
    }

    if result.session_token.is_empty() && result.orgs.is_empty() && result.auth1_token.is_none() {
        return Err("WindsurfPostAuth response did not contain a session token".to_string());
    }

    Ok(result)
}

fn parse_windsurf_org(bytes: &[u8]) -> Option<WindsurfOrg> {
    let mut org = WindsurfOrg::default();
    let mut i = 0;

    while i < bytes.len() {
        let (tag, consumed) = decode_varint(bytes, i)?;
        i += consumed;
        let field_no = (tag >> 3) as u32;
        let wire_type = (tag & 0x7) as u8;

        match wire_type {
            2 => {
                let (len, consumed_len) = decode_varint(bytes, i)?;
                i += consumed_len;
                let end = i + len as usize;
                if end > bytes.len() {
                    break;
                }
                let payload = &bytes[i..end];
                match field_no {
                    1 => org.id = String::from_utf8_lossy(payload).into_owned(),
                    2 => org.name = String::from_utf8_lossy(payload).into_owned(),
                    _ => {}
                }
                i = end;
            }
            0 => {
                let (_, c) = decode_varint(bytes, i)?;
                i += c;
            }
            1 => i += 8,
            5 => i += 4,
            _ => break,
        }
    }

    if org.id.is_empty() && org.name.is_empty() {
        None
    } else {
        Some(org)
    }
}

pub async fn refresh_session(account: &mut Account) -> Result<(), String> {
    let auth1_token = account.devin_auth1_token.clone().ok_or_else(|| {
        "This Devin account does not have auth1_token and cannot refresh session".to_string()
    })?;
    let org_id = account.devin_primary_org_id.clone().unwrap_or_default();
    let result = post_auth_to_login_result(
        &auth1_token,
        account.devin_account_id.clone(),
        Some(&org_id),
    )
    .await?;

    if result.session_token.is_empty() {
        return Err("Devin session refresh did not return a session token".to_string());
    }

    account.token.access_token = result.session_token;
    account.token.expiry_timestamp = chrono::Utc::now().timestamp() + DEVIN_SESSION_EXPIRES_IN;
    account.auth_provider = Some("devin".to_string());
    account.devin_auth1_token = Some(result.auth1_token);
    if result.account_id.is_some() {
        account.devin_account_id = result.account_id;
    }
    if result.primary_org_id.is_some() {
        account.devin_primary_org_id = result.primary_org_id;
    }
    account.updated_at = chrono::Utc::now().timestamp();

    Ok(())
}

pub fn devin_session_expires_in() -> i64 {
    DEVIN_SESSION_EXPIRES_IN
}

pub async fn get_current_user(account: &Account) -> Result<CurrentUserInfo, String> {
    let client = create_proxy_client()?;
    let token = &account.token.access_token;
    let url = format!(
        "{}/exa.seat_management_pb.SeatManagementService/GetCurrentUser",
        WINDSURF_BACKEND_URL
    );

    let mut body = Vec::new();
    encode_proto_string_field(&mut body, 1, token);
    body.extend_from_slice(&[0x10, 0x01, 0x18, 0x01, 0x20, 0x01]);

    let response = apply_account_auth_headers(client.post(&url), account)
        .body(body)
        .header("accept", "*/*")
        .header("accept-language", "zh-CN,zh;q=0.9")
        .header("cache-control", "no-cache")
        .header("connect-protocol-version", "1")
        .header("content-type", "application/proto")
        .header("pragma", "no-cache")
        .header("sec-fetch-dest", "empty")
        .header("sec-fetch-mode", "cors")
        .header("sec-fetch-site", "same-site")
        .header("Referer", "https://windsurf.com/")
        .send()
        .await
        .map_err(|e| format!("GetCurrentUser request failed: {}", e))?;

    let status = response.status();
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!(
            "GetCurrentUser failed ({}): {}",
            status,
            String::from_utf8_lossy(&bytes)
        ));
    }

    parse_current_user_response(&bytes)
}

pub async fn enrich_account(account: &mut Account) -> Result<(), String> {
    let info = get_current_user(account).await?;

    if let Some(email) = info.email.filter(|s| !s.is_empty()) {
        account.email = email;
    }
    if let Some(name) = info.name.filter(|s| !s.is_empty()) {
        account.name = Some(name);
    }
    if let Some(api_key) = info.api_key.filter(|s| !s.is_empty()) {
        account.api_key = Some(api_key);
    }
    if let Some(user_id) = info.user_id.filter(|s| !s.is_empty()) {
        account.token.user_id = Some(user_id);
    }
    if account.api_server_url.is_none() {
        account.api_server_url = Some(DEFAULT_API_SERVER_URL.to_string());
    }

    let used = info.used_credits.unwrap_or(0);
    let total = info.total_credits.unwrap_or(0);
    if info.plan_name.is_some() || total > 0 || used > 0 {
        let usage_percentage = if total > 0 { (used * 100) / total } else { 0 };
        account.quota = Some(QuotaData {
            plan_name: info.plan_name.unwrap_or_else(|| "Unknown".to_string()),
            used_credits: used,
            total_credits: total,
            usage_percentage,
            expires_at: info.expires_at,
            plan_start: None,
            billing_strategy: None,
            daily_quota_remaining_percent: None,
            weekly_quota_remaining_percent: None,
            daily_quota_reset_at_unix: None,
            weekly_quota_reset_at_unix: None,
            overage_balance_micros: None,
            last_updated: chrono::Utc::now().timestamp(),
        });
    }

    account.updated_at = chrono::Utc::now().timestamp();
    Ok(())
}

fn parse_current_user_response(response_body: &[u8]) -> Result<CurrentUserInfo, String> {
    let response_str = String::from_utf8_lossy(response_body);
    let decoded = if response_str.starts_with("data:application/proto;base64,") {
        general_purpose::STANDARD
            .decode(&response_str[31..])
            .map_err(|e| format!("Failed to decode base64 protobuf response: {}", e))?
    } else if response_str.starts_with("AAEAAQ==")
        || response_str
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    {
        general_purpose::STANDARD
            .decode(response_str.trim())
            .map_err(|e| format!("Failed to decode base64 protobuf response: {}", e))?
    } else {
        response_body.to_vec()
    };

    let mut parser = ProtobufParser::new(decoded);
    let parsed = parser.parse_message()?;
    extract_current_user_info(&parsed)
}

fn extract_current_user_info(parsed: &Value) -> Result<CurrentUserInfo, String> {
    let obj = parsed
        .as_object()
        .ok_or_else(|| "GetCurrentUser protobuf root is not an object".to_string())?;
    let user = obj
        .get("subMesssage_1")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "GetCurrentUser response does not contain user info".to_string())?;

    let api_key = user
        .get("string_1")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let name = user
        .get("string_2")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let email = user
        .get("string_3")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let user_id = user
        .get("string_6")
        .and_then(|v| v.as_str())
        .map(ToString::to_string);

    let plan = obj.get("subMesssage_6").and_then(|v| v.as_object());
    let team = obj.get("subMesssage_4").and_then(|v| v.as_object());

    let plan_name = plan
        .and_then(|p| p.get("string_2"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string);
    let base_quota = plan
        .and_then(|p| p.get("int_12"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let extra_quota = team
        .and_then(|t| t.get("int_15"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let used_credits = team
        .and_then(|t| t.get("int_17"))
        .and_then(|v| v.as_i64())
        .or_else(|| user.get("int_28").and_then(|v| v.as_i64()));
    let total_credits = match base_quota + extra_quota {
        total if total > 0 => Some(total),
        _ => None,
    };
    let expires_at = team
        .and_then(|t| t.get("subMesssage_18"))
        .and_then(|v| v.get("int_1"))
        .and_then(|v| v.as_i64())
        .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0))
        .map(|dt| dt.to_rfc3339());

    Ok(CurrentUserInfo {
        api_key,
        name,
        email,
        user_id,
        plan_name,
        used_credits,
        total_credits,
        expires_at,
    })
}
