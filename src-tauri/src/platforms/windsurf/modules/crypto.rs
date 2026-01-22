//! 跨平台加密模块
//! - Windows: DPAPI + AES-256-GCM
//! - macOS: Keychain + AES-128-CBC
//! - Linux: Secret Service + AES-128-CBC

use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use pbkdf2::pbkdf2_hmac;
use sha1::Sha1;
use rand::RngCore;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;

/// 加密后的数据结构
#[derive(Debug, Clone)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub iv: Vec<u8>,
}

impl EncryptedData {
    /// 转换为 Windsurf 期望的格式: "v10" + IV(16) + ciphertext
    pub fn to_windsurf_format(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(3 + 16 + self.ciphertext.len());
        result.extend_from_slice(b"v10");
        result.extend_from_slice(&self.iv);
        result.extend_from_slice(&self.ciphertext);
        result
    }
}

/// 使用 AES-128-CBC 加密（macOS/Linux）
pub fn encrypt_aes128_cbc(plaintext: &[u8], key: &[u8; 16]) -> Result<EncryptedData, String> {
    // 生成随机 IV
    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);
    
    // 计算填充后的长度
    let padded_len = ((plaintext.len() / 16) + 1) * 16;
    let mut buffer = vec![0u8; padded_len];
    buffer[..plaintext.len()].copy_from_slice(plaintext);
    
    // 加密
    let cipher = Aes128CbcEnc::new(key.into(), &iv.into());
    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, plaintext.len())
        .map_err(|e| format!("AES-128-CBC encryption failed: {:?}", e))?;
    
    Ok(EncryptedData {
        ciphertext: ciphertext.to_vec(),
        iv: iv.to_vec(),
    })
}

pub fn encrypt_aes128_cbc_with_iv(
    plaintext: &[u8],
    key: &[u8; 16],
    iv: &[u8; 16],
) -> Result<Vec<u8>, String> {
    let padded_len = ((plaintext.len() / 16) + 1) * 16;
    let mut buffer = vec![0u8; padded_len];
    buffer[..plaintext.len()].copy_from_slice(plaintext);

    let cipher = Aes128CbcEnc::new(key.into(), iv.into());
    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, plaintext.len())
        .map_err(|e| format!("AES-128-CBC encryption failed: {:?}", e))?;
    Ok(ciphertext.to_vec())
}

pub fn decrypt_aes128_cbc(ciphertext: &[u8], iv: &[u8], key: &[u8; 16]) -> Result<Vec<u8>, String> {
    if iv.len() != 16 {
        return Err("Invalid IV length for AES-128-CBC".to_string());
    }

    let mut buffer = ciphertext.to_vec();
    let cipher = cbc::Decryptor::<aes::Aes128>::new(key.into(), iv.into());
    let decrypted = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|e| format!("AES-128-CBC decryption failed: {:?}", e))?;
    Ok(decrypted.to_vec())
}

#[allow(dead_code)]
fn parse_windsurf_format(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    if data.len() < 19 || &data[..3] != b"v10" {
        return Err("Invalid encrypted payload format".to_string());
    }
    let iv = data[3..19].to_vec();
    let ciphertext = data[19..].to_vec();
    Ok((iv, ciphertext))
}

#[allow(dead_code)]
fn derive_key_from_password(password: &[u8], iterations: u32) -> [u8; 16] {
    let mut key = [0u8; 16];
    pbkdf2_hmac::<Sha1>(password, b"saltysalt", iterations, &mut key);
    key
}

// ============ Windows 实现 ============
#[cfg(windows)]
pub mod platform {
    use super::*;
    use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead, Nonce};
    use base64::{Engine as _, engine::general_purpose};
    use std::sync::OnceLock;
    use std::path::PathBuf;

    /// 缓存从 Local State 解密的 AES 密钥
    static CACHED_OS_CRYPT_KEY: OnceLock<[u8; 32]> = OnceLock::new();

    /// 获取 Windsurf Local State 文件路径
    fn get_local_state_path() -> Result<PathBuf, String> {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| "Cannot get APPDATA environment variable".to_string())?;
        Ok(PathBuf::from(appdata).join("Windsurf\\Local State"))
    }

    /// 从 Local State 文件读取并解密 os_crypt 密钥
    fn get_os_crypt_key() -> Result<[u8; 32], String> {
        if let Some(key) = CACHED_OS_CRYPT_KEY.get() {
            return Ok(*key);
        }

        let local_state_path = get_local_state_path()?;
        if !local_state_path.exists() {
            return Err(format!("Local State file not found: {:?}", local_state_path));
        }

        let content = std::fs::read_to_string(&local_state_path)
            .map_err(|e| format!("Failed to read Local State: {}", e))?;

        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse Local State JSON: {}", e))?;

        let encrypted_key_b64 = json
            .get("os_crypt")
            .and_then(|v| v.get("encrypted_key"))
            .and_then(|v| v.as_str())
            .ok_or("Missing os_crypt.encrypted_key in Local State")?;

        let encrypted_key = general_purpose::STANDARD
            .decode(encrypted_key_b64)
            .map_err(|e| format!("Failed to decode encrypted_key: {}", e))?;

        // 前5字节是 "DPAPI" 标识，跳过
        if encrypted_key.len() < 5 + 32 {
            return Err("encrypted_key too short".to_string());
        }
        if &encrypted_key[..5] != b"DPAPI" {
            return Err("Invalid encrypted_key format (missing DPAPI prefix)".to_string());
        }

        let dpapi_blob = &encrypted_key[5..];
        let decrypted = dpapi_decrypt(dpapi_blob)?;

        if decrypted.len() != 32 {
            return Err(format!("Decrypted key has wrong length: {} (expected 32)", decrypted.len()));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&decrypted);

        let _ = CACHED_OS_CRYPT_KEY.set(key);
        Ok(*CACHED_OS_CRYPT_KEY.get().unwrap())
    }

    /// Windows: 使用 DPAPI 保护的密钥 + AES-256-GCM (旧实现，保留兼容)
    pub fn encrypt_for_windsurf(plaintext: &[u8]) -> Result<Vec<u8>, String> {
        // 使用与 encrypt_for_secret_storage 相同的实现
        encrypt_for_secret_storage(plaintext)
    }

    /// Windows: 使用 Chromium os_crypt 模式加密（匹配 Electron safeStorage）
    /// 格式: "v10" + nonce(12) + ciphertext(包含16字节tag)
    pub fn encrypt_for_secret_storage(plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let key = get_os_crypt_key()?;

        // 生成随机 nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        // AES-256-GCM 加密
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Failed to create cipher: {:?}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| format!("AES-GCM encryption failed: {:?}", e))?;

        // 格式: "v10" + nonce(12) + ciphertext(包含tag)
        let mut result = Vec::with_capacity(3 + 12 + ciphertext.len());
        result.extend_from_slice(b"v10");
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Windows: 使用 Chromium os_crypt 模式解密（匹配 Electron safeStorage）
    pub fn decrypt_for_secret_storage(ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        if ciphertext.len() < 3 + 12 + 16 {
            return Err("Ciphertext too short".to_string());
        }
        if &ciphertext[..3] != b"v10" {
            return Err("Invalid ciphertext format (missing v10 prefix)".to_string());
        }

        let key = get_os_crypt_key()?;
        let nonce_bytes = &ciphertext[3..15];
        let encrypted_data = &ciphertext[15..];

        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("Failed to create cipher: {:?}", e))?;
        let nonce = Nonce::from_slice(nonce_bytes);
        let decrypted = cipher.decrypt(nonce, encrypted_data)
            .map_err(|e| format!("AES-GCM decryption failed: {:?}", e))?;

        Ok(decrypted)
    }
    
    /// Windows DPAPI 加密
    #[allow(dead_code)]
    fn dpapi_encrypt(data: &[u8]) -> Result<Vec<u8>, String> {
        use windows::Win32::Security::Cryptography::{
            CryptProtectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
        };
        use windows::Win32::Foundation::{LocalFree, HLOCAL};
        use std::ptr;
        
        unsafe {
            let mut input_blob = CRYPT_INTEGER_BLOB {
                cbData: data.len() as u32,
                pbData: data.as_ptr() as *mut u8,
            };
            
            let mut output_blob = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: ptr::null_mut(),
            };
            
            let result = CryptProtectData(
                &mut input_blob,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output_blob,
            );
            
            if result.is_err() {
                return Err("DPAPI encryption failed".to_string());
            }
            
            let encrypted = std::slice::from_raw_parts(
                output_blob.pbData,
                output_blob.cbData as usize,
            ).to_vec();
            
            let _ = LocalFree(HLOCAL(output_blob.pbData as *mut _));
            
            Ok(encrypted)
        }
    }

    fn dpapi_decrypt(data: &[u8]) -> Result<Vec<u8>, String> {
        use windows::Win32::Security::Cryptography::{
            CryptUnprotectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
        };
        use windows::Win32::Foundation::{LocalFree, HLOCAL};
        use std::ptr;

        unsafe {
            let mut input_blob = CRYPT_INTEGER_BLOB {
                cbData: data.len() as u32,
                pbData: data.as_ptr() as *mut u8,
            };

            let mut output_blob = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: ptr::null_mut(),
            };

            let result = CryptUnprotectData(
                &mut input_blob,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output_blob,
            );

            if result.is_err() {
                return Err("DPAPI decryption failed".to_string());
            }

            let decrypted = std::slice::from_raw_parts(
                output_blob.pbData,
                output_blob.cbData as usize,
            ).to_vec();

            let _ = LocalFree(HLOCAL(output_blob.pbData as *mut _));

            Ok(decrypted)
        }
    }
}

// ============ macOS 实现 ============
#[cfg(target_os = "macos")]
pub mod platform {
    use super::*;
    use std::sync::OnceLock;

    /// 缓存 Keychain 密码，避免多次弹出确认框
    static CACHED_KEYCHAIN_PASSWORD: OnceLock<Vec<u8>> = OnceLock::new();
    /// 缓存派生密钥
    static CACHED_DERIVED_KEY: OnceLock<[u8; 16]> = OnceLock::new();

    /// macOS: 使用 Windsurf Keychain 密码派生密钥 + AES-128-CBC
    /// 注意：统一使用 Windsurf Safe Storage，与 Electron safeStorage 一致
    pub fn encrypt_for_windsurf(plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let key = get_cached_derived_key()?;
        let encrypted = encrypt_aes128_cbc(plaintext, &key)?;
        Ok(encrypted.to_windsurf_format())
    }

    /// macOS: 使用 Keychain 密码派生密钥 + AES-128-CBC（匹配 Electron safeStorage）
    pub fn encrypt_for_secret_storage(plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let key = get_cached_derived_key()?;
        let iv = [b' '; 16];
        let ciphertext = encrypt_aes128_cbc_with_iv(plaintext, &key, &iv)?;
        let mut result = Vec::with_capacity(3 + ciphertext.len());
        result.extend_from_slice(b"v10");
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// macOS: 使用 Keychain 密码派生密钥 + AES-128-CBC 解密（匹配 Electron safeStorage）
    pub fn decrypt_for_secret_storage(ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        let key = get_cached_derived_key()?;
        if ciphertext.len() < 3 || &ciphertext[..3] != b"v10" {
            return Err("Invalid encrypted payload format".to_string());
        }
        let iv = [b' '; 16];
        let payload = &ciphertext[3..];
        decrypt_aes128_cbc(payload, &iv, &key)
    }

    /// 获取缓存的派生密钥（只访问一次 Keychain）
    fn get_cached_derived_key() -> Result<[u8; 16], String> {
        if let Some(key) = CACHED_DERIVED_KEY.get() {
            return Ok(*key);
        }

        let password = get_keychain_password_cached()?;
        let key = derive_key_from_password(&password, 1003);

        // 尝试缓存，如果已被其他线程设置则使用已有值
        let _ = CACHED_DERIVED_KEY.set(key);
        Ok(*CACHED_DERIVED_KEY.get().unwrap())
    }

    /// 获取缓存的 Keychain 密码（只访问一次 Keychain）
    fn get_keychain_password_cached() -> Result<Vec<u8>, String> {
        if let Some(password) = CACHED_KEYCHAIN_PASSWORD.get() {
            return Ok(password.clone());
        }

        let password = get_keychain_password()?;

        // 尝试缓存
        let _ = CACHED_KEYCHAIN_PASSWORD.set(password.clone());
        Ok(CACHED_KEYCHAIN_PASSWORD.get().unwrap().clone())
    }

    fn get_keychain_password() -> Result<Vec<u8>, String> {
        use security_framework::passwords::get_generic_password;

        const SERVICE: &str = "Windsurf Safe Storage";
        const ACCOUNT: &str = "Windsurf Key";

        match get_generic_password(SERVICE, ACCOUNT) {
            Ok(password) => Ok(password),
            Err(e) => Err(format!(
                "Keychain read failed: {:?}. Ensure Windsurf has been launched at least once to create the Keychain entry.",
                e
            )),
        }
    }
}

// ============ Linux 实现 ============
#[cfg(target_os = "linux")]
pub mod platform {
    use super::*;
    
    /// Linux: 从 Secret Service 获取密钥 + AES-128-CBC
    pub fn encrypt_for_windsurf(plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let key = get_or_create_secret_key()?;
        let encrypted = encrypt_aes128_cbc(plaintext, &key)?;
        Ok(encrypted.to_windsurf_format())
    }

    /// Linux: 使用 Secret Service 密码派生密钥 + AES-128-CBC（匹配 Electron safeStorage）
    /// 格式: "v11" + ciphertext (使用 Secret Service 存储的密码)
    pub fn encrypt_for_secret_storage(plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let password = get_windsurf_secret_password()?;
        let key = derive_key_from_password(&password, 1);
        let iv = [b' '; 16];
        let ciphertext = encrypt_aes128_cbc_with_iv(plaintext, &key, &iv)?;
        let mut result = Vec::with_capacity(3 + ciphertext.len());
        // 使用 v11 前缀表示使用 Secret Service 密码
        result.extend_from_slice(b"v11");
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Linux: 使用 Secret Service 密码派生密钥 + AES-128-CBC 解密（匹配 Electron safeStorage）
    pub fn decrypt_for_secret_storage(ciphertext: &[u8]) -> Result<Vec<u8>, String> {
        let password = get_windsurf_secret_password()?;
        let key = derive_key_from_password(&password, 1);
        // 支持 v10 和 v11 前缀
        if ciphertext.len() < 3 {
            return Err("Ciphertext too short".to_string());
        }
        let prefix = &ciphertext[..3];
        if prefix != b"v10" && prefix != b"v11" {
            return Err("Invalid encrypted payload format (expected v10 or v11 prefix)".to_string());
        }
        let iv = [b' '; 16];
        let payload = &ciphertext[3..];
        decrypt_aes128_cbc(payload, &iv, &key)
    }

    /// 从 Secret Service 获取 Windsurf 已创建的密码（只读取，不创建）
    fn get_windsurf_secret_password() -> Result<Vec<u8>, String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        rt.block_on(get_windsurf_secret_password_async())
    }

    async fn get_windsurf_secret_password_async() -> Result<Vec<u8>, String> {
        use std::collections::HashMap;
        use secret_service::{SecretService, EncryptionType};

        let ss = SecretService::connect(EncryptionType::Dh).await
            .map_err(|e| format!("Failed to connect to Secret Service: {:?}", e))?;

        let collection = ss.get_default_collection().await
            .map_err(|e| format!("Failed to get default collection: {:?}", e))?;

        if collection.is_locked().await.unwrap_or(true) {
            collection.unlock().await
                .map_err(|e| format!("Failed to unlock collection: {:?}", e))?;
        }

        // 搜索 Windsurf 创建的密码条目
        let attributes: HashMap<&str, &str> = [
            ("application", "windsurf"),
            ("xdg:schema", "chrome_libsecret_os_crypt_password_v2"),
        ].into_iter().collect();

        let items = collection.search_items(attributes).await
            .map_err(|e| format!("Failed to search items: {:?}", e))?;

        if let Some(item) = items.first() {
            let secret = item.get_secret().await
                .map_err(|e| format!("Failed to get secret: {:?}", e))?;
            return Ok(secret);
        }

        Err("Windsurf Safe Storage password not found in Secret Service. Please ensure Windsurf has been launched at least once.".to_string())
    }
    
    /// 从 Secret Service 获取或创建加密密钥（异步版本的同步包装）
    fn get_or_create_secret_key() -> Result<[u8; 16], String> {
        // 使用 tokio 运行异步代码
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;
        
        rt.block_on(get_or_create_secret_key_async())
    }
    
    async fn get_or_create_secret_key_async() -> Result<[u8; 16], String> {
        use std::collections::HashMap;
        use secret_service::{SecretService, EncryptionType};
        
        let ss = SecretService::connect(EncryptionType::Dh).await
            .map_err(|e| format!("Failed to connect to Secret Service: {:?}", e))?;
        
        let collection = ss.get_default_collection().await
            .map_err(|e| format!("Failed to get default collection: {:?}", e))?;
        
        // 确保集合已解锁
        if collection.is_locked().await.unwrap_or(true) {
            collection.unlock().await
                .map_err(|e| format!("Failed to unlock collection: {:?}", e))?;
        }
        
        let attributes: HashMap<&str, &str> = [
            ("application", "windsurf"),
            ("xdg:schema", "chrome_libsecret_os_crypt_password_v2"),
        ].into_iter().collect();
        
        // 尝试获取现有密钥
        let items = collection.search_items(attributes.clone()).await
            .map_err(|e| format!("Failed to search items: {:?}", e))?;
        
        if let Some(item) = items.first() {
            let secret = item.get_secret().await
                .map_err(|e| format!("Failed to get secret: {:?}", e))?;
            
            if secret.len() >= 16 {
                let mut key = [0u8; 16];
                key.copy_from_slice(&secret[..16]);
                return Ok(key);
            }
        }
        
        // 创建新密钥
        let mut key = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut key);
        
        collection.create_item(
            "Windsurf Safe Storage",
            attributes,
            &key,
            true, // replace
            "text/plain",
        ).await.map_err(|e| format!("Failed to create secret: {:?}", e))?;
        
        Ok(key)
    }

}
