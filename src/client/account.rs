use crate::client::string_to_f64;
use serde::{Deserialize, Serialize};

use super::rest_response::Permission;

/// 账户类型
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    Spot,
    Margin,
    Futures,
    Leveraged,
}

/// 账户余额信息
#[derive(Debug, Deserialize, Serialize)]
pub struct Balance {
    asset: String,

    /// 可用的资产数量
    #[serde(deserialize_with = "string_to_f64")]
    pub free: f64,

    /// 冻结的资产数量
    #[serde(deserialize_with = "string_to_f64")]
    pub locked: f64,
}

/// 账户信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// maker手续费，10表示0.1%(10 * 0.0001)
    pub maker_commission: u16,
    /// taker手续费，10表示0.1%(10 * 0.0001)
    pub taker_commission: u16,
    pub buyer_commission: u16,
    pub seller_commission: u16,
    /// 能否交易
    pub can_trade: bool,
    /// 能否提现
    pub can_withdraw: bool,
    /// 能否充值
    pub can_deposit: bool,
    pub update_time: u64,
    pub account_type: AccountType,
    /// 余额信息
    pub balances: Vec<Balance>,
    /// 账户权限
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitType {
    RequestWeight,
    Orders,
    RawRequests,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitInterVal {
    Second,
    Minute,
    Day,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub rate_limit_type: RateLimitType,
    pub interval: RateLimitInterVal,
    pub interval_num: u32,
    pub limit: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitInfo {
    pub rate_limit_type: RateLimitType,
    pub interval: RateLimitInterVal,
    pub interval_num: u32,
    pub limit: u32,
    pub count: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenKey {
  #[serde(default)]
  pub listen_key: String,
}

