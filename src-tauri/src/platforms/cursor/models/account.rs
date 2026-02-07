use serde::{Deserialize, Serialize};
use super::TokenData;
use crate::data::storage::common::SyncableAccount;

/// 机器码信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MachineInfo {
    #[serde(rename = "telemetry.machineId", default, skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<String>,
    #[serde(rename = "telemetry.macMachineId", default, skip_serializing_if = "Option::is_none")]
    pub mac_machine_id: Option<String>,
    #[serde(rename = "telemetry.devDeviceId", default, skip_serializing_if = "Option::is_none")]
    pub dev_device_id: Option<String>,
    #[serde(rename = "telemetry.sqmId", default, skip_serializing_if = "Option::is_none")]
    pub sqm_id: Option<String>,
    #[serde(rename = "system.machineGuid", default, skip_serializing_if = "Option::is_none")]
    pub system_machine_guid: Option<String>,
    #[serde(rename = "storage.serviceMachineId", default, skip_serializing_if = "Option::is_none")]
    pub storage_service_machine_id: Option<String>,
}

impl MachineInfo {
    /// 检查是否有任何有效的机器码数据
    pub fn has_data(&self) -> bool {
        self.machine_id.is_some()
            || self.mac_machine_id.is_some()
            || self.dev_device_id.is_some()
            || self.sqm_id.is_some()
            || self.system_machine_guid.is_some()
            || self.storage_service_machine_id.is_some()
    }
}

/// Cursor 账号数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub token: TokenData,

    /// 用户标签
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// 标签颜色
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag_color: Option<String>,

    /// 绑定的机器码信息
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine_info: Option<MachineInfo>,

    #[serde(default)]
    pub disabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_at: Option<i64>,
    pub created_at: i64,
    pub last_used: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub version: i64,
    #[serde(default)]
    pub deleted: bool,
}

impl SyncableAccount for Account {
    fn id(&self) -> &str {
        &self.id
    }

    fn email(&self) -> &str {
        &self.email
    }

    fn updated_at(&self) -> i64 {
        self.updated_at
    }

    fn version(&self) -> i64 {
        self.version
    }

    fn set_version(&mut self, version: i64) {
        self.version = version;
    }

    fn is_deleted(&self) -> bool {
        self.deleted
    }

    fn set_deleted(&mut self, deleted: bool) {
        self.deleted = deleted;
    }

    fn platform_name() -> &'static str {
        "cursor"
    }
}

impl Account {
    pub fn new(id: String, email: String, token: TokenData) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            email,
            name: None,
            token,
            tag: None,
            tag_color: None,
            machine_info: None,
            disabled: false,
            disabled_reason: None,
            disabled_at: None,
            created_at: now,
            last_used: now,
            updated_at: now,
            version: 0,
            deleted: false,
        }
    }

    /// 创建带机器码信息的账号
    pub fn new_with_machine_info(id: String, email: String, token: TokenData, machine_info: Option<MachineInfo>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            email,
            name: None,
            token,
            tag: None,
            tag_color: None,
            machine_info,
            disabled: false,
            disabled_reason: None,
            disabled_at: None,
            created_at: now,
            last_used: now,
            updated_at: now,
            version: 0,
            deleted: false,
        }
    }

    /// 检查账号是否有绑定的机器码
    pub fn has_machine_info(&self) -> bool {
        self.machine_info.as_ref().map_or(false, |info| info.has_data())
    }

    pub fn update_last_used(&mut self) {
        self.last_used = chrono::Utc::now().timestamp();
        self.updated_at = self.last_used;
    }
}

/// 账号索引数据（cursor_accounts.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountIndex {
    pub version: String,
    pub accounts: Vec<AccountSummary>,
    pub current_account_id: Option<String>,
}

/// 账号摘要信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSummary {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub created_at: i64,
    pub last_used: i64,
}

impl AccountIndex {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            accounts: Vec::new(),
            current_account_id: None,
        }
    }
}

impl Default for AccountIndex {
    fn default() -> Self {
        Self::new()
    }
}


/// 导出账号数据格式（与导入格式兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportAccountData {
    pub email: String,
    #[serde(rename = "auth_info", skip_serializing_if = "Option::is_none")]
    pub auth_info: Option<ExportAuthInfo>,
    #[serde(rename = "machine_info", skip_serializing_if = "Option::is_none")]
    pub machine_info: Option<ExportMachineInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportAuthInfo {
    #[serde(rename = "WorkosCursorSessionToken", skip_serializing_if = "Option::is_none")]
    pub workos_cursor_session_token: Option<String>,
    #[serde(rename = "cursorAuth/accessToken", skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(rename = "cursorAuth/refreshToken", skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMachineInfo {
    #[serde(rename = "telemetry.machineId", skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<String>,
    #[serde(rename = "telemetry.macMachineId", skip_serializing_if = "Option::is_none")]
    pub mac_machine_id: Option<String>,
    #[serde(rename = "telemetry.devDeviceId", skip_serializing_if = "Option::is_none")]
    pub dev_device_id: Option<String>,
    #[serde(rename = "telemetry.sqmId", skip_serializing_if = "Option::is_none")]
    pub sqm_id: Option<String>,
    #[serde(rename = "storage.serviceMachineId", skip_serializing_if = "Option::is_none")]
    pub storage_service_machine_id: Option<String>,
    #[serde(rename = "system.machineGuid", skip_serializing_if = "Option::is_none")]
    pub system_machine_guid: Option<String>,
}

impl ExportAccountData {
    pub fn from_account(account: &Account) -> Self {
        let auth_info = if account.token.workos_cursor_session_token.is_some()
            || !account.token.access_token.is_empty()
            || !account.token.refresh_token.is_empty()
        {
            Some(ExportAuthInfo {
                workos_cursor_session_token: account.token.workos_cursor_session_token.clone(),
                access_token: Some(account.token.access_token.clone()),
                refresh_token: if account.token.refresh_token.is_empty() {
                    None
                } else {
                    Some(account.token.refresh_token.clone())
                },
            })
        } else {
            None
        };

        let machine_info = account.machine_info.as_ref().filter(|m| m.has_data()).map(|m| {
            ExportMachineInfo {
                machine_id: m.machine_id.clone(),
                mac_machine_id: m.mac_machine_id.clone(),
                dev_device_id: m.dev_device_id.clone(),
                sqm_id: m.sqm_id.clone(),
                storage_service_machine_id: m.storage_service_machine_id.clone(),
                system_machine_guid: m.system_machine_guid.clone(),
            }
        });

        Self {
            email: account.email.clone(),
            auth_info,
            machine_info,
        }
    }
}
