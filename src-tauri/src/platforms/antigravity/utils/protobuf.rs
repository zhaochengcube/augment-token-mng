/// Protobuf 编码工具

/// 读取 Protobuf Varint
fn read_varint(data: &[u8], offset: usize) -> Result<(u64, usize), String> {
    let mut result = 0u64;
    let mut shift = 0;
    let mut pos = offset;

    loop {
        if pos >= data.len() {
            return Err("Invalid varint: unexpected end".to_string());
        }
        let byte = data[pos];
        result |= ((byte & 0x7F) as u64) << shift;
        pos += 1;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }

    Ok((result, pos))
}

/// 跳过指定 wire_type 的字段
fn skip_field(data: &[u8], offset: usize, wire_type: u8) -> Result<usize, String> {
    match wire_type {
        0 => {
            let (_, new_offset) = read_varint(data, offset)?;
            Ok(new_offset)
        }
        1 => Ok(offset + 8),
        2 => {
            let (length, content_offset) = read_varint(data, offset)?;
            Ok(content_offset + length as usize)
        }
        5 => Ok(offset + 4),
        _ => Err(format!("Unsupported wire type: {}", wire_type)),
    }
}

/// 移除 Protobuf 中的指定 Field
pub fn remove_protobuf_field(data: &[u8], field_number: u8) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let mut offset = 0;
    let target_field = field_number as u32;

    while offset < data.len() {
        let start_offset = offset;
        let (tag, new_offset) = read_varint(data, offset)?;
        let wire_type = (tag & 7) as u8;
        let current_field = (tag >> 3) as u32;

        if current_field == target_field {
            offset = skip_field(data, new_offset, wire_type)?;
        } else {
            let next_offset = skip_field(data, new_offset, wire_type)?;
            result.extend_from_slice(&data[start_offset..next_offset]);
            offset = next_offset;
        }
    }

    Ok(result)
}

/// 创建 OAuth Field (Field 6)
pub fn create_oauth_field(access_token: &str, refresh_token: &str, expiry: i64) -> Vec<u8> {
    encode_length_delimited_field(
        6,
        &create_oauth_token_info(access_token, refresh_token, expiry),
    )
}

/// 创建 OAuth TokenInfo 数据
pub fn create_oauth_token_info(access_token: &str, refresh_token: &str, expiry: i64) -> Vec<u8> {
    let mut token_info = Vec::new();
    // Field 1: access_token (string)
    token_info.extend(encode_length_delimited_field(1, access_token.as_bytes()));

    // Field 2: token_type (string)
    let token_type = "Bearer";
    token_info.extend(encode_length_delimited_field(2, token_type.as_bytes()));

    // Field 3: refresh_token (string)
    token_info.extend(encode_length_delimited_field(3, refresh_token.as_bytes()));

    // Field 4: expiry (google.protobuf.Timestamp)
    let expiry_seconds = if expiry < 0 { 0 } else { expiry as u64 };
    let mut expiry_inner = Vec::new();
    expiry_inner.push((1 << 3) | 0);
    expiry_inner.extend(encode_varint(expiry_seconds));
    token_info.extend(encode_length_delimited_field(4, &expiry_inner));

    token_info
}

/// 创建 Antigravity unified state 中的 OAuth 主题数据
pub fn create_unified_oauth_state(access_token: &str, refresh_token: &str, expiry: i64) -> Vec<u8> {
    use base64::Engine as _;

    let auth_state = r#"{"state":"signedIn","context":{"project":"","showProjectError":false,"errorMessage":"","ineligibleMessage":"","verificationUrl":"","isGcpTos":false,"browserOpenFailed":false,"appealUrl":"","appealLinkText":""}}"#;
    let token_info = base64::engine::general_purpose::STANDARD.encode(create_oauth_token_info(
        access_token,
        refresh_token,
        expiry,
    ));

    let mut state = Vec::new();
    state.extend(create_unified_state_entry(
        "authStateWithContextSentinelKey",
        auth_state,
    ));
    state.extend(create_unified_state_entry(
        "oauthTokenInfoSentinelKey",
        &token_info,
    ));
    state
}

fn create_unified_state_entry(key: &str, value: &str) -> Vec<u8> {
    let mut value_wrapper = Vec::new();
    value_wrapper.extend(encode_length_delimited_field(1, value.as_bytes()));

    let mut entry = Vec::new();
    entry.extend(encode_length_delimited_field(1, key.as_bytes()));
    entry.extend(encode_length_delimited_field(2, &value_wrapper));

    encode_length_delimited_field(1, &entry)
}

fn encode_length_delimited_field(field_number: u8, value: &[u8]) -> Vec<u8> {
    let mut field = Vec::new();
    field.push((field_number << 3) | 2);
    field.extend(encode_varint(value.len() as u64));
    field.extend_from_slice(value);
    field
}

/// 编码 Varint
fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut result = Vec::new();

    while value >= 0x80 {
        result.push((value & 0x7F | 0x80) as u8);
        value >>= 7;
    }
    result.push(value as u8);

    result
}

// decode_varint is intentionally omitted; use read_varint for protobuf tags/lengths.
