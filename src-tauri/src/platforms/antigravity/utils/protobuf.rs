/// Protobuf 编码工具

/// 移除 Protobuf 中的指定 Field
pub fn remove_protobuf_field(data: &[u8], field_number: u8) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < data.len() {
        let tag = data[i];
        let field_num = tag >> 3;
        let wire_type = tag & 0x07;
        
        if field_num == field_number {
            // 跳过这个 field
            i += 1;
            match wire_type {
                0 => {
                    // Varint
                    while i < data.len() && (data[i] & 0x80) != 0 {
                        i += 1;
                    }
                    i += 1;
                }
                2 => {
                    // Length-delimited
                    let (length, bytes_read) = decode_varint(&data[i..])?;
                    i += bytes_read + length;
                }
                _ => {
                    return Err(format!("Unsupported wire type: {}", wire_type));
                }
            }
        } else {
            // 保留这个 field
            result.push(tag);
            i += 1;
            
            match wire_type {
                0 => {
                    // Varint
                    while i < data.len() && (data[i] & 0x80) != 0 {
                        result.push(data[i]);
                        i += 1;
                    }
                    if i < data.len() {
                        result.push(data[i]);
                        i += 1;
                    }
                }
                2 => {
                    // Length-delimited
                    let start = i;
                    let (length, bytes_read) = decode_varint(&data[i..])?;
                    let end = i + bytes_read + length;
                    result.extend_from_slice(&data[start..end]);
                    i = end;
                }
                _ => {
                    return Err(format!("Unsupported wire type: {}", wire_type));
                }
            }
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
    inner.extend(encode_varint(access_token.len()));
    inner.extend_from_slice(access_token.as_bytes());
    
    // Field 2: refresh_token (string)
    inner.push((2 << 3) | 2);
    inner.extend(encode_varint(refresh_token.len()));
    inner.extend_from_slice(refresh_token.as_bytes());
    
    // Field 3: expiry (int64)
    inner.push((3 << 3) | 0);
    inner.extend(encode_varint(expiry as usize));
    
    // 写入长度
    field.extend(encode_varint(inner.len()));
    field.extend(inner);
    
    field
}

/// 编码 Varint
fn encode_varint(mut value: usize) -> Vec<u8> {
    let mut result = Vec::new();
    
    while value >= 0x80 {
        result.push((value & 0x7F | 0x80) as u8);
        value >>= 7;
    }
    result.push(value as u8);
    
    result
}

/// 解码 Varint
fn decode_varint(data: &[u8]) -> Result<(usize, usize), String> {
    let mut result = 0usize;
    let mut shift = 0;
    let mut bytes_read = 0;
    
    for &byte in data.iter().take(10) {
        bytes_read += 1;
        result |= ((byte & 0x7F) as usize) << shift;
        
        if (byte & 0x80) == 0 {
            return Ok((result, bytes_read));
        }
        
        shift += 7;
    }
    
    Err("Varint too long".to_string())
}

