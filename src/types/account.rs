use std::collections::HashMap;

use crate::client::string_to_f64;
use serde::{Deserialize, Deserializer, Serialize};

/// 交易权限
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Permission {
    Spot,
    Margin,
    Futures,
    Leveraged,
    // #[allow(non_camel_case_types)]
    // TrdGrp_002,
    // #[allow(non_camel_case_types)]
    // TrdGrp_003,
    // #[allow(non_camel_case_types)]
    // TrdGrp_004,
    #[serde(other)]
    Unknown,
}

/// 账户类型
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    Spot,
    Margin,
    Futures,
    Leveraged,
    // #[allow(non_camel_case_types)]
    // TrdGrp_002,
    // #[allow(non_camel_case_types)]
    // TrdGrp_003,
    // #[allow(non_camel_case_types)]
    // TrdGrp_004,
    #[serde(other)]
    Unknown,
}

/// 目前用于字母账户划转时的账户类型
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubAccountType {
    Spot,
    IsolatedMargin,
    UsdtFuture,
    CoinFuture,
    #[serde(other)]
    Unknown,
}

impl From<&str> for SubAccountType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "spot" => Self::Spot,
            "usdtfuture" | "usdt_future" => Self::UsdtFuture,
            "coinfuture" | "coin_future" => Self::CoinFuture,
            "isolatedmargin" | "isolated_margin" => Self::IsolatedMargin,
            _ => Self::Unknown,
        }
    }
}

/// 账户余额信息
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RawBalance {
    /// 可用的资产数量
    pub free: f64,

    /// 冻结的资产数量
    pub locked: f64,
}

impl RawBalance {
    pub fn new() -> Self {
        RawBalance::default()
    }
}

/// 账户余额信息
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Balances(
    #[serde(deserialize_with = "balances_de_to_map")] pub HashMap<String, RawBalance>,
);

impl Balances {
    pub fn new() -> Balances {
        Self::default()
    }

    pub fn get_balance(&self, coin: &str) -> RawBalance {
        self.0
            .get(&coin.to_uppercase())
            .cloned()
            .unwrap_or_default()
    }

    /// 将另一个Balances合并到当前余额
    pub fn merge_balance(&mut self, balances: Balances) {
        for (key, value) in balances.0 {
            self.0.insert(key, value);
        }
    }
}

fn balances_de_to_map<'de, D>(deserializer: D) -> Result<HashMap<String, RawBalance>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum M {
        Vec(Vec<Balance>),
        HashMap(HashMap<String, RawBalance>),
    }

    match M::deserialize(deserializer)? {
        M::HashMap(bs) => Ok(bs),
        M::Vec(bs) => {
            let mut hash = HashMap::new();
            for i in &bs {
                hash.insert(
                    i.asset.to_string(),
                    RawBalance {
                        free: i.free,
                        locked: i.locked,
                    },
                );
            }
            Ok(hash)
        }
    }
}

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
    #[serde(rename(deserialize = "a"))]
    asset: String,

    /// 可用的资产数量
    #[serde(rename(deserialize = "f"))]
    #[serde(deserialize_with = "string_to_f64")]
    free: f64,

    /// 冻结的资产数量
    #[serde(rename(deserialize = "l"))]
    #[serde(deserialize_with = "string_to_f64")]
    locked: f64,
}

/// 账户信息
#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenKey {
    #[serde(default)]
    pub listen_key: String,
}

#[cfg(test)]
mod test_account {
    use super::*;
    #[test]
    fn account_serde_test() {
        let str = r##"
            {
              "makerCommission": 15,
              "takerCommission": 15,
              "buyerCommission": 0,
              "sellerCommission": 0,
              "canTrade": true,
              "canWithdraw": true,
              "canDeposit": true,
              "updateTime": 123456789,
              "accountType": "SPOT",
              "balances": [
                {
                  "asset": "BTC",
                  "free": "4723846.89208129",
                  "locked": "0.00000000"
                },
                {
                  "asset": "LTC",
                  "free": "4763368.68006011",
                  "locked": "0.00000000"
                }
              ],
              "permissions": [
                "SPOT"
              ]
            }
        "##;
        let res = serde_json::from_str::<Account>(str);
        println!("{:?}", res);
        assert!(res.is_ok());
    }
}
