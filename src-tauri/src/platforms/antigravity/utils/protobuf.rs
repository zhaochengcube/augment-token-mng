/// Protobuf 编码工具

pub fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    while value >= 0x80 {
        result.push((value & 0x7F | 0x80) as u8);
        value >>= 7;
    }
    result.push(value as u8);
    result
}

/// 读取 Protobuf Varint
pub fn read_varint(data: &[u8], offset: usize) -> Result<(u64, usize), String> {
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
pub fn skip_field(data: &[u8], offset: usize, wire_type: u8) -> Result<usize, String> {
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

/// 查找指定 Length-Delimited 字段内容。
pub fn find_field(data: &[u8], target_field: u32) -> Result<Option<Vec<u8>>, String> {
    let mut offset = 0;

    while offset < data.len() {
        let (tag, new_offset) = read_varint(data, offset)?;
        let wire_type = (tag & 7) as u8;
        let field_number = (tag >> 3) as u32;

        if field_number == target_field && wire_type == 2 {
            let (length, content_offset) = read_varint(data, new_offset)?;
            return Ok(Some(
                data[content_offset..content_offset + length as usize].to_vec(),
            ));
        }

        offset = skip_field(data, new_offset, wire_type)?;
    }

    Ok(None)
}

/// 编码长度分隔字段 (wire_type = 2)。
pub fn encode_len_delim_field(field_number: u32, value: &[u8]) -> Vec<u8> {
    let mut field = encode_varint(((field_number << 3) | 2) as u64);
    field.extend(encode_varint(value.len() as u64));
    field.extend_from_slice(value);
    field
}

pub fn encode_string_field(field_number: u32, value: &str) -> Vec<u8> {
    encode_len_delim_field(field_number, value.as_bytes())
}

pub fn encode_varint_field(field_number: u32, value: u64) -> Vec<u8> {
    let mut field = encode_varint(((field_number << 3) | 0) as u64);
    field.extend(encode_varint(value));
    field
}

/// 创建旧格式 Field 6 OAuthTokenInfo。
pub fn create_oauth_field(access_token: &str, refresh_token: &str, expiry: i64) -> Vec<u8> {
    encode_len_delim_field(
        6,
        &create_oauth_token_info(access_token, refresh_token, expiry, false, None, None),
    )
}

pub fn create_email_field(email: &str) -> Vec<u8> {
    encode_string_field(2, email)
}

/// 创建 OAuthTokenInfo 数据。
pub fn create_oauth_token_info(
    access_token: &str,
    refresh_token: &str,
    expiry: i64,
    is_gcp_tos: bool,
    id_token: Option<&str>,
    email: Option<&str>,
) -> Vec<u8> {
    create_oauth_info(access_token, refresh_token, expiry, is_gcp_tos, id_token, email)
}

/// 创建新格式 OAuthTokenInfo 消息，不包含外层 Field 6 包装。
pub fn create_oauth_info(
    access_token: &str,
    refresh_token: &str,
    expiry: i64,
    mut is_gcp_tos: bool,
    id_token: Option<&str>,
    email: Option<&str>,
) -> Vec<u8> {
    if let Some(email) = email {
        let lower = email.to_lowercase();
        let is_personal = lower.ends_with("@gmail.com")
            || lower.ends_with("@outlook.com")
            || lower.ends_with("@hotmail.com")
            || lower.ends_with("@qq.com")
            || lower.ends_with("@163.com");
        if is_personal {
            is_gcp_tos = false;
        }
    }

    let expiry_seconds = if expiry < 0 { 0 } else { expiry as u64 };
    let mut timestamp = encode_varint_field(1, expiry_seconds);
    timestamp.extend(encode_varint_field(2, 0));

    let mut oauth_info = Vec::new();
    oauth_info.extend(encode_string_field(1, access_token));
    oauth_info.extend(encode_string_field(2, "Bearer"));
    oauth_info.extend(encode_string_field(3, refresh_token));
    oauth_info.extend(encode_len_delim_field(4, &timestamp));
    if let Some(id_token) = id_token {
        oauth_info.extend(encode_string_field(5, id_token));
    }
    if is_gcp_tos {
        oauth_info.extend(encode_varint_field(6, 1));
    }

    oauth_info
}

/// 创建统一状态同步条目：Topic -> DataEntry -> Row(base64 payload)。
pub fn create_unified_state_entry(sentinel_key: &str, payload: &[u8]) -> String {
    use base64::{engine::general_purpose, Engine as _};

    let row = encode_string_field(1, &general_purpose::STANDARD.encode(payload));
    let data_entry = [
        encode_string_field(1, sentinel_key),
        encode_len_delim_field(2, &row),
    ]
    .concat();
    let topic = encode_len_delim_field(1, &data_entry);

    general_purpose::STANDARD.encode(topic)
}

pub fn create_string_value_payload(value: &str) -> Vec<u8> {
    encode_string_field(3, value)
}

pub fn create_minimal_user_status_payload(email: &str) -> Vec<u8> {
    [encode_string_field(3, email), encode_string_field(7, email)].concat()
}
