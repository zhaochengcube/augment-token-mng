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
    let mut field = Vec::new();
    
    // Field 6, wire type 2 (length-delimited)
    field.push((6 << 3) | 2);
    
    // 构建内部数据
    let mut inner = Vec::new();
    
    // Field 1: access_token (string)
    inner.push((1 << 3) | 2);
    inner.extend(encode_varint(access_token.len() as u64));
    inner.extend_from_slice(access_token.as_bytes());
    
    // Field 2: token_type (string)
    let token_type = "Bearer";
    inner.push((2 << 3) | 2);
    inner.extend(encode_varint(token_type.len() as u64));
    inner.extend_from_slice(token_type.as_bytes());
    
    // Field 3: refresh_token (string)
    inner.push((3 << 3) | 2);
    inner.extend(encode_varint(refresh_token.len() as u64));
    inner.extend_from_slice(refresh_token.as_bytes());

    // Field 4: expiry (google.protobuf.Timestamp)
    let expiry_seconds = if expiry < 0 { 0 } else { expiry as u64 };
    let mut expiry_inner = Vec::new();
    expiry_inner.push((1 << 3) | 0);
    expiry_inner.extend(encode_varint(expiry_seconds));
    inner.push((4 << 3) | 2);
    inner.extend(encode_varint(expiry_inner.len() as u64));
    inner.extend(expiry_inner);
    
    // 写入长度
    field.extend(encode_varint(inner.len() as u64));
    field.extend(inner);
    
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
