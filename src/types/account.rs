use crate::client::string_to_f64;
use serde::{Deserialize, Serialize};


/// 交易权限
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Permission {
    Spot,
    Margin,
}

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
pub struct Balances(pub Vec<Balance>);

/// 账户余额信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(from = "WrapBalance")]
pub struct Balance {
    pub asset: String,

    /// 可用的资产数量
    pub free: f64,

    /// 冻结的资产数量
    pub locked: f64,
}

impl From<WrapBalance> for Balance {
    fn from(balance: WrapBalance) -> Self {
        match balance {
            WrapBalance::RestBalance(data) => Self {
                asset: data.asset,
                free: data.free,
                locked: data.locked,
            },
            WrapBalance::WebSocketBalance(data) => Self {
                asset: data.asset,
                free: data.free,
                locked: data.locked,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum WrapBalance {
    RestBalance(RestBalance),
    WebSocketBalance(WebSocketBalance),
}

#[derive(Debug, Deserialize, Serialize)]
struct RestBalance {
    asset: String,
    /// 可用的资产数量
    #[serde(deserialize_with = "string_to_f64")]
    free: f64,
    /// 冻结的资产数量
    #[serde(deserialize_with = "string_to_f64")]
    locked: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct WebSocketBalance {
    #[serde(rename = "a")]
    asset: String,

    /// 可用的资产数量
    #[serde(rename = "f")]
    #[serde(deserialize_with = "string_to_f64")]
    free: f64,

    /// 冻结的资产数量
    #[serde(rename = "l")]
    #[serde(deserialize_with = "string_to_f64")]
    locked: f64,
}

/// 账户信息
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// maker手续费，10表示0.1%(10 * 0.0001)
    #[serde(rename = "makerCommission")]
    pub maker_fee: u16,

    /// taker手续费，10表示0.1%(10 * 0.0001)
    #[serde(rename = "takerCommission")]
    pub taker_fee: u16,

    #[serde(rename = "buyerCommission")]
    pub buyer_fee: u16,
    #[serde(rename = "sellerCommission")]
    pub seller_fee: u16,
    
    /// 能否交易
    pub can_trade: bool,
    /// 能否提现
    pub can_withdraw: bool,
    /// 能否充值
    pub can_deposit: bool,
    pub update_time: u64,
    pub account_type: AccountType,
    /// 余额信息
    pub balances: Balances,
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
