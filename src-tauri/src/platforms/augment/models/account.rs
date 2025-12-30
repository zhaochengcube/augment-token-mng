use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub email: Option<String>,
    pub suspensions: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    #[serde(rename = "portalUrl")]
    pub portal_url: Option<String>,
    #[serde(rename = "billingPeriodEnd")]
    pub billing_period_end: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteUserInfo {
    pub suspensions: Option<Value>,
    pub ban_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentMethodLinkResponse {
    pub success: bool,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentMethodLinkResult {
    pub payment_method_link: String,
}

// get-credit-info API 响应结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct CreditInfoResponse {
    pub usage_units_remaining: f64,
    pub usage_units_total: f64,
    pub current_billing_cycle_end_date_iso: String,
}

// /api/credits API 响应结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct CreditsInfoResponse {
    #[serde(rename = "usageUnitsAvailable")]
    pub usage_units_available: i32,
    #[serde(rename = "usageUnitsPending")]
    pub usage_units_pending: i32,
    #[serde(rename = "usageUnitsRemaining")]
    pub usage_units_remaining: i32,
    #[serde(rename = "usageUnitsConsumedThisBillingCycle")]
    pub usage_units_consumed_this_billing_cycle: i32,
    #[serde(rename = "usageBalanceStatus")]
    pub usage_balance_status: String,
}
